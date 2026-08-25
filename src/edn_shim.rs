//! `:wat::edn::*` — render any wat value as EDN/JSON text.
//!
//! Arc 079. The wat-edn crate ships a complete EDN parser/writer +
//! JSON bridge in Rust. This module exposes its WRITER side to wat
//! through three primitives that take any wat `Value` and return a
//! `String`:
//!
//! - `:wat::edn::write v` → compact EDN, single line (default for
//!   line-per-record logging).
//! - `:wat::edn::write-pretty v` → multi-line indented EDN (debug /
//!   diagnostic output).
//! - `:wat::edn::write-json v` → JSON via wat-edn's sentinel-key
//!   tagged-object convention. Round-trip-safe with
//!   `:wat::edn::parse` (slice 2; not yet shipped).
//!
//! # The walker
//!
//! [`value_to_edn`] converts a wat `Value` into a `wat_edn::OwnedValue`.
//! Per-variant mapping lives there; the three eval functions are thin
//! wrappers that call the writer and return the string.
//!
//! Coverage in slice 1:
//!
//! | wat Value variant | wat-edn output |
//! |---|---|
//! | Unit | `nil` |
//! | bool | `true` / `false` |
//! | i64 / u8 | `Integer` |
//! | f64 (incl. NaN/Inf) | `Float` (sentinel-tagged for non-finite) |
//! | String | quoted EDN string |
//! | keyword | `Keyword` (namespace split at last `::`) |
//! | Vec | `Vector` |
//! | Tuple | `Vector` (no tuple distinction in EDN) |
//! | Option(None) | `Tagged #wat.core.Option/None []` (arc 278 A.0) |
//! | Option(Some(v)) | `Tagged #wat.core.Option/Some [v]` (arc 278 A.0) |
//! | Result(Ok(v)) | `Tagged #wat.core.Result/Ok [v]` (arc 278 A.0) |
//! | Result(Err(e)) | `Tagged #wat.core.Result/Err [e]` (arc 278 A.0) |
//! | HashMap | `Map` |
//! | HashSet | `Set` |
//! | Struct | `Tagged #ns/Type {:field-0 v0 :field-1 v1 ...}` |
//! | Enum | `Tagged #ns/Variant [v0 v1 ...]` (unit variant → `[]`) |
//! | HolonAST | DATA, never a wat source form (arc 294.j RELAND): a data-shaped holon renders as the plain EDN `from_holon_item` recovers; `Thermometer`/`SlotMarker` (constructor directives) render as `#wat.holon/Thermometer {…}` / `#wat.holon/SlotMarker {…}`; the algebra (Bind/Bundle/Atom/Permute/Blend) never crosses the wire in any form — encoding one RAISES |
//! | All other substrate handles | `Tagged #wat.<home>/<TypeName> nil` (arc 294.i — per-type home, not a shared bucket) |
//!
//! # Performance
//!
//! Walks the wat value tree once; constructs an `OwnedValue` tree in
//! memory; passes to wat-edn's `write` / `write_pretty` /
//! `to_json_string`. The intermediate tree is the cost; for typical
//! log-line sizes (a struct with ~5 fields) it's well under 1µs per
//! value.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::value::{AggregateValue, ForeignRecordValue, ForeignVariantValue};
use crate::scope::Identifier;
use crate::span::{span_prefix, Span};
use std::sync::Arc;
use wat_edn::{Keyword, OwnedValue, Tag};

// ─── Public eval entry points ────────────────────────────────────

/// `(:wat::edn::write v)` → `:String`. Compact single-line EDN.
pub fn eval_edn_write(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::write(&edn))))
}

/// `(:wat::edn::write-pretty v)` → `:String`. Multi-line indented EDN.
pub fn eval_edn_write_pretty(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-pretty";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::write_pretty(&edn))))
}

/// `(:wat::edn::write-json v)` → `:String`. JSON via wat-edn's
/// round-trip-safe sentinel-tagged-object convention.
pub fn eval_edn_write_json(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-json";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::to_json_string(&edn))))
}

pub(crate) fn require_one_arg(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &crate::span::Span,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        }));
    }
    eval(&args[0], env, sym).map(|tv| tv.value_owned())
}

/// `(:wat::edn::write-json-natural v)` → `:String`. Ingestion-tooling-
/// friendly JSON. Drops the `#tag`/`body` sentinel wrapping (so
/// struct fields land at the top level of the JSON object), drops
/// the `:` prefix from keywords (so they read as plain JSON strings),
/// renders Instants as bare ISO-8601 strings (no `{"#inst": ...}`
/// wrapper). Encodes enum tagged variants with a `_type`
/// discriminator + the variant's named fields at the top level.
///
/// Lossy. Suitable for pumping logs into ELK / DataDog / CloudWatch
/// Logs / etc. — formats that expect a "natural" JSON shape.
/// Round-trip back to wat values is not preserved; use `write-json`
/// for round-trip-safe JSON encoding.
pub fn eval_edn_write_json_natural(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-json-natural";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = value_to_json_natural(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::to_json_string(&edn))))
}

/// `(:wat::edn::read s)` → `:T`. Parses an EDN string into a wat
/// runtime Value. The polymorphic-fresh-var return type lets the
/// caller's binding context unify with whatever shape the parsed
/// value takes; runtime mismatches (e.g. parsed value is a
/// HashMap but the caller expects a struct) surface as
/// pattern-match / accessor errors at the use site.
///
/// Tag dispatch — the body shape disambiguates struct vs enum:
///   - Tagged + Map body → look up `:<dotted-ns>::<name>` as Struct;
///     reconstruct `Value::Aggregate(Struct)` with declared field names.
///   - Tagged + Vector body → look up `:<dotted-ns>` as Enum; find
///     variant `<name>`; reconstruct `Value::Enum` with the vector
///     elements as positional fields.
///   - Tagged + Nil body → enum unit-variant; same lookup as above.
///   - `#inst` (handled by wat-edn parser) → `Value::Instant`.
///   - Other tags → `EdnReadError::UnknownTag` panic; consumer sees
///     the path that failed.
// Arc 233 Stone 233.2.j: returns TrackedValue directly (no Value::Tracked wrap).
pub fn eval_edn_read(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::edn::read";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(other))
            }));
        }
    };
    let edn = wat_edn::parse_owned(&s).map_err(|e| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("EDN parse error: {e}")
    }))?;
    // Arc 233 Stone 233.2.c — wrap result in Tracked with RuntimeBuilt provenance
    // so that errors flowing from edn::read-produced Values surface the producer origin.
    let result = edn_to_value(
        &edn,
        sym.types().map(|a| a.as_ref()),
        sym.encoding_ctx().map(|a| a.as_ref()),
    ).map_err(|e| {
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: e.to_string()
        })
    })?;
    // Arc 233 Stone 233.2.j: construct TrackedValue::new directly (no Value::Tracked wrap).
    Ok(crate::value::TrackedValue::new(
        result,
        crate::value::Provenance::RuntimeBuilt {
            producer: ":wat::edn::read",
            call_span: list_span.clone(),
        },
    ))
}

// Arc 278 Stone 1 (`wat --mcp`) — the type path of `:wat::edn::ReadJsonOutcome` (registered in
// `types.rs`, beside `:wat::core::ReadOutcome`).
const READ_JSON_OUTCOME_TYPE: &str = ":wat::edn::ReadJsonOutcome";
const READ_FOREIGN_OUTCOME_TYPE: &str = ":wat::edn::ReadForeignOutcome";

/// `ReadJsonOutcome::Value` / `ReadForeignOutcome::Value` — the decoded value.
fn tagged_read_outcome_value(type_path: &str, value: Value) -> Value {
    Value::Enum(std::sync::Arc::new(crate::runtime::EnumValue {
        type_path: type_path.into(),
        variant_name: "Value".into(),
        names: crate::runtime::builtin_enum_variant_names(type_path, "Value"),
        fields: vec![value],
    }))
}

/// `ReadJsonOutcome::Value [value]` — the decoded value.
fn read_json_outcome_value(value: Value) -> Value {
    tagged_read_outcome_value(READ_JSON_OUTCOME_TYPE, value)
}

/// `ReadJsonOutcome::Malformed [cause]` — the JSON text did not parse, or the parsed JSON did
/// not decode to a runtime value.
///
/// `wat_edn::JsonError` (the error `from_json_string` raises) CANNOT impl `WatError`: it lives in
/// the `wat-edn` crate, and the trait lives in `src/to_edn.rs` (the orphan rule forbids the
/// reverse impl). The message is lifted through `FlatMessage` — the existing adapter for a
/// genuinely flat, structure-free failure (`to_edn.rs:346`) — then decoded back to a typed
/// `:wat::core::Error` via the IDENTICAL tail the `read-string` Malformed helper uses: the STRICT
/// decode is preferred and the FOREIGN (data-mode) decode is the fallback for tags the type
/// registry does not carry yet. Same reasoning as that helper: a structured
/// diagnostic flattened into a String is the mask this arc exists to kill, and a lossy carrier is
/// what makes that mask mandatory.
fn tagged_read_outcome_malformed(
    type_path: &str,
    error_tag: &str,
    message: &str,
    sym: &SymbolTable,
    list_span: &crate::span::Span,
) -> Value {
    use crate::to_edn::WatError;
    let flat = crate::to_edn::FlatMessage {
        tag: error_tag,
        key: "reason",
        message,
    };
    let cause_edn = wat_edn::write(&flat.error_edn());
    let types = sym.types().map(|t| &**t);
    let ctx = sym.encoding_ctx().map(|c| &**c);
    // Arc 109 — the decoded diagnostic rides as a CAUSE under a real `:wat::core::Fault`,
    // never AS the returned value. This variant's cause field is DECLARED
    // `:wat::core::Error`; the FOREIGN arm below yields a `Value::ForeignRecord`, a
    // self-describing dynamic bag that satisfies that surface NOWHERE — so returning it
    // directly made the declared type a lie at the boundary, and every consumer calling
    // `(:wat::core::Error/message __cause)` died with `UnknownFunction: ForeignRecord does
    // not implement surface method message` instead of reporting the failure. 75 such call
    // sites across 57 files (wat/fix.wat, lint.wat, service.wat, core.wat's
    // string::interpolate, deporder.wat, telemetry/journal.wat, and 32 of the 66 recorded
    // migrations) — written and never once invoked, because until arc 109's lexer walls
    // landed the reader never failed on corpus text. `check_failed_cause` in `runtime.rs`
    // ran the identical ladder and already disposed of it correctly; all three now go
    // through the one `fault_with_cause` door.
    let cause = decode_trusted_wire(&cause_edn, types, ctx)
        .or_else(|_| {
            wat_edn::parse_owned(&cause_edn)
                .map_err(|_| ())
                .and_then(|owned| edn_to_value_foreign(&owned, types, ctx).map_err(|_| ()))
        })
        .map(|inner| crate::runtime::fault_with_cause(message.to_string(), list_span.clone(), inner))
        .unwrap_or_else(|_| {
            // A FlatMessage whose own EDN will not decode is itself a defect; report the
            // headline as a minimal TRUE record rather than smuggling the tree back in as prose.
            Value::Aggregate(std::sync::Arc::new(
                crate::value::value::AggregateValue::record(
                    "wat::core::Fault".into(),
                    crate::runtime::fault_names(),
                    std::sync::Arc::new(vec![
                        Value::String(std::sync::Arc::new(message.to_string())),
                        crate::runtime::value_from_span(list_span.clone()),
                        Value::Vec(std::sync::Arc::new(Vec::new())),
                    ]),
                ),
            ))
        });
    Value::Enum(std::sync::Arc::new(crate::runtime::EnumValue {
        type_path: type_path.into(),
        variant_name: "Malformed".into(),
        names: crate::runtime::builtin_enum_variant_names(type_path, "Malformed"),
        fields: vec![cause],
    }))
}

fn read_json_outcome_malformed(
    message: &str,
    sym: &SymbolTable,
    list_span: &crate::span::Span,
) -> Value {
    tagged_read_outcome_malformed(READ_JSON_OUTCOME_TYPE, "JsonReadError", message, sym, list_span)
}

/// `(:wat::edn::read-json s)` → `:wat::edn::ReadJsonOutcome`. Arc 278 Stone 1 (`wat --mcp`) —
/// decodes a JSON string into a wat runtime Value: `wat_edn::from_json_string`
/// (`crates/wat-edn/src/json.rs:225`, the JSON→EDN bridge) then `edn_to_value` (the same typed
/// decode `:wat::edn::read` uses). TOTAL, never raises — see `ReadJsonOutcome` in `types.rs` for
/// why: this verb's input arrives from a REMOTE, UNTRUSTED harness over stdio, so a malformed
/// byte must not be able to end the session.
pub fn eval_edn_read_json(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::edn::read-json";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    let types = sym.types().map(|a| a.as_ref());
    let ctx = sym.encoding_ctx().map(|a| a.as_ref());
    let value = match wat_edn::from_json_string(&s) {
        Ok(owned) => match edn_to_value(&owned, types, ctx) {
            Ok(v) => read_json_outcome_value(v),
            Err(e) => read_json_outcome_malformed(&e.to_string(), sym, list_span),
        },
        Err(e) => read_json_outcome_malformed(&e.to_string(), sym, list_span),
    };
    Ok(crate::value::TrackedValue::new(
        value,
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// `(:wat::edn::read-foreign s)` → `:wat::edn::ReadForeignOutcome<T>`. Arc 278
/// Stone A — the DATA-MODE sibling of [`eval_edn_read`]. Same
/// String→`parse_owned`→decode path, but an UNKNOWN tag reconstructs a
/// self-describing dynamic value (`ForeignRecord` for a map body,
/// `ForeignVariant` for a vector body) instead of raising `UnknownTag`.
/// Recursive: nested unknown tags decode all the way down. STRICT
/// [`eval_edn_read`] is UNCHANGED (unknown tag still errors — the
/// no-hidden-failures floor, R41 EGO SVM LEX). The consumer that HOLDS a type
/// uses `read`; the consumer that LACKS it uses `read-foreign`.
///
/// TOTAL — parse/decode failure is `:Malformed`, never a raise. Type/arity
/// mismatches still raise (the type checker's concern, same as `read-json`).
pub fn eval_edn_read_foreign(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::edn::read-foreign";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(other))
            }));
        }
    };
    let types = sym.types().map(|a| a.as_ref());
    let ctx = sym.encoding_ctx().map(|a| a.as_ref());
    let value = match wat_edn::parse_owned(&s) {
        Ok(edn) => match edn_to_value_foreign(&edn, types, ctx) {
            Ok(v) => tagged_read_outcome_value(READ_FOREIGN_OUTCOME_TYPE, v),
            Err(e) => tagged_read_outcome_malformed(
                READ_FOREIGN_OUTCOME_TYPE,
                "ForeignReadError",
                &e.to_string(),
                sym,
                list_span,
            ),
        },
        Err(e) => tagged_read_outcome_malformed(
            READ_FOREIGN_OUTCOME_TYPE,
            "ForeignReadError",
            &format!("EDN parse error: {e}"),
            sym,
            list_span,
        ),
    };
    Ok(crate::value::TrackedValue::new(
        value,
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// Arc 278 Stone A — extract the bare field name a `ForeignRecord/get` key
/// keyword refers to. A wat keyword value carries its leading `:` (and possibly
/// a `::`-namespace); foreign field keys are the bare name (as read off the
/// wire via `Keyword::name()`), so strip the `:` and take the last `::`-segment.
fn foreign_key_name(kw: &str) -> String {
    let body = kw.strip_prefix(':').unwrap_or(kw);
    match body.rsplit_once("::") {
        Some((_, last)) => last.to_string(),
        None => body.to_string(),
    }
}

/// `(:wat::edn::ForeignRecord/get fr :key)` → `:wat::core::Option<wat::core::Value>`.
/// Arc 278 Stone A — navigate a foreign record BY KEY (the consumer holds no
/// type). Same contract as `HashMap/get` / `PersistentMap/get`: miss is `None`,
/// never a raise. The inner value is `Value` (heterogeneous dynamic boundary —
/// R7 universal top): a leaf, or a nested `ForeignRecord`/`ForeignVariant`.
/// Type/arity mismatches still raise (the type checker's concern, not this
/// axis — same convention as `HashMap/get`).
pub fn eval_foreign_record_get(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::ForeignRecord/get";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(), expected: 2, got: args.len()
        }));
    }
    let fr_v = eval(&args[0], env, sym).map(|tv| tv.value_owned())?;
    let key_v = eval(&args[1], env, sym).map(|tv| tv.value_owned())?;
    let fr = match &fr_v {
        Value::ForeignRecord(fr) => fr,
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::edn::ForeignRecord",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    let key = match &key_v {
        Value::wat__core__keyword(k) => foreign_key_name(k),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Keyword",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    match fr.fields.iter().find(|(k, _)| *k == key) {
        Some((_, v)) => Ok(Value::Option(Arc::new(Some(v.clone())))),
        None => Ok(Value::Option(Arc::new(None))),
    }
}

/// `(:wat::edn::ForeignRecord/class fr)` → `:wat::core::String`. Arc 278
/// Stone A — the record's fully-qualified (colon-free) class string.
pub fn eval_foreign_record_class(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::ForeignRecord/class";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    match &v {
        Value::ForeignRecord(fr) => Ok(Value::String(Arc::new(fr.class.clone()))),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::edn::ForeignRecord",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    }
}

/// `(:wat::edn::ForeignVariant/variant v)` → `:wat::core::Keyword`. Arc 278
/// Stone A — the variant name as a keyword (`:Click`). Traffics in `Value` at
/// the argument boundary (heterogeneous), runtime-checking it is a
/// `ForeignVariant` and raising a clean located error otherwise
/// (no-hidden-failures, R41).
pub fn eval_foreign_variant_variant(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::ForeignVariant/variant";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    match &v {
        Value::ForeignVariant(fv) => {
            Ok(Value::wat__core__keyword(Arc::new(format!(":{}", fv.variant))))
        }
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::edn::ForeignVariant",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    }
}

/// `(:wat::edn::ForeignVariant/enum-class v)` → `:wat::core::String`. Arc 278
/// Stone A — the enum's fully-qualified (colon-free) class string.
pub fn eval_foreign_variant_enum_class(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::ForeignVariant/enum-class";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    match &v {
        Value::ForeignVariant(fv) => Ok(Value::String(Arc::new(fv.enum_class.clone()))),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::edn::ForeignVariant",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    }
}

/// `(:wat::edn::ForeignVariant/fields v)` → `:wat::core::Vector<Value>`. Arc 278
/// Stone A — the positional fields as a vector (each element a `Value`, itself
/// possibly a nested foreign value).
pub fn eval_foreign_variant_fields(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::ForeignVariant/fields";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    match &v {
        Value::ForeignVariant(fv) => Ok(Value::Vec(Arc::new(fv.fields.clone()))),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::edn::ForeignVariant",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    }
}

/// `(:wat::core::read-string <source>)` — arc 251 Stone 251.5a-i.
///
/// The homoiconic `read`: parse wat SOURCE text into forms-as-DATA, WITHOUT
/// evaluating. Returns the program's top-level forms wrapped in a single
/// `Value::wat__WatAST(WatAST::List([form0 form1 …]))` — the same AST-as-value
/// shape `quote` produces, so the macro engine + `List?`/`first`/`rest` walk it.
///
/// This is what `:wat::edn::read` is NOT: `edn::read` runs the EDN parser
/// (`wat_edn::parse_owned`), which rejects the pre-251.5 surface (`::`, `<>`,
/// `Fn(…)->`). `read-string` runs wat's OWN source parser, so it reads the corpus
/// as it stands today — the foundation the wat-to-wat fixer needs to read what it
/// is about to rewrite. (Once the migration lands, source IS clean EDN and the two
/// converge; until then, only the source parser can read the dirty corpus.)
/// Arc 170 — the type path of `:wat::core::ReadOutcome` (registered in `types.rs`).
const READ_OUTCOME_TYPE: &str = ":wat::core::ReadOutcome";

/// `ReadOutcome::Forms [forms]` — the parsed top-level forms.
fn read_outcome_forms(forms: Value) -> Value {
    Value::Enum(std::sync::Arc::new(crate::runtime::EnumValue {
        type_path: READ_OUTCOME_TYPE.into(),
        variant_name: "Forms".into(),
        names: crate::runtime::builtin_enum_variant_names(READ_OUTCOME_TYPE, "Forms"),
        fields: vec![forms],
    }))
}

/// `ReadOutcome::Malformed [cause]` — the text did not parse.
///
/// The cause is the parser's own `error_edn()` floor record, decoded back to a typed value; the
/// STRICT decode is preferred and the FOREIGN (data-mode) decode is the fallback for tags the type
/// registry does not carry yet. Identical ladder to `check_failed_cause` in `runtime.rs`, and for
/// the identical reason: a structured diagnostic flattened into a String is the mask this arc
/// exists to kill, and a lossy carrier is what makes that mask mandatory.
fn read_outcome_malformed(e: &crate::parser::ParseError, sym: &SymbolTable) -> Value {
    use crate::to_edn::WatError;
    let cause_edn = wat_edn::write(&e.error_edn());
    let types = sym.types().map(|t| &**t);
    let ctx = sym.encoding_ctx().map(|c| &**c);
    // Arc 109 — the decoded diagnostic rides as a CAUSE under a real `:wat::core::Fault`,
    // never AS the returned value. This variant's cause field is DECLARED
    // `:wat::core::Error`; the FOREIGN arm below yields a `Value::ForeignRecord`, a
    // self-describing dynamic bag that satisfies that surface NOWHERE — so returning it
    // directly made the declared type a lie at the boundary, and every consumer calling
    // `(:wat::core::Error/message __cause)` died with `UnknownFunction: ForeignRecord does
    // not implement surface method message` instead of reporting the failure. 75 such call
    // sites across 57 files (wat/fix.wat, lint.wat, service.wat, core.wat's
    // string::interpolate, deporder.wat, telemetry/journal.wat, and 32 of the 66 recorded
    // migrations) — written and never once invoked, because until arc 109's lexer walls
    // landed the reader never failed on corpus text. `check_failed_cause` in `runtime.rs`
    // ran the identical ladder and already disposed of it correctly; all three now go
    // through the one `fault_with_cause` door.
    let cause = decode_trusted_wire(&cause_edn, types, ctx)
        .or_else(|_| {
            wat_edn::parse_owned(&cause_edn)
                .map_err(|_| ())
                .and_then(|owned| edn_to_value_foreign(&owned, types, ctx).map_err(|_| ()))
        })
        .map(|inner| crate::runtime::fault_with_cause(e.message(), e.span.clone(), inner))
        .unwrap_or_else(|_| {
            // A parse error whose own EDN will not decode is itself a defect; report the headline
            // as a minimal TRUE record rather than smuggling the tree back in as prose.
            Value::Aggregate(std::sync::Arc::new(
                crate::value::value::AggregateValue::record(
                    "wat::core::Fault".into(),
                    crate::runtime::fault_names(),
                    std::sync::Arc::new(vec![
                        Value::String(std::sync::Arc::new(e.message())),
                        crate::runtime::value_from_span(e.span.clone()),
                        Value::Vec(std::sync::Arc::new(Vec::new())),
                    ]),
                ),
            ))
        });
    Value::Enum(std::sync::Arc::new(crate::runtime::EnumValue {
        type_path: READ_OUTCOME_TYPE.into(),
        variant_name: "Malformed".into(),
        names: crate::runtime::builtin_enum_variant_names(READ_OUTCOME_TYPE, "Malformed"),
        fields: vec![cause],
    }))
}

pub fn eval_read_string(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::read-string";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    // Arc 170 — TOTAL. A parse failure is a matchable `ReadOutcome::Malformed`, never a raise:
    // wat has no try/catch, so a raise here is unsurvivable by construction, and at a REPL one
    // stray control byte ended the session. The cause is the parser's OWN structured diagnostic
    // (`ParseError` impls `WatError` at `src/parser.rs`, so `error_edn()` composes the
    // message/location/causes floor), decoded back to a typed value — the tree stays navigable
    // down to `#wat.parse/Lex` and its span, rather than being flattened into a message String.
    let value = match crate::parser::parse_all_with_file(&s, "<read-string>") {
        Ok(forms) => {
            let ast = WatAST::List(forms, crate::rust_caller_span!());
            read_outcome_forms(Value::wat__WatAST(std::sync::Arc::new(ast)))
        }
        Err(e) => read_outcome_malformed(&e, sym),
    };
    Ok(crate::value::TrackedValue::new(
        value,
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// `(:wat::core::write-forms <forms>)` — arc 251 Stone 251.5a-ii.
///
/// The write side of the homoiconic round-trip: serialize a forms-value
/// (`Value::wat__WatAST`, as produced by `read-string` or `quote`) to a clean EDN
/// String, via the structural bridge (`watast_to_edn` + `wat_edn::write`). This is
/// what the general `edn::write` is NOT for forms — CORRECTED (arc 278): `value_to_edn`
/// ALSO serializes a `wat__WatAST` FAITHFULLY (via `watast_to_edn`), NOT opaque-nil
/// (only genuinely-opaque LIVE values nil). The real catch is the READ side: the general
/// edn *decoder* cannot rebuild a form (a form's bare symbols have no value type — see
/// the `Edn::Symbol` arm ~:1440), and every write path dialect-translates `::`→`.`. To
/// round-trip a form back to an evaluable `::`-AST use `read-string` (or an `ast->source`
/// printer for `::`-faithful text), never the general edn codec. `write-forms` serializes
/// the AST faithfully — `read-string → transform → write-forms` is the wat-to-wat fixer's
/// full read→rewrite→write cycle, all in wat's own primitives.
pub fn eval_write_forms(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::write-forms";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    let edn = crate::wat_edn_bridge::watast_to_edn(ast);
    let text = wat_edn::write(&edn);
    Ok(crate::value::TrackedValue::new(
        Value::String(std::sync::Arc::new(text)),
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// `(:wat::core::ast->source <ast>)` — arc 278 Stone 1 (the sift Predicate's enabling
/// primitive).
///
/// The resurrection of the retired `wat_ast_to_source` (`crates/wat-reader/src/ast.rs:459-466`,
/// RETIRED in arc 012 slice 3, explicitly inviting reintroduction as a stdlib primitive).
/// Serializes a `Value::wat__WatAST` back to VERBATIM wat source — every `::` keyword/symbol
/// printed UNTOUCHED. This is deliberately NOT `write-forms` (which goes through
/// `watast_to_edn` + `wat_edn::write`, and those dial `::` → `.` — GROUNDED: `write-forms` on a
/// `::`-form emits `:wat.core/fn`, not `:wat::core::fn`). `ast->source` walks the AST directly
/// so `read-string(ast->source(form))` reproduces the SAME form — the `::` notation survives
/// round-trip untranslated. See `write_wat_source` below for the per-variant spellings, each
/// grounded against the parser/lexer so `read-string` re-reads to an identical node.
pub fn eval_ast_to_source(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast->source";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    let mut out = String::new();
    write_wat_source(ast, &mut out);
    Ok(crate::value::TrackedValue::new(
        Value::String(std::sync::Arc::new(out)),
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// Recursive verbatim printer for [`eval_ast_to_source`]. Resurrects the retired
/// `write_wat_ast` (`b5bca8be^:src/ast.rs:101-138`) and ADDS the 7 variants born since:
/// `RationalLit`, `BigIntLit`, `CharLit`, `NilLit`, `Vector`, `Map`, `Set`. Each spelling is
/// grounded against the parser (`crates/wat-reader/src/parser.rs`) / lexer
/// (`crates/wat-reader/src/lexer.rs`) so `read-string` re-reads the printed text to an
/// identical node:
/// - `RationalLit`: `BigRational`'s `Display` prints `numer/denom` (already-reduced, sign on
///   numerator, den >= 2 by construction — `lexer.rs:888-902` mirrors this on the read side).
/// - `BigIntLit`: digits + trailing `N` suffix (`lexer.rs:911-917`, the `N`-suffix lane).
/// - `CharLit` (arc 300 stone D): `\c`, using the named forms (`\newline`/`\return`/`\space`/
///   `\tab`) for the four chars a bare `\<char>` can't spell unambiguously — mirrors
///   `lexer.rs::lex_char`'s read side.
/// - `NilLit`: bare `nil` (parser produces `NilLit` for bare `nil`, per `ast.rs:94-98`).
/// - `Vector`: `[` items space-joined `]` (`parser.rs:283-296`, `Token::LBracket`).
/// - `Map`: `{` alternating key/value space-joined `}` (`parser.rs:528-556`,
///   `parse_map_literal_body` — comma is EDN whitespace, `lexer.rs:379`, so plain spaces
///   round-trip identically).
/// - `Set`: `#{` items space-joined `}` (`parser.rs:571-576`, `Token::LHashBrace`).
pub(crate) fn write_wat_source(ast: &WatAST, out: &mut String) {
    match ast {
        WatAST::IntLit(n, _) => out.push_str(&n.to_string()),
        WatAST::FloatLit(x, _) => {
            // `{:?}` keeps the decimal point for integral floats — `3.0` serializes as
            // `3.0`, which parses back as FloatLit. `{}` would emit `3`, which parses as
            // IntLit and would round-trip to a different variant.
            out.push_str(&format!("{:?}", x));
        }
        WatAST::RationalLit(r, _) => out.push_str(&r.to_string()),
        WatAST::BigIntLit(n, _) => {
            out.push_str(&n.to_string());
            out.push('N');
        }
        // Arc 300 stone D — CharLit prints as `\c`, using the lexer's named
        // forms (`lexer.rs::lex_char`) for the four whitespace-family
        // chars a bare `\<char>` couldn't spell unambiguously, and
        // `\uNNNN` for anything outside BMP printable single-char form
        // (defensive; CharLit is BMP-only by construction). Every other
        // char (alphanumeric or not) round-trips as a literal `\c`.
        WatAST::CharLit(c, _) => match c {
            '\n' => out.push_str("\\newline"),
            '\r' => out.push_str("\\return"),
            ' ' => out.push_str("\\space"),
            '\t' => out.push_str("\\tab"),
            other => out.push_str(&format!("\\{}", other)),
        },
        WatAST::BoolLit(b, _) => out.push_str(if *b { "true" } else { "false" }),
        WatAST::StringLit(s, _) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    other => out.push(other),
                }
            }
            out.push('"');
        }
        WatAST::NilLit(_) => out.push_str("nil"),
        WatAST::Keyword(k, _) => out.push_str(k),
        WatAST::Symbol(ident, _) => out.push_str(ident.as_str()),
        WatAST::List(items, _) => {
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_wat_source(item, out);
            }
            out.push(')');
        }
        WatAST::Vector(items, _) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_wat_source(item, out);
            }
            out.push(']');
        }
        WatAST::Set(items, _) => {
            out.push_str("#{");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_wat_source(item, out);
            }
            out.push('}');
        }
        WatAST::Map(pairs, _) => {
            out.push('{');
            for (i, (k, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_wat_source(k, out);
                out.push(' ');
                write_wat_source(val, out);
            }
            out.push('}');
        }
    }
}

/// `(:wat::core::ast->children <ast>)` — arc 251 Stone 251.5a-iii (the bridge).
///
/// The AST↔walkable bridge: decompose a `:wat::WatAST` node into a
/// `(Vector :- [:wat::WatAST])` of its children — the SAME walkable shape `:wat::core::forms`
/// produces (`Value::Vec` of `wat__WatAST`), so the existing `first`/`rest`/`map`
/// collection vocab applies for free. A List/Vector/Set node yields its items; a Map
/// yields its keys and values interleaved; a leaf (Symbol/Keyword/literal) yields the
/// empty vector. This is what lets a recursive transform written IN WAT walk a form
/// read by `read-string` — the tendon between the read/write spine and the fixer's will.
pub fn eval_ast_children(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast->children";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            }));
        }
    };
    let wrap = |n: &WatAST| Value::wat__WatAST(std::sync::Arc::new(n.clone()));
    let children: Vec<Value> = match ast {
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items.iter().map(&wrap).collect()
        }
        WatAST::Map(pairs, _) => pairs
            .iter()
            .flat_map(|(k, val)| [wrap(k), wrap(val)])
            .collect(),
        _ => Vec::new(),
    };
    Ok(crate::value::TrackedValue::new(
        Value::Vec(std::sync::Arc::new(children)),
        crate::value::Provenance::RuntimeBuilt {
            producer: OP,
            call_span: list_span.clone(),
        },
    ))
}

/// `(:wat::core::with-children <template> <children>)` — arc 251 Stone 251.5a-iv.
///
/// The kind-preserving REBUILD: a NEW AST node of the SAME KIND as `template`,
/// carrying `children` (a `(Vector :- [:wat::WatAST])`, as `ast->children` yields) as its
/// children. The inverse of `ast->children` GIVEN the decomposed node — the template
/// restores the kind `ast->children` collapses. Faithful round-trip:
/// `(with-children n (ast->children n)) = n` for every node kind. This lets a
/// recursive `fix-source` rebuild a walked tree without corrupting a Vector binder
/// into a List call.
pub fn eval_with_children(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::with-children";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(), expected: 2, got: args.len(),
        }));
    }
    let template_v = eval(&args[0], env, sym)?.value_owned();
    let children_v = eval(&args[1], env, sym)?.value_owned();
    // template must be a forms-value
    let template: &WatAST = match &template_v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    };
    // children must be a Vec of forms-values; unwrap each to WatAST
    let child_vals: &Vec<Value> = match &children_v {
        Value::Vec(v) => v.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: "(:wat::core::Vector :- [:wat::WatAST])",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    };
    let mut kids: Vec<WatAST> = Vec::with_capacity(child_vals.len());
    for cv in child_vals.iter() {
        match cv {
            Value::wat__WatAST(a) => kids.push(a.as_ref().clone()),
            other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(), expected: ":wat::WatAST (child)",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            })),
        }
    }
    // rebuild the SAME KIND as the template, preserving its span
    let rebuilt: WatAST = match template {
        WatAST::List(_, span) => WatAST::List(kids, span.clone()),
        WatAST::Vector(_, span) => WatAST::Vector(kids, span.clone()),
        WatAST::Set(_, span) => WatAST::Set(kids, span.clone()),
        WatAST::Map(_, span) => {
            if !kids.len().is_multiple_of(2) {
                return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("Map rebuild needs an even child count (k/v interleaved); got {}", kids.len()),
                }));
            }
            let mut pairs = Vec::with_capacity(kids.len() / 2);
            let mut it = kids.into_iter();
            while let (Some(k), Some(v)) = (it.next(), it.next()) {
                pairs.push((k, v));
            }
            WatAST::Map(pairs, span.clone())
        }
        // a leaf has no children — rebuilding it with children is a contract violation
        leaf => {
            if !kids.is_empty() {
                return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("leaf node has no children; cannot rebuild with {} child(ren)", kids.len()),
                }));
            }
            leaf.clone()
        }
    };
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(rebuilt)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::ast-kind <node>)` — arc 251 Stone 251.5a-v. Total kind discriminant.
pub fn eval_ast_kind(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast-kind";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let kind = match ast {
        WatAST::IntLit(..) => "int",
        WatAST::FloatLit(..) => "float",
        WatAST::RationalLit(..) => "rational",
        WatAST::BigIntLit(..) => "bigint",
        WatAST::CharLit(..) => "char",
        WatAST::BoolLit(..) => "bool",
        WatAST::StringLit(..) => "string",
        WatAST::NilLit(..) => "nil",
        WatAST::Keyword(..) => "keyword",
        WatAST::Symbol(..) => "symbol",
        WatAST::List(..) => "list",
        WatAST::Vector(..) => "vector",
        WatAST::Set(..) => "set",
        WatAST::Map(..) => "map",
    };
    Ok(crate::value::TrackedValue::new(
        Value::String(std::sync::Arc::new(kind.to_string())),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::ast-name <node>)` — arc 251 Stone 251.5a-v. Verbatim token text of a Symbol/Keyword.
pub fn eval_ast_name(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast-name";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let name: String = match ast {
        WatAST::Symbol(ident, _) => ident.as_str().to_string(),
        WatAST::Keyword(s, _) => s.clone(),
        // Arc 279 — format macro needs the string content from a StringLit node.
        // "Does a macro need it?" → YES: format extracts the template text at expand time.
        // ast-name on a StringLit returns the string VALUE (unquoted content), matching the
        // natural meaning of "name" for literal nodes alongside Symbol/Keyword.
        WatAST::StringLit(s, _) => s.clone(),
        _ => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "ast-name requires a Symbol, Keyword, or StringLit node".to_string(),
        })),
    };
    Ok(crate::value::TrackedValue::new(
        Value::String(std::sync::Arc::new(name)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::ast-span <node>)` — Stone 251.5 / Slice 4.2a. Source START location of any node.
/// Returns `{:line N :col N}` as a `HashMap<keyword, i64>`. `:file` is intentionally excluded because
/// the single-file codemod consumer holds its own path and threads it directly — NOT because file is
/// unknowable. (It IS threadable: `parse_all_with_file` accepts a real label and `read-file` holds the
/// path; today `read-string` discards it at the seam, stamping every node `"<read-string>"`.) A
/// multi-file consumer would want a `Span` record (product type, file/line/col typed) plus a path-aware
/// read — see `DESIGN-STONE-251.5-4.2-comment-faithful-drive.md` "CORRECTION (2026-06-11)".
/// (The earlier "mixed-value map un-typeable" rationale here was wrong; corrected per builder catch.)
pub fn eval_ast_span(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast-span";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let span = ast.span();
    #[allow(clippy::mutable_key_type)]
    let mut map: std::collections::HashMap<Value, Value> = std::collections::HashMap::new();
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":line".to_string())),
        Value::i64(span.line),
    );
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":col".to_string())),
        Value::i64(span.col),
    );
    Ok(crate::value::TrackedValue::new(
        Value::wat__std__HashMap(std::sync::Arc::new(map)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::ast-end-span <node>)` — Arc 281. Source END location of any node.
/// Returns `{:line N :col N}` as a `HashMap<keyword, i64>` — the position ONE char past the
/// node's last char (for `(a b c)`, col 8, just after the `)`).
/// Symmetric twin of `eval_ast_span`; reads `span.end_line`/`span.end_col`.
pub fn eval_ast_end_span(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast-end-span";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let span = ast.span();
    #[allow(clippy::mutable_key_type)]
    let mut map: std::collections::HashMap<Value, Value> = std::collections::HashMap::new();
    let end_line = span.end.as_ref().map(|p| p.line).unwrap_or(span.line);
    let end_col  = span.end.as_ref().map(|p| p.col).unwrap_or(span.col);
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":line".to_string())),
        Value::i64(end_line),
    );
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":col".to_string())),
        Value::i64(end_col),
    );
    Ok(crate::value::TrackedValue::new(
        Value::wat__std__HashMap(std::sync::Arc::new(map)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::symbol-node <string>)` — arc 251 Stone 251.5a-v. Construct a bare Symbol node.
pub fn eval_symbol_node(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::symbol-node";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    // STONE-the-last-mint — the third door. Genuinely unwalled until now; harmless only
    // because the checker's surface arm keys on Keyword rather than Symbol. Same predicate,
    // same message as `keyword-node` / `keyword/from-string` above.
    if crate::runtime::angle_type_head_in_name(&s) {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: crate::runtime::angle_minted_name_reason(&s),
        }));
    }
    let node = WatAST::Symbol(Identifier::bare(s), crate::rust_caller_span!());
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::fresh-symbol <base>)` — arc 274 Stone 274.1. Construct a capture-proof Symbol node.
///
/// Like `symbol-node` but adds a fresh unique `ScopeId` to the `Identifier` via `add_scope(fresh_scope())`.
/// The resulting symbol's `env_key` is `"<base>\u{1}<scope-id>"` — distinct from any user symbol of the
/// same base name (which carries an empty scope set, key = bare name). A computing macro uses the SAME
/// returned value for both the binder and all references, so they share the unique scope and resolve to
/// each other — never to a user variable. Capture is structurally impossible by construction.
pub fn eval_fresh_symbol(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::fresh-symbol";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let ident = Identifier::bare(s).add_scope(crate::scope::fresh_scope());
    let node = WatAST::Symbol(ident, crate::rust_caller_span!());
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::keyword-node <string>)` — arc 251 Stone 251.5a-v. Construct a Keyword node (arg must start with ':').
pub fn eval_keyword_node(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::keyword-node";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    if crate::runtime::angle_type_head_in_name(&s) {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: crate::runtime::angle_minted_name_reason(&s),
        }));
    }
    if !s.starts_with(':') {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("keyword-node requires a ':'-prefixed string; got {s:?}"),
        }));
    }
    let node = WatAST::Keyword(s, crate::rust_caller_span!());
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::keyword/to-symbol <keyword-node>)` — arc 251 head role-inversion. Convert a
/// wat rust-scheme call-head Keyword node into a faithful-Clojure Symbol node via
/// [`wat_keyword_to_clojure_symbol`]. The kind CHANGE (Keyword → Symbol) is the inversion: a
/// call head is a symbol in Clojure, never a keyword. Errors if the keyword is not a
/// convertible head/reference (bare data keyword or namespace-prefix marker).
pub fn eval_keyword_to_symbol(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::keyword/to-symbol";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let kw: String = match &v {
        Value::wat__WatAST(a) => match a.as_ref() {
            WatAST::Keyword(s, _) => s.clone(),
            _ => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "keyword/to-symbol requires a Keyword node".to_string(),
            })),
        },
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    let symbol_name = wat_keyword_to_clojure_symbol(&kw).ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "not a convertible call-head/reference keyword (bare data keyword or namespace-prefix marker): {kw:?}"
            ),
        }))?;
    let node = WatAST::Symbol(Identifier::bare(symbol_name), crate::rust_caller_span!());
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// Arc 109 Stone ②-i — head-spelling mode for [`type_expr_to_clojure_form`]. Threaded through
/// the 4-way ladder (the `Path` arm and the `Parametric` head arm) so the SAME renderer serves
/// both spellings — a second copy is how the two spellings drift apart (DESIGN-STONE-2, "do not
/// fork it").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeFormHeadMode {
    /// `wat.type/Vector` — today's faithful-Clojure spelling. The later Clojure-flip's target;
    /// this is the mode `eval_keyword_to_type_form` has always used and keeps using.
    Clojure,
    /// `:wat::core::Vector` — the rust-ish keyword spelling, rendered as a `WatAST::Keyword`
    /// (a node-KIND difference from `Clojure`'s `Symbol`, not just a different string). Arc 109
    /// step ② needs this: the corpus codemod moves `Head<…>` → `(Head [types])` while keeping
    /// the `:wat::core::` head spelling — the Clojure head-flip is separate and later.
    Colon,
}

/// Arc 251 type-position rendering — convert a closed [`crate::types::TypeExpr`] into a
/// faithful WatAST node for the type FORM surface. `mode` selects the head spelling (Room 2,
/// arc 109 Stone ②-i); the bracketing below is unconditional in BOTH modes (Room 1), and, since
/// Stone ②-i-b, prefixed with the `:-` parameterization operator (Room 1 again): `(Head :- [a
/// b])`, never a bare `(Head [a b])` — `:-` declares "the thing on the left is parameterized by
/// the thing on the right", the same relation the arg-spec and ret-type arrows already carry.
///
/// 4-way discriminator (Path; Parametric head mirrors it) — `mode` only changes cases 1-3;
/// case 4 (type-var) has no colon form (a type-var was never namespace-qualified in ANY
/// spelling — `T`/`K`/`V` are lexically-scoped identifiers, not keywords) and stays a bare
/// symbol in both modes:
/// 1. core FQDN (`wat::core::X`) — Clojure: `wat.type/X` Symbol. Colon: `:wat::core::X` Keyword.
/// 2. bare legacy primitive (`:i64`, `:String`, ...) — Clojure: `wat.type/X` Symbol. Colon: the
///    primitive's own core FQDN, `:wat::core::X` Keyword.
/// 3. user/library type (has `::`, not core) — Clojure: namespace-preserving Symbol
///    (`wat.holon/HolonAST`). Colon: the FQDN Keyword unchanged (`:wat::holon::HolonAST`).
/// 4. type-var (no `::`, not a primitive) — bare symbol (`T`, `K`, `V`), both modes.
/// - `Parametric{head,args}`: same 4-way ladder on head; args bracket into ONE `WatAST::Vector`
///   behind the `:-` operator, in the list's third position (`(Head :- [a b])`, both modes, Room
///   1); recurse on each arg with the SAME `mode`.
/// - `Fn{args,ret}`: `Vector([…args, Keyword(":->"), ret])` — UNCHANGED by this stone; args/ret
///   recurse with `mode`, but the `:->` keyword and the Vector shape are fixed either way.
/// - `Tuple(items)`: `List([head, Keyword(":-"), Vector(…rendered-items)])`. As of Stone
///   ②-i-b the head is NO LONGER out of scope for `mode` — it now runs the SAME 4-way ladder
///   Parametric's head does, except only case 1 can ever fire (Tuple's implicit head is always
///   the core FQDN `wat::core::Tuple`; there is no user/bare/type-var Tuple head): Clojure ->
///   `wat.type/Tuple` Symbol, Colon -> `:wat::core::Tuple` Keyword. Items still recurse with
///   `mode`. Args bracket UNCONDITIONALLY behind `:-`, at every arity including zero — the
///   empty tuple renders `(Head :- [])`, a first-class rung of the arity ladder (NOT the
///   `(wat.type/Tuple)` bare-head spelling this stone retires), and distinct from `nil` (wat's
///   unit) — wat's `()` empty tuple is not `nil`.
/// - `Var`: synthetic — NEVER produced by parsing source (the `TypeExpr` doc guarantees it).
///
/// Fallible: the two unmodeled shapes (a malformed trailing-`::` path, and a bare/higher-kinded
/// Parametric head like `(Stream …)`/`(T …)`) return a clean `Err` — NEVER a panic, in EITHER
/// mode. This renderer backs the runtime verbs `keyword/to-type-form` (Clojure) and
/// `keyword/to-type-form-colon` (Colon) AND the corpus drive; both error shapes are reachable
/// (`parse_type_expr` accepts `:foo::` and `:Stream<i64>`), so a panic would crash wat / the drive.
pub(crate) fn type_expr_to_clojure_form(t: &crate::types::TypeExpr, mode: TypeFormHeadMode) -> Result<WatAST, String> {
    use crate::types::TypeExpr;
    let unk = crate::rust_caller_span!();
    Ok(match t {
        TypeExpr::Path(s) => {
            // 4-way ladder: core FQDN > bare primitive > user type (::) > type-var.
            let body = s.strip_prefix(':').unwrap_or(s);
            if let Some(tail) = body.strip_prefix("wat::core::") {
                // Case 1: core FQDN -> flat wat.type/ namespace (Clojure) or :wat::core:: keyword (Colon).
                match mode {
                    TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare(format!("wat.type/{tail}")), unk),
                    TypeFormHeadMode::Colon => WatAST::Keyword(format!(":wat::core::{tail}"), unk),
                }
            } else if let Some((_bare, fqdn)) = crate::check::BARE_PRIMITIVES.iter().find(|(bare, _)| *bare == format!(":{body}").as_str()) {
                // Case 2: bare legacy primitive (:i64, :String, ...) -> wat.type/{body} (Clojure)
                // or the primitive's own core FQDN keyword, `fqdn` (Colon; already colon-prefixed).
                match mode {
                    TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare(format!("wat.type/{body}")), unk),
                    TypeFormHeadMode::Colon => WatAST::Keyword((*fqdn).to_string(), unk),
                }
            } else if body.contains("::") {
                // Case 3: user/library type -> namespace-preserving Symbol (Clojure) or the FQDN
                // keyword unchanged (Colon). `wat_keyword_to_clojure_symbol` is also the ONLY
                // validation this shape gets (malformed trailing `::`/empty segment) — reuse it
                // in BOTH modes so a malformed path errors identically either way.
                let clojure_sym = wat_keyword_to_clojure_symbol(&format!(":{body}")).ok_or_else(|| {
                    format!("cannot render type `:{body}` to a faithful form (malformed namespaced path — trailing `::` or empty segment)")
                })?;
                match mode {
                    TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare(clojure_sym), unk),
                    TypeFormHeadMode::Colon => WatAST::Keyword(format!(":{body}"), unk),
                }
            } else {
                // Case 4: type-var -- stays as a bare symbol (T, K, V, ...), SAME in both modes:
                // a type-var was never colon-spelled in any surface.
                WatAST::Symbol(Identifier::bare(body.to_string()), unk)
            }
        }
        TypeExpr::Parametric { head, args } => {
            // head is stored WITHOUT a leading colon (e.g. "wat::core::Vector").
            // 4-way ladder mirrors Path.
            let head_node: WatAST = if let Some(tail) = head.strip_prefix("wat::core::") {
                // Case 1: core FQDN -> flat wat.type/ namespace (Clojure) or :wat::core:: keyword (Colon).
                match mode {
                    TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare(format!("wat.type/{tail}")), unk.clone()),
                    TypeFormHeadMode::Colon => WatAST::Keyword(format!(":wat::core::{tail}"), unk.clone()),
                }
            } else if let Some((_bare, fqdn)) = crate::check::BARE_CONTAINER_HEADS.iter().find(|(bare, _)| *bare == head.as_str()) {
                // Case 2: bare container head (Option, Vec, ...) -> canonical FQDN. Clojure uses
                // the FQDN's last segment (Vec -> wat::core::Vector rename, so the FQDN tail, not
                // `head`); Colon uses the whole FQDN as a keyword.
                match mode {
                    TypeFormHeadMode::Clojure => {
                        let tail = wat_reader::identifier::leaf(fqdn);
                        WatAST::Symbol(Identifier::bare(format!("wat.type/{tail}")), unk.clone())
                    }
                    TypeFormHeadMode::Colon => WatAST::Keyword(format!(":{fqdn}"), unk.clone()),
                }
            } else if head.contains("::") {
                // Case 3: user/library type -> namespace-preserving Symbol (Clojure) or the FQDN
                // keyword unchanged (Colon). Validate via wat_keyword_to_clojure_symbol in BOTH
                // modes, same reasoning as the Path arm's case 3.
                let fqdn = crate::types::parametric_head_fqdn(head);
                let clojure_sym = wat_keyword_to_clojure_symbol(&fqdn).ok_or_else(|| {
                    format!("cannot render parametric head `:{head}` (malformed namespaced path)")
                })?;
                match mode {
                    TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare(clojure_sym), unk.clone()),
                    TypeFormHeadMode::Colon => WatAST::Keyword(fqdn, unk.clone()),
                }
            } else {
                // Case 4: bare/higher-kinded head (`(Stream …)`, `(T …)`) — not in the model.
                // Clean error (the source should use the FQDN form), never panic — same in both modes.
                return Err(format!(
                    "cannot render parametric type with bare head `{head}` — not a core container and not FQDN; \
                     use the fully-qualified type name (bare/higher-kinded heads are unsupported)"
                ));
            };
            // Room 1 — args bracket into ONE WatAST::Vector, UNCONDITIONALLY, in both modes,
            // behind the `:-` parameterization operator (Stone ②-i-b): `(Head :- [a b])`. Was
            // flat-spliced `(Head a b)`, then bracketed bare `(Head [a b])` (Stone ②-i); `:-` is
            // a Keyword and is mode-independent, so it is identical in both modes.
            let mut arg_items: Vec<WatAST> = Vec::with_capacity(args.len());
            for a in args {
                arg_items.push(type_expr_to_clojure_form(a, mode)?);
            }
            WatAST::List(
                vec![head_node, WatAST::Keyword(":-".into(), unk.clone()), WatAST::Vector(arg_items, unk.clone())],
                unk,
            )
        }
        TypeExpr::Fn { args, ret } => {
            let mut items: Vec<WatAST> = Vec::with_capacity(args.len() + 2);
            for a in args {
                items.push(type_expr_to_clojure_form(a, mode)?);
            }
            items.push(WatAST::Keyword(":->".into(), unk.clone()));
            items.push(type_expr_to_clojure_form(ret, mode)?);
            WatAST::Vector(items, unk)
        }
        TypeExpr::Tuple(items) => {
            // Arc 109 Stone ②-i-b — the Tuple arm now honours `mode`, exactly what Parametric
            // got at Stone ②-i (see the fn doc): the head runs the same 4-way ladder as
            // Parametric's head, and only case 1 can ever fire — Tuple's implicit head is
            // always the core FQDN `wat::core::Tuple`. Args bracket UNCONDITIONALLY behind the
            // `:-` operator, at EVERY arity including zero — `(Head :- [a b …])`, never a bare
            // head. The empty tuple renders `(Head :- [])`, a first-class rung of the arity
            // ladder, distinct from `nil` (wat's unit; wat's `()` empty tuple is not `nil`).
            let head_node: WatAST = match mode {
                TypeFormHeadMode::Clojure => WatAST::Symbol(Identifier::bare("wat.type/Tuple".to_string()), unk.clone()),
                TypeFormHeadMode::Colon => WatAST::Keyword(":wat::core::Tuple".into(), unk.clone()),
            };
            let mut arg_items: Vec<WatAST> = Vec::with_capacity(items.len());
            for it in items {
                arg_items.push(type_expr_to_clojure_form(it, mode)?);
            }
            WatAST::List(
                vec![head_node, WatAST::Keyword(":-".into(), unk.clone()), WatAST::Vector(arg_items, unk.clone())],
                unk,
            )
        }
        // Var is synthetic — NEVER produced by parsing source (the TypeExpr doc guarantees it),
        // so this verb (which only ever sees parsed-from-source types) cannot reach it.
        TypeExpr::Var(_) => unreachable!("type_expr_to_clojure_form: Var is never produced by parsing source"),
    })
}

/// Shared body for [`eval_keyword_to_type_form`] and [`eval_keyword_to_type_form_colon`] —
/// arc 109 Stone ②-i, Room 3. Parameterized on `OP` (for error messages/provenance) and `mode`
/// (Room 2's head-spelling mode); do NOT fork this into two copies, per DESIGN-STONE-2.
fn eval_keyword_to_type_form_impl(
    op: &'static str,
    mode: TypeFormHeadMode,
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    let v = require_one_arg(op, args, env, sym, list_span)?;
    let kw: String = match &v {
        Value::wat__WatAST(a) => match a.as_ref() {
            WatAST::Keyword(s, _) => s.clone(),
            _ => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "keyword/to-type-form requires a Keyword node".to_string(),
            })),
        },
        other => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) })),
    };
    // Arc 109 Stone ②-i-b — the NON-canonicalizing preserving parse: keeps the source
    // spelling (`:wat::core::nil` stays `Path`, not collapsed to `Tuple(vec![])`) so the
    // renderer below can round-trip what was actually written instead of a type that
    // canonicalization already erased. See `parse_type_expr_preserving_with_span`'s doc.
    let te = crate::types::parse_type_expr_preserving_with_span(&kw, list_span).map_err(|e| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("type-keyword parse failed: {:?}", e.kind()),
        }))?;
    let node = type_expr_to_clojure_form(&te, mode).map_err(|reason| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm { head: op.into(), reason }))?;
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: op, call_span: list_span.clone() },
    ))
}

/// `(:wat::core::keyword/to-type-form <keyword-node>)` — arc 251 type-position rendering.
/// Convert an old rust-scheme TYPE keyword (`:wat::core::Vector<wat::core::i64>`) into the
/// faithful-Clojure type FORM (`(wat.type/Vector [wat.type/i64])`). Parses the keyword string
/// via the EXISTING type parser ([`crate::types::parse_type_expr_with_span`] → `TypeExpr`),
/// then renders the closed `TypeExpr` enum via [`type_expr_to_clojure_form`] in
/// [`TypeFormHeadMode::Clojure`] — UNCHANGED head spelling; only the bracketing moved (Room 1,
/// arc 109 Stone ②-i).
pub fn eval_keyword_to_type_form(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::keyword/to-type-form";
    eval_keyword_to_type_form_impl(OP, TypeFormHeadMode::Clojure, args, list_span, env, sym)
}

/// `(:wat::core::keyword/to-type-form-colon <keyword-node>)` — arc 109 Stone ②-i, Room 3 sibling
/// of [`eval_keyword_to_type_form`]. Same parse + render pipeline, [`TypeFormHeadMode::Colon`]:
/// `:wat::core::Vector<wat::core::i64>` → `(:wat::core::Vector [:wat::core::i64])` — a colon-
/// quoted Keyword head, bracketed args, the rust-ish spelling step ②'s corpus codemod needs
/// (the Clojure head-flip is separate and later).
pub fn eval_keyword_to_type_form_colon(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::keyword/to-type-form-colon";
    eval_keyword_to_type_form_impl(OP, TypeFormHeadMode::Colon, args, list_span, env, sym)
}

/// Errors surfaced by [`read_edn`] / [`edn_to_value`] when an EDN
/// document fails to coerce to a runtime [`Value`]. Pattern A (Stone
/// 243.7d): span at the outer struct level; variant data in
/// `EdnReadErrorKind`. Substrate-consumer crates (e.g.
/// `wat-telemetry-sqlite`'s row reify) match against these to surface
/// diagnostic messages.
#[derive(Debug)]
pub struct EdnReadError {
    pub span: Span,
    pub kind: EdnReadErrorKind,
}

/// Variant data for [`EdnReadError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug)]
pub enum EdnReadErrorKind {
    /// `#ns/Name {body}` whose `ns/Name` doesn't resolve to any
    /// declared struct or enum in the type registry. `body_shape`
    /// reports what was found ("Map", "Vector", "Nil", etc.) so
    /// the caller can disambiguate "the type registry doesn't
    /// have this name" from "the body shape doesn't match the
    /// declared kind."
    UnknownTag {
        ns: String,
        name: String,
        body_shape: &'static str,
    },
    /// A substrate-reserved tag the bridge doesn't currently
    /// understand. `#inst` is handled by the underlying
    /// `wat_edn` parser; everything else lands here.
    UnsupportedTag(String),
    /// No type registry was attached. The bridge needs the
    /// registry to interpret `#ns/Name` tags; without one,
    /// any tagged value fails. Pass `None` only when you know
    /// the EDN document contains no tagged values.
    NoTypeRegistry,
    /// `#ns/Name {map}` referenced a key that isn't a declared
    /// field of the named struct.
    UnknownStructField {
        type_path: String,
        key: String,
    },
    /// `#ns/Name [body]` or `#ns/Name nil` referenced a variant
    /// name that isn't declared on the named enum.
    EnumVariantNotFound {
        type_path: String,
        variant: String,
    },
    /// Catch-all — the EDN value couldn't be coerced to a wat
    /// Value for the listed structural reason (e.g. unsupported
    /// `wat_edn::Value` variant like Symbol or BigInt, or a
    /// surface-level parse error wrapped here).
    Other(String),
    // ── RETIRED arc 293.W.2a ──────────────────────────────────────────────────
    // StructOnWire { class: String } — deleted by arc 293.W.2d.
    // The §7 struct-on-wire runtime backstop is superseded by the compile-time
    // purity wall at wire-peer PRODUCERS (peer-pair', connect',
    // accept', program-self-peer'). A struct can no longer be typed into a wire
    // peer at CHECK time, so the runtime decode door has no reachable struct case
    // to reject. The untyped pprintln path is an out-of-scope trust-boundary
    // concern (user validates inputs — the compiler wall is the primary defense).
}

impl std::fmt::Display for EdnReadErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTag { ns, name, body_shape } => write!(
                f,
                "unknown tag #{ns}/{name} (body shape: {body_shape}); \
                 no matching struct or enum in the type registry"
            ),
            Self::UnsupportedTag(t) => {
                write!(f, "unsupported substrate tag #{t}")
            }
            Self::NoTypeRegistry => write!(
                f,
                "no type registry attached to SymbolTable; arc 085 capability missing"
            ),
            Self::UnknownStructField { type_path, key } => write!(
                f,
                "struct {type_path} has no field named {key}"
            ),
            Self::EnumVariantNotFound { type_path, variant } => write!(
                f,
                "enum {type_path} has no variant named {variant}"
            ),
            Self::Other(s) => {
                write!(f, "{s}")
            }
            // StructOnWire retired by arc 293.W.2d (compile-time wall supersedes runtime backstop).
        }
    }
}

impl std::fmt::Display for EdnReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
    }
}

/// Walk a `wat_edn::OwnedValue` into a wat runtime `Value`. The
/// inverse of [`value_to_edn_with`]; tags drive struct/enum
/// reconstruction via the type registry.
/// Parse an EDN string into a runtime [`Value`], using `types`
/// to interpret `#ns/Name` tags as struct or enum references.
/// Higher-level convenience over [`edn_to_value`] — does the
/// `wat_edn::parse_owned` step too, so callers that have a raw
/// `&str` get one call instead of two.
///
/// Pass `Some(registry)` for any EDN containing tagged structs
/// or enums; pass `None` only for primitive-only documents (the
/// bridge will return [`EdnReadError::NoTypeRegistry`] on the
/// first tagged value otherwise).
///
/// Public arc-093: arc-093's row-reify path in
/// `wat-telemetry-sqlite` calls this per column to convert each
/// `:wat::edn::Tagged` / `:wat::edn::NoTag` TEXT cell back into
/// the typed runtime [`Value`] the cursor's `step!` shim returns
/// to wat callers.
/// `ctx` (arc 294.g) is the ambient `EncodingCtx` a HolonRecord-nature class needs to DERIVE
/// its hologram on decode (the wire no longer carries it). Pass `None` only when you know the
/// registry the caller passed can't resolve to a `Nature::HolonRecord` class (e.g. `types` is
/// itself `None`, or the registry is a fixed capability-codec type) — a decode that reaches a
/// HolonRecord class with `ctx = None` errors loudly rather than derive a wrong-dimension index.
pub fn read_edn(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    // General (untrusted) decode — capability tags are REFUSED (allow_caps = false).
    read_edn_caps(s, types, false, ctx)
}

/// Arc 272 6a-i — the capability-aware decode worker. PRIVATE by design: when `allow_caps` is true,
/// portable capability tags reconstruct into live capabilities. There is intentionally NO public
/// way to pass `allow_caps = true` — the only caller that may is [`decode_trusted_wire`], the single
/// audited door. This is what makes "mint a capability from an untrusted decode" UNREPRESENTABLE:
/// general code holds no flag to flip and no fn to reach (extirpare top rung; ocap transfer-only).
fn read_edn_caps(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
    allow_caps: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_edn operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })?;
    // Trusted peer wire is a KNOWN-types channel — never foreign-mode.
    edn_to_value_caps(&edn, types, allow_caps, false, ctx)
}

// ─── EDN value-framing (pipe wire protocol) ─────────────────────────────────

/// Status of a string with respect to forming a complete EDN value.
///
/// Used by `read_framed_edn` to drive line accumulation: read physical
/// lines until `edn_frame_status` reports `Complete`, surface `Malformed`
/// immediately.
#[derive(Debug, PartialEq)]
pub enum EdnFrameStatus {
    /// The buffer contains exactly one complete EDN value — no more input needed.
    Complete,
    /// The buffer is cut off mid-value — read another line and retry.
    Incomplete,
    /// The buffer contains a genuine syntax error. `String` is the message.
    Malformed(String),
}

/// Classify a string with respect to forming a complete EDN value.
///
/// Calls `wat_edn::parse_owned(s)` directly (NOT through `read_edn`, which
/// stringifies the error kind — we need the structured kind to call
/// `is_incomplete()`):
/// - `Ok(_)` → `Complete` (the parser accepted and required EOF)
/// - `Err(e)` where `e.is_incomplete()` → `Incomplete`
/// - `Err(e)` otherwise → `Malformed(format!("{e}"))`
pub fn edn_frame_status(s: &str) -> EdnFrameStatus {
    match wat_edn::parse_owned(s) {
        Ok(_) => EdnFrameStatus::Complete,
        Err(e) if e.is_incomplete() => EdnFrameStatus::Incomplete,
        Err(e) => EdnFrameStatus::Malformed(format!("{e}")),
    }
}

/// Maximum accumulated bytes for a single EDN value-frame.
///
/// Bounds the in-memory accumulation inside `read_framed_edn` (and the
/// inline accumulator in `channel/transfer.rs`) for a single logical value.
/// A sender that never closes a value (`{` forever, or a broken/malicious
/// peer) would otherwise grow the buffer until OOM; this cap terminates
/// accumulation and surfaces `FramedRead::TooLarge` instead.
pub const DEFAULT_MAX_FRAME_BYTES: usize = 512 * 1024; // 512 KiB

/// Outcome of [`next_complete_frame`] — the ONE frame-finder that both
/// the blocking-pull path (`read_framed_edn`) and the non-blocking comms
/// path (`take_frame` in `comms/process.rs`) route through.
#[derive(Debug)]
pub enum FrameScan {
    /// A complete EDN value ends at byte offset `end` (exclusive), i.e.
    /// `buf[..end]` contains the value + its terminating `'\n'`. The
    /// caller consumes `buf[..end]` and keeps `buf[end..]` as the residual.
    Frame(usize),
    /// No complete EDN value is present yet — the caller should append
    /// more bytes and retry.
    Incomplete,
    /// The accumulated buffer exceeded `max_bytes` before a complete EDN
    /// value was found. The `usize` is `buf.len()` at the point of rejection.
    TooLarge(usize),
    /// The accumulated buffer contains a wire-level error that no
    /// `from_wire` impl could decode — currently only non-UTF-8 bytes. The
    /// `String` is the error message. NOTE: a genuine EDN *syntax* error is
    /// NOT reported here; it returns `Frame(end)` (rule 2 of
    /// [`next_complete_frame`]) so the decode step surfaces it — `String`
    /// wire content is raw passthrough, not EDN.
    Malformed(String),
}

/// The ONE frame-finder. Pure — no I/O.
///
/// Scans `buf` line-by-line (splitting on `'\n'`); for each prefix up to
/// and including that newline, calls [`edn_frame_status`] on the prefix
/// WITHOUT the trailing `'\n'`. The FIRST prefix that is NOT `Incomplete`
/// (i.e. `Complete` or `Malformed`) → `FrameScan::Frame(end)` where `end`
/// is the byte index just past the `'\n'` (i.e. `buf[..end]` is the
/// complete frame including its terminator).
///
/// Rules applied in order as newlines are consumed:
/// 1. `Complete` prefix → size-check `end` against `max_bytes`; if `end >
///    max_bytes` → `TooLarge(end)` (semantics B: max MESSAGE size, not merely
///    un-terminated accumulation); otherwise `Frame(end)`.
/// 2. `Malformed` prefix → same size-check as (1): `TooLarge(end)` if too
///    large; otherwise `Frame(end)`. The decode step (`from_wire` / `read_edn`)
///    handles the content error. This covers non-EDN wire formats
///    (`String::from_wire` raw passthrough) and genuinely malformed multi-line
///    EDN alike; the content error surfaces at decode time, not frame-finding
///    time.
/// 3. `Incomplete` prefix → advance past this newline; the EDN value is not
///    yet complete (e.g. `{` without closing `}`). Continue scanning.
/// 4. After exhausting all `'\n'` positions with no non-Incomplete prefix:
///    if `buf.len() > max_bytes` → `TooLarge(buf.len())`; otherwise
///    `Incomplete` (need more bytes).
/// 5. Non-UTF-8 bytes → `Malformed("non-UTF-8 bytes in frame")`. This is the
///    ONLY path that returns `FrameScan::Malformed`; it is a wire-level error
///    that cannot be decoded by any `from_wire` impl.
///
/// Both `read_framed_edn` (blocking `WatReader` path) and `take_frame`
/// (`comms/process.rs` io_uring path) route through this single function
/// so framing logic cannot diverge.
pub fn next_complete_frame(buf: &[u8], max_bytes: usize) -> FrameScan {
    let mut search_start = 0usize;
    loop {
        // Find the next '\n' from the current scan position.
        match buf[search_start..].iter().position(|&b| b == b'\n') {
            None => {
                // No more newlines — no complete frame in the buffer.
                if buf.len() > max_bytes {
                    return FrameScan::TooLarge(buf.len());
                }
                return FrameScan::Incomplete;
            }
            Some(rel) => {
                let newline_idx = search_start + rel;
                let end = newline_idx + 1; // byte past the '\n'
                // The candidate value is buf[0..newline_idx] (stripped of '\n').
                let prefix = &buf[..newline_idx];
                let prefix_str = match std::str::from_utf8(prefix) {
                    Ok(s) => s,
                    Err(_) => {
                        // Wire-level encoding error — can't decode regardless.
                        return FrameScan::Malformed(
                            "non-UTF-8 bytes in frame".to_string(),
                        )
                    }
                };
                match edn_frame_status(prefix_str) {
                    // Complete EDN value — size-check then clean frame boundary.
                    // Semantics B: reject complete frames that exceed the budget
                    // (a complete but oversized message is still too large).
                    EdnFrameStatus::Complete | EdnFrameStatus::Malformed(_) => {
                        if end > max_bytes {
                            return FrameScan::TooLarge(end);
                        }
                        return FrameScan::Frame(end);
                    }
                    EdnFrameStatus::Incomplete => {
                        // This prefix is cut off mid-value (e.g. `{` without `}`);
                        // advance past this newline and accumulate more.
                        search_start = end;
                        continue;
                    }
                }
            }
        }
    }
}

/// The result of a single `read_framed_edn` call.
#[derive(Debug)]
pub enum FramedRead {
    /// A complete EDN frame (the accumulated buffer, `\n`-terminated).
    Frame(String),
    /// Clean EOF before any bytes were read — the writer closed normally.
    Eof,
    /// EOF arrived mid-frame — the writer died while sending a value.
    Truncated(String),
    /// The accumulated buffer contains a wire-level encoding error —
    /// currently only non-UTF-8 bytes (from `FrameScan::Malformed`).
    /// EDN syntax errors in otherwise UTF-8 content reach the caller
    /// as `Frame` and surface as decode errors at the `from_wire` step.
    Malformed(String),
    /// The accumulated buffer exceeded `max_bytes` without completing a
    /// value. The `usize` is the byte count at the point of rejection.
    /// Indicates a broken or malicious peer that never terminates its frame.
    TooLarge(usize),
    /// Arc 170 — a process-wide stop was requested while waiting on a line.
    /// NOT an `Eof` (the writer didn't close) and NOT an error (nothing is
    /// wrong with the stream) — its own outcome, carried up rather than
    /// erased by the `Ok(None) | Err(_)` wildcard this replaces. Mirrors
    /// `RecvOutcome::Shutdown` (`channel/transfer.rs`), the accepted shape
    /// for this same defect class one layer over.
    /// Named `Shutdown`, not `Stopped`: ruled by the arc-170 intueri cast —
    /// this is a Rust-internal type, and Rust's vocabulary for this fact is
    /// uniformly `shutdown` (`trigger_shutdown`, `RecvError::Shutdown`,
    /// `SHUTDOWN_BROADCAST_READ_FD`). Only the wat-visible siblings
    /// (`:wat::io::IOReader::ReadFrameOutcome::Stopped` and
    /// `:wat::kernel::ReadFrameOutcome::Stopped`) were renamed — see
    /// `src/io.rs`'s `eval_ioreader_read_frame`, the ONE site where this
    /// Rust variant crosses into the wat vocabulary.
    Shutdown,
}

/// What a single `next_line` call passed to [`read_framed_edn`] reports.
/// Distinct from a plain `Result<Option<String>, RuntimeError>` so a caller
/// doing OS-level poll-multiplexing (see `channel/transfer.rs`'s
/// `LineResult`, the exemplar this mirrors) can report "a stop was
/// requested" without it collapsing into `Eof` at the very first hop.
/// Rust-internal glue — not a wat-visible type, so not part of the
/// intueri naming cast for `Shutdown`/`Stopped`. Named `LineRead`, not
/// `NextLine` (arc 170 rename brief): it names the REQUEST's answer, not
/// the request itself, and `LineRead`/`FramedRead` pair exactly the way
/// `LineResult`/`FramedRead`'s Rust-side siblings already do.
pub enum LineRead {
    /// One physical line, without its trailing `\n`.
    Line(String),
    /// Clean EOF.
    Eof,
    /// A process-wide stop was requested before a line arrived.
    Shutdown,
}

/// Accumulate physical lines from `next_line` until the buffer forms a
/// complete EDN value, then return it as a `FramedRead::Frame`.
///
/// Each call to `next_line(span)` must return:
/// - `Ok(LineRead::Line(line))` — one line WITHOUT its trailing `\n`
/// - `Ok(LineRead::Eof)` — EOF
/// - `Ok(LineRead::Shutdown)` — a stop was requested; nothing is wrong with
///   the stream (arc 170 — carried through to `FramedRead::Shutdown`,
///   never collapsed into `Eof`)
/// - `Err(e)` — a read error (treated as EOF / disconnect — unchanged from
///   before arc 170; a genuine I/O error is not a stop request)
///
/// Internally accumulates bytes and routes through [`next_complete_frame`]
/// (the one frame-finder) so the framing logic is shared with the comms
/// io_uring path and cannot diverge.
///
/// Note: anti-smuggling (two concatenated values on one physical line) is
/// enforced at the DECODE step (`edn_to_value` / `from_wire`) rather than
/// at the frame-finding step — `next_complete_frame` treats a `Malformed`
/// prefix (including trailing-value EDN) as a terminal `Frame`. The ambient
/// channel path (`channel/transfer.rs`) enforces anti-smuggling at its own
/// `edn_frame_status` loop, independent of this function.
///
/// The `max_bytes` parameter caps total accumulated bytes before a
/// `FramedRead::TooLarge` is returned, defending against a peer that
/// sends an open-ended frame (`{` forever) that would otherwise exhaust
/// memory.
pub fn read_framed_edn<F>(mut next_line: F, span: Span, max_bytes: usize) -> Result<FramedRead, RuntimeError>
where
    F: FnMut(Span) -> Result<LineRead, RuntimeError>,
{
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match next_line(span.clone()) {
            Ok(LineRead::Line(line)) => {
                buf.extend_from_slice(line.as_bytes());
                buf.push(b'\n');
                match next_complete_frame(&buf, max_bytes) {
                    FrameScan::Frame(end) => {
                        // buf[..end] is the complete frame including trailing '\n'.
                        let s = String::from_utf8_lossy(&buf[..end]).into_owned();
                        return Ok(FramedRead::Frame(s));
                    }
                    FrameScan::Incomplete => continue,
                    FrameScan::TooLarge(n) => return Ok(FramedRead::TooLarge(n)),
                    FrameScan::Malformed(msg) => return Ok(FramedRead::Malformed(msg)),
                }
            }
            // Arc 170 — the stop request gets its OWN arm, matched explicitly
            // BEFORE it could ever reach a wildcard. This is the fix: the prior
            // shape here was `Ok(None) | Err(_)`, which is exactly the erasing
            // wildcard `kernel/peer.rs`'s `Thread::recv` was fixed to stop doing
            // for `RecvError::Shutdown` one layer over.
            Ok(LineRead::Shutdown) => return Ok(FramedRead::Shutdown),
            Ok(LineRead::Eof) | Err(_) => {
                if buf.is_empty() {
                    return Ok(FramedRead::Eof);
                } else {
                    return Ok(FramedRead::Truncated(
                        String::from_utf8_lossy(&buf).into_owned(),
                    ));
                }
            }
        }
    }
}

/// Bridge a parsed `wat_edn::OwnedValue` to a runtime [`Value`],
/// using `types` to interpret `#ns/Name` tags. Most consumers
/// want [`read_edn`] (parse + bridge in one call); reach for
/// this directly when you already have the parsed EDN tree (e.g.
/// when bridging multiple sub-expressions of one document).
// Stone 216.5b — suppress `mutable_key_type` for `HashSet<Value>`.
// `Value` contains `Arc`-wrapped types with interior mutability, triggering the lint.
// The opaque-handle variants with interior mutability are never inserted into the set
// (only EDN-representable primitive values are bridged here). False positive.
/// `ctx` (arc 294.g) — see [`read_edn`]'s doc: required to derive a HolonRecord's hologram
/// on decode; `None` is safe only when `types` can't resolve to a `Nature::HolonRecord` class.
pub fn edn_to_value(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    // Arc 272 6a-i gating — the GENERAL decode path REFUSES portable-capability
    // tags. Object-capability rule: a capability is obtained only by being handed it on a trusted
    // channel, NEVER forged from parsed data. The trusted peer wire opts in via the `_caps` worker
    // with `allow_caps = true` (see `read_edn_caps` / `edn_string_to_value_trusted`).
    edn_to_value_caps(edn, types, false, false, ctx)
}

/// Arc 278 Stone A — the DATA-MODE decode entry (`:wat::edn::read-foreign`).
///
/// Identical to [`edn_to_value`] except an UNKNOWN tag reconstructs a
/// self-describing dynamic value (`Value::ForeignRecord` for a map body,
/// `Value::ForeignVariant` for a vector body) instead of raising `UnknownTag`.
/// `allow_caps` is kept `false` — foreign decode is untrusted parsed data, so
/// capability tags stay refused (the strict floor is not the only guard here).
/// STRICT [`edn_to_value`] is UNCHANGED (unknown tag still errors — R41).
pub fn edn_to_value_foreign(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    edn_to_value_caps(edn, types, /*allow_caps*/ false, /*foreign*/ true, ctx)
}

#[allow(clippy::mutable_key_type)]
fn edn_to_value_caps(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    match edn {
        Edn::Nil => Ok(Value::Unit),
        Edn::Bool(b) => Ok(Value::bool(*b)),
        Edn::Integer(n) => Ok(Value::i64(*n)),
        Edn::Float(x) => Ok(Value::f64(*x)),
        // Arc 300 stone B — rational literal, representation only (no
        // arithmetic; Stone A already normalized so `denom() >= 2`).
        Edn::Rational(r) => Ok(Value::wat__core__Rational(Box::new((**r).clone()))),
        Edn::String(s) => Ok(Value::String(Arc::new(s.to_string()))),
        // Arc 220 slice 2: EDN character literal `\c` → typed `:wat::core::char`.
        // Previously folded to String (lossy). Now preserved as a typed char
        // so round-trips through EDN are lossless. BMP guaranteed by wat-edn parser.
        // Stone 242.1: renamed from :wat::core::Char to :wat::core::char.
        Edn::Char(c) => Ok(Value::wat__core__Char(*c)),
        Edn::Keyword(k) => {
            let s = match k.namespace() {
                Some(ns) => format!(":{}::{}", ns.replace('.', "::"), k.name()),
                None => format!(":{}", k.name()),
            };
            Ok(Value::wat__core__keyword(Arc::new(s)))
        }
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        Edn::Symbol(_) => Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other("EDN Symbol — wat has no symbol value type".into()) }),
        Edn::BigInt(_) | Edn::BigDec(_) => Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other("EDN BigInt / BigDecimal — wat numeric tower is i64 + f64 only".into()) }),
        // Arc 220 Stone 220.4 — EDN list `(...)` → `Value::wat__core__List` (preserves
        // the parens-vs-brackets distinction for faithful Clojure round-trips).
        // Previously both List and Vector collapsed to Vec (lossy).
        Edn::List(items) => {
            let walked: std::collections::LinkedList<Value> = items
                .iter()
                .map(|x| edn_to_value_caps(x, types, allow_caps, foreign, ctx))
                .collect::<Result<_, _>>()?;
            Ok(Value::wat__core__List(Arc::new(walked)))
        }
        Edn::Vector(items) => {
            let walked: Vec<Value> = items
                .iter()
                .map(|x| edn_to_value_caps(x, types, allow_caps, foreign, ctx))
                .collect::<Result<_, _>>()?;
            Ok(Value::Vec(Arc::new(walked)))
        }
        Edn::Map(entries) => {
            // Generic HashMap — the no-tag map case. Walk keys + values.
            // Stone 216.5c — native HashMap<Value, Value>; hashmap_key crutch removed.
            // Guard: reject non-hashable keys (opaque handles) with a clear error.
            #[allow(clippy::mutable_key_type)]
            let mut backing: std::collections::HashMap<Value, Value> =
                std::collections::HashMap::with_capacity(entries.len());
            for (k, v) in entries {
                let k_val = edn_to_value_caps(k, types, allow_caps, foreign, ctx)?;
                let v_val = edn_to_value_caps(v, types, allow_caps, foreign, ctx)?;
                if !crate::runtime::value_is_key_hashable(&k_val) {
                    return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other(format!("non-hashable map key: {}", k_val.type_name())) });
                }
                backing.insert(k_val, v_val);
            }
            Ok(Value::wat__std__HashMap(Arc::new(backing)))
        }
        Edn::Set(items) => {
            // Stone 216.5b — native HashSet<Value> insert; hashmap_key crutch removed.
            // Value: Hash + Eq (Stone 216.5a) makes this work natively.
            let mut backing = std::collections::HashSet::with_capacity(items.len());
            for x in items {
                let v_val = edn_to_value_caps(x, types, allow_caps, foreign, ctx)?;
                backing.insert(v_val);
            }
            Ok(Value::wat__std__HashSet(Arc::new(backing)))
        }
        Edn::Inst(t) => Ok(Value::Instant(*t)),
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        // Arc 207 slice 2: `#uuid "..."` EDN reader literal → typed `:wat::core::Uuid`.
        // `uuid::Uuid` is `Copy`; mirrors `Edn::Inst(t) → Value::Instant(*t)` pattern.
        Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u)),
        Edn::Tagged(tag, body) => tagged_to_value(tag, body, types, allow_caps, foreign, ctx),
    }
}

// ─── EDN → typed-T coercion (arc 170 slice 1f-ι) ───────────────────

/// Error returned by [`edn_to_typed_value`] when the parsed EDN tree
/// doesn't match the caller's declared target type.
///
/// Mirrors the diagnostic shape of [`EdnReadError`] (the untyped
/// counterpart) plus the load-bearing `expected` field carrying the
/// declared `TypeExpr` (rendered via
/// [`crate::check::format_type`]). `path` accumulates field /
/// element indices as the coercion recurses, so the surfaced
/// `RuntimeError::EdnCoerceMismatch` names the exact mismatch
/// site (e.g., `".name"`, `".[2]"`, `".some.[0].field"`).
#[derive(Debug)]
pub struct EdnCoerceError {
    pub expected: String,
    pub got: String,
    pub path: String,
}

impl EdnCoerceError {
    fn at(mut self, segment: &str) -> Self {
        // Prepend to build the path from the leaf back up.
        self.path = format!("{}{}", segment, self.path);
        self
    }
}

/// Shape names for EDN values used in diagnostic surfaces.
fn edn_shape_name(edn: &wat_edn::OwnedValue) -> &'static str {
    use wat_edn::Value as Edn;
    match edn {
        Edn::Nil => "Nil",
        Edn::Bool(_) => "Bool",
        Edn::Integer(_) => "Integer",
        Edn::Float(_) => "Float",
        Edn::Rational(_) => "Rational",
        Edn::String(_) => "String",
        Edn::Char(_) => "Char",
        Edn::Keyword(_) => "Keyword",
        Edn::Symbol(_) => "Symbol",
        Edn::List(_) => "List",
        Edn::Vector(_) => "Vector",
        Edn::Map(_) => "Map",
        Edn::Set(_) => "Set",
        Edn::Tagged(_, _) => "Tagged",
        Edn::Inst(_) => "Inst",
        Edn::Uuid(_) => "Uuid",
        Edn::BigInt(_) => "BigInt",
        Edn::BigDec(_) => "BigDec",
    }
}

fn mismatch(target: &crate::types::TypeExpr, edn: &wat_edn::OwnedValue) -> EdnCoerceError {
    EdnCoerceError {
        expected: crate::check::format_type(target),
        got: edn_shape_name(edn).into(),
        path: String::new(),
    }
}

/// Arc 278 Stone A.0 — extract the single field of a one-arity vector-bodied
/// variant (`#tag [v]`) on the typed-coerce path. Enforces vector body + arity-1
/// so a malformed body fails loudly.
fn coerce_variant_single<'a>(
    target: &crate::types::TypeExpr,
    edn: &wat_edn::OwnedValue,
    body: &'a wat_edn::OwnedValue,
) -> Result<&'a wat_edn::OwnedValue, EdnCoerceError> {
    use wat_edn::Value as Edn;
    match body {
        Edn::Vector(items) | Edn::List(items) if items.len() == 1 => Ok(&items[0]),
        _ => Err(mismatch(target, edn)),
    }
}

/// Coerce an already-parsed EDN tree to a runtime [`Value`] whose
/// type matches the caller's declared `target` annotation.
///
/// Arc 170 slice 1f-ι — the load-bearing piece of the EDN-only
/// `(:wat::kernel::readln)` contract.
/// `(:wat::kernel::println v)` emits canonical EDN via
/// [`value_to_edn_with`]; this function is its asymmetric inverse —
/// asymmetric because the caller declares `T`, so the coercion can
/// disambiguate shapes that EDN itself doesn't (`nil` → `:None` vs
/// `Value::Unit`, vector → tuple vs `Vec`, map → struct, etc.).
///
/// Recursive coercion rules (table):
///
/// | Target | EDN form expected | Result |
/// |---|---|---|
/// | `:wat::core::i64` | `Integer` | `Value::i64(n)` |
/// | `:wat::core::f64` | `Float` OR `Integer` (widening) | `Value::f64(f)` |
/// | `:wat::core::String` | `String` | `Value::String(s.into())` |
/// | `:wat::core::bool` | `Bool` | `Value::Bool(b)` |
/// | `:wat::core::nil` / `:()` | `Nil` | `Value::Unit` |
/// | `:wat::core::keyword` | `Keyword` | `Value::wat__core__keyword(...)` |
/// | `:(A,B,...)` (tuple) | `Vector` of len N | recurse per element |
/// | `:wat::core::Vector<T>` | `Vector` | recurse on each element |
/// | `:wat::core::Option<T>` | `Tagged #wat.core.Option/{None,Some}` | arc 298.1 |
/// | `:wat::core::Result<T,E>` | `Tagged #wat.core.Result/{Ok,Err}` | recurse on payload |
/// | user `Struct` | `Tagged #ns/Name {map}` | recurse per field |
/// | user `Enum` (Unit variant) | `Tagged #ns/Variant nil` | enum variant |
/// | user `Enum` (Tagged variant) | `Tagged #ns/Variant [items]` | recurse per field |
/// | `:wat::holon::HolonAST` | any | call [`edn_derive_holon`] (arc 294.j — one collapsed reader) |
///
/// On mismatch the returned [`EdnCoerceError`] carries the declared
/// type's rendered form, the EDN shape that arrived, and the path
/// to the offending sub-field. Callers wrap into
/// `RuntimeError::EdnCoerceMismatch`.
pub fn edn_to_typed_value(
    target: &crate::types::TypeExpr,
    edn: &wat_edn::OwnedValue,
    sym: &crate::runtime::SymbolTable,
) -> Result<Value, EdnCoerceError> {
    let types = sym.types().map(|a| a.as_ref());
    let ctx = sym.encoding_ctx().map(|a| a.as_ref());
    edn_to_typed_value_inner(target, edn, types, ctx)
}

fn edn_to_typed_value_inner(
    target: &crate::types::TypeExpr,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnCoerceError> {
    use crate::types::TypeExpr;
    use wat_edn::Value as Edn;
    // Resolve user-declared typealiases / newtypes to the underlying
    // form so coercion logic operates against canonical types. Aliases
    // collapse transparently; newtypes coerce against their inner
    // declared shape (the wat-side wrapper is invisible at the EDN
    // layer).
    if let TypeExpr::Path(p) = target {
        if let Some(env) = types {
            if let Some(def) = env.get(p) {
                match def {
                    crate::types::TypeDef::Alias(a) => {
                        return edn_to_typed_value_inner(&a.expr, edn, types, ctx);
                    }
                    crate::types::TypeDef::Newtype(n) => {
                        return edn_to_typed_value_inner(&n.inner, edn, types, ctx);
                    }
                    _ => {}
                }
            }
        }
    }
    match target {
        // ── Path-form: primitive scalars + user struct / enum (by name) ──
        TypeExpr::Path(p) => match p.as_str() {
            ":wat::core::i64" => match edn {
                Edn::Integer(n) => Ok(Value::i64(*n)),
                other => Err(mismatch(target, other)),
            },
            ":wat::core::f64" => match edn {
                Edn::Float(x) => Ok(Value::f64(*x)),
                // Widening: Integer fits a Float request.
                Edn::Integer(n) => Ok(Value::f64(*n as f64)),
                other => Err(mismatch(target, other)),
            },
            ":wat::core::String" => match edn {
                Edn::String(s) => Ok(Value::String(Arc::new(s.to_string()))),
                other => Err(mismatch(target, other)),
            },
            ":wat::core::bool" => match edn {
                Edn::Bool(b) => Ok(Value::bool(*b)),
                other => Err(mismatch(target, other)),
            },
            ":wat::core::nil" => match edn {
                Edn::Nil => Ok(Value::Unit),
                other => Err(mismatch(target, other)),
            },
            ":wat::core::keyword" => match edn {
                Edn::Keyword(k) => {
                    let s = match k.namespace() {
                        Some(ns) => format!(":{}::{}", ns.replace('.', "::"), k.name()),
                        None => format!(":{}", k.name()),
                    };
                    Ok(Value::wat__core__keyword(Arc::new(s)))
                }
                other => Err(mismatch(target, other)),
            },
            ":wat::core::u8" => match edn {
                Edn::Integer(n) => Ok(Value::u8(*n as u8)),
                other => Err(mismatch(target, other)),
            },
            // Arc 207 slice 4 (latent gap from slice 2): `#uuid "..."` EDN → typed `:Uuid`.
            // `edn_to_value` (untyped path) already handled `Edn::Uuid`; this arm
            // covers the typed path (`readln -> :T` where T contains `:wat::core::Uuid`
            // fields). Required for subprocess wire deserialization of UUID-typed fields.
            ":wat::core::Uuid" => match edn {
                Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u)),
                other => Err(mismatch(target, other)),
            },
            // Arc 220 slice 2: EDN character literal `\c` → typed `:char`.
            // Typed path mirrors `:wat::core::Uuid` above (latent gap pattern).
            // Stone 242.1 — renamed from :wat::core::Char to :wat::core::char
            // (scalar types lowercase per Doctrine 2).
            ":wat::core::char" => match edn {
                Edn::Char(c) => Ok(Value::wat__core__Char(*c)),
                other => Err(mismatch(target, other)),
            },
            // Arc 300 stone B — rational literal typed-coerce path, mirrors
            // the `:wat::core::Uuid` / `:wat::core::char` latent-gap pattern.
            // Stone C1 lowercased the surface (Doctrine 2: scalar types lowercase).
            ":wat::core::rational" => match edn {
                Edn::Rational(r) => Ok(Value::wat__core__Rational(Box::new((**r).clone()))),
                other => Err(mismatch(target, other)),
            },
            // Universal top (arc 278 R7): UP is free — ANY EDN value IS a
            // `:wat::core::Value`. Decode structurally via the untyped bridge
            // (no concrete-type coercion), so a heterogeneous value (e.g. the
            // `metadata-of` map) reads back. This makes `Value` SYMMETRIC: it is
            // `EdnRepresentable` (write side) and now an EDN coerce target (read
            // side) — closing the write-but-not-read asymmetry. `edn_to_value`
            // honours `types` so `#ns/Variant` enum tags rebuild as `Value::Enum`.
            ":wat::core::Value" => edn_to_value(edn, types, ctx).map_err(|e| EdnCoerceError {
                expected: ":wat::core::Value".into(),
                got: format!("{e}"),
                path: String::new(),
            }),
            // Arc 278 DESIGN-STONE-watast-is-the-wire — :wat::WatAST is the universal
            // top of the WIRE: every well-formed EDN value IS a WatAST (a form crosses
            // as a bare, untagged EDN list/vector/map/scalar — there is nothing to tag,
            // it already IS the EDN). A declared field type is a refinement applied
            // AFTER decode, never a gate on whether the value may cross; for WatAST
            // that refinement is the IDENTITY. Same move as R7's universal top of the
            // TYPE lattice (types.rs:5212, `:wat::core::Value`), one domain over.
            // `edn_to_watast` is the write side's own inverse (`watast_to_edn`), so
            // accepting here is literally undoing what the wire's own writer did.
            ":wat::WatAST" => crate::wat_edn_bridge::edn_to_watast(edn)
                .map(|ast| Value::wat__WatAST(Arc::new(ast)))
                .map_err(|e| EdnCoerceError {
                    expected: ":wat::WatAST".into(),
                    got: format!("{e}"),
                    path: String::new(),
                }),
            // Arc 294.j — ONE reader, no mode selector. The old tagged-vs-
            // natural branch existed to pick between two readers that only
            // differed in how they treated a BARE leaf; both died with the
            // tag family (`edn_derive_holon`, above) — there is nothing left
            // to select between, so both former branches call the same fn.
            ":wat::holon::HolonAST" => {
                edn_derive_holon(edn, types, ctx).map(Value::holon__HolonAST).map_err(|e| EdnCoerceError {
                    expected: ":wat::holon::HolonAST".into(),
                    got: format!("HolonAST decode error: {e}"),
                    path: String::new(),
                })
            }
            // ── Arc 278 the PARAMETRIC PROTOCOL — a type VARIABLE position is OPAQUE ──
            // `:K` / `:V` / `:T` is a declaration's lexically-scoped binder, not a registered
            // type, and it never resolves in the registry. Reached here it used to be an
            // instant `expected=:K got=String` — which is how the request-sanitization wall
            // refused every WELL-FORMED parametric request (`probes <- Vector<K>`), and how a
            // generated child-main's decode target `:S::Op<K,V>` refused every frame.
            //
            // wat ERASES type params at runtime. A server inside `serve<K,V>` does not know
            // what `K` was instantiated to and therefore has no basis on which to reject any
            // value in a `K`-typed position — so this arm accepts the value at its natural
            // shape (exactly `:wat::core::Value`) rather than pretending to a check it cannot
            // make. Every CONCRETE field around it is still walked and enforced to the leaf;
            // `K` itself is pinned STATICALLY, at the client's call site, where it is known.
            // (When a caller CAN name the instantiation — `(:wat::edn::validate v
            // (:S::Req :- [wat::core::String]))` — `substitute_type_params` puts the real argument
            // here first and this arm is never reached.)
            //
            // The var test is the substrate's own (`runtime::is_type_var_path`): bare, no
            // `::`, first alphabetic char uppercase — so no FQDN type can land here.
            _ if crate::runtime::is_type_var_path(p) => {
                edn_to_value(edn, types, ctx).map_err(|e| EdnCoerceError {
                    expected: crate::check::format_type(target),
                    got: e.to_string(),
                    path: String::new(),
                })
            }
            // User-declared name (struct / enum) — look up in the registry.
            _ => {
                let env = types.ok_or_else(|| EdnCoerceError {
                    expected: crate::check::format_type(target),
                    got: edn_shape_name(edn).into(),
                    path: String::new(),
                })?;
                match env.get(p) {
                    // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
                    // Arc 278 the REQUEST-MALFORMED wall (Stone 1) — RECORD-nature aggregates
                    // join Struct here. They were excluded by a `nature == Struct` guard, so
                    // every `defrecord` type was an instant `mismatch` on this path: a stale
                    // narrowing left by the 293.2b Struct/Record collapse, invisible because
                    // this walker has had ZERO production callers since arc 258 Stone 258.5b.
                    // Records are the ONLY thing a service request ever is (the S1 convention
                    // `<S>::<Op>Request` is a defrecord), so without this the sanitization wall
                    // would reject every well-formed request. `coerce_struct_path` builds with
                    // the DECLARED nature (below) so the reconstructed value does not lie.
                    // HolonRecord stays out: its wire form is the SAME class-tag-plus-fields
                    // map a plain record wears (294.g moved the discriminator from body shape
                    // to the registry; the hologram is a DERIVED index, never on the wire — see
                    // `reconstruct_holon_record`, which builds it via `build_holon_hologram`
                    // rather than reading one). This walker doesn't know how to derive it, so
                    // it keeps its existing fall-through and leaves that to the dedicated path.
                    Some(crate::types::TypeDef::Aggregate(a))
                        if matches!(
                            a.nature,
                            crate::types::Nature::Struct | crate::types::Nature::Record
                        ) =>
                    {
                        coerce_struct_path(p, a, edn, types, ctx, &[])
                    }
                    Some(crate::types::TypeDef::Enum(def)) => {
                        coerce_enum_path(p, def, edn, types, ctx, &[])
                    }
                    _ => Err(mismatch(target, edn)),
                }
            }
        },

        // ── Parametric: Vector<T>, Option<T>, Result<T,E>, ... ──
        TypeExpr::Parametric { head, args } => match head.as_str() {
            "wat::core::Vector" => {
                let elem_ty = args.first().ok_or_else(|| mismatch(target, edn))?;
                match edn {
                    Edn::Vector(items) | Edn::List(items) => {
                        let mut walked = Vec::with_capacity(items.len());
                        for (i, item) in items.iter().enumerate() {
                            let v = edn_to_typed_value_inner(elem_ty, item, types, ctx)
                                .map_err(|e| e.at(&format!(".[{}]", i)))?;
                            walked.push(v);
                        }
                        Ok(Value::Vec(Arc::new(walked)))
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            // Arc 220 Stone 220.4 — `:wat::core::List<T>` typed path.
            // Accepts EDN list `(...)` (and vector `[...]` for compatibility with
            // Clojure that pr-str's lists as parens but JSON consumers may emit brackets).
            "wat::core::List" => {
                let elem_ty = args.first().ok_or_else(|| mismatch(target, edn))?;
                match edn {
                    Edn::List(items) | Edn::Vector(items) => {
                        let mut walked = std::collections::LinkedList::new();
                        for (i, item) in items.iter().enumerate() {
                            let v = edn_to_typed_value_inner(elem_ty, item, types, ctx)
                                .map_err(|e| e.at(&format!(".[{}]", i)))?;
                            walked.push_back(v);
                        }
                        Ok(Value::wat__core__List(Arc::new(walked)))
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            "wat::core::Option" => {
                // Arc 278 Stone A.0 — Option wire form is VECTOR-bodied:
                // `#wat.core.Option/None []` / `#wat.core.Option/Some [inner]`.
                let inner_ty = args.first().ok_or_else(|| mismatch(target, edn))?;
                match edn {
                    Edn::Tagged(tag, body) if tag.namespace() == "wat.core.Option" => {
                        match tag.name() {
                            "None" => Ok(Value::Option(Arc::new(None))),
                            "Some" => {
                                let inner_edn = coerce_variant_single(target, edn, body)?;
                                let inner = edn_to_typed_value_inner(inner_ty, inner_edn, types, ctx)
                                    .map_err(|e| e.at(".some"))?;
                                Ok(Value::Option(Arc::new(Some(inner))))
                            }
                            _ => Err(mismatch(target, edn)),
                        }
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            "wat::core::Result" => {
                if args.len() != 2 {
                    return Err(mismatch(target, edn));
                }
                let ok_ty = &args[0];
                let err_ty = &args[1];
                match edn {
                    // Arc 278 Stone A.0 — Result is VECTOR-bodied:
                    // `#wat.core.Result/Ok [v]` / `#wat.core.Result/Err [e]`.
                    Edn::Tagged(tag, body) if tag.namespace() == "wat.core.Result" => {
                        match tag.name() {
                            "Ok" => {
                                let inner_edn = coerce_variant_single(target, edn, body)?;
                                let v = edn_to_typed_value_inner(ok_ty, inner_edn, types, ctx)
                                    .map_err(|e| e.at(".ok"))?;
                                Ok(Value::Result(Arc::new(Ok(v))))
                            }
                            "Err" => {
                                let inner_edn = coerce_variant_single(target, edn, body)?;
                                let v = edn_to_typed_value_inner(err_ty, inner_edn, types, ctx)
                                    .map_err(|e| e.at(".err"))?;
                                Ok(Value::Result(Arc::new(Err(v))))
                            }
                            _ => Err(mismatch(target, edn)),
                        }
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            // Arc 278 Stone 2 — the HashMap / HashSet arms. These were a `not yet supported`
            // STUB, and it was invisible for exactly as long as this walker had no callers
            // (arc 258 Stone 258.5b deleted the last one). Stone 2 turns the walker into the
            // request-sanitization wall on EVERY op of EVERY service, and the stub immediately
            // refused well-formed production traffic: `:wat::query::Store::PutRequest` carries
            // `rows <- Vector<StoredRow>` and `StoredRow.index-keys <- HashMap<String,IndexKey>`,
            // so every journal write came back `RequestMalformed` at
            // `["rows" "[0]" "index-keys"]`. That is a FALSE REFUSAL — a `HashMap<K,V>` is
            // perfectly validatable; the arm was simply never written. Writing it is the fix;
            // exempting the type would be an escape hatch by another name.
            //
            // The wire forms are `value_to_edn_with`'s own (this walk must invert exactly that
            // writer, since the guard feeds it): a std HashMap writes as a bare EDN map
            // `{k v …}`, a std HashSet as a bare EDN set `#{v …}`. Keys are walked against `K`
            // and values against `V`, so a mistyped key or value is still caught — the
            // recursion is the point, and the offending path names which one.
            "wat::core::HashMap" => {
                if args.len() != 2 {
                    return Err(mismatch(target, edn));
                }
                let (key_ty, val_ty) = (&args[0], &args[1]);
                match edn {
                    Edn::Map(pairs) => {
                        #[allow(clippy::mutable_key_type)]
                        let mut map: std::collections::HashMap<Value, Value> =
                            std::collections::HashMap::with_capacity(pairs.len());
                        for (k_edn, v_edn) in pairs.iter() {
                            // The key's own coordinate is its written form — a map has no
                            // positional index, so `.{<key>}` is the honest segment.
                            let seg = format!(".{{{}}}", wat_edn::write(k_edn));
                            let k = edn_to_typed_value_inner(key_ty, k_edn, types, ctx)
                                .map_err(|e| e.at(&seg))?;
                            let v = edn_to_typed_value_inner(val_ty, v_edn, types, ctx)
                                .map_err(|e| e.at(&seg))?;
                            map.insert(k, v);
                        }
                        Ok(Value::wat__std__HashMap(Arc::new(map)))
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            "wat::core::HashSet" => {
                let elem_ty = args.first().ok_or_else(|| mismatch(target, edn))?;
                match edn {
                    Edn::Set(items) => {
                        #[allow(clippy::mutable_key_type)]
                        let mut set: std::collections::HashSet<Value> =
                            std::collections::HashSet::with_capacity(items.len());
                        for item in items.iter() {
                            let seg = format!(".{{{}}}", wat_edn::write(item));
                            let v = edn_to_typed_value_inner(elem_ty, item, types, ctx)
                                .map_err(|e| e.at(&seg))?;
                            set.insert(v);
                        }
                        Ok(Value::wat__std__HashSet(Arc::new(set)))
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            _ => {
                // Parametric user type — strip `<...>` to look up the
                // base declaration; coerce against the base shape.
                let path = crate::types::parametric_head_fqdn(head);
                let env = types.ok_or_else(|| mismatch(target, edn))?;
                match env.get(&path) {
                    // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
                    // Arc 278 the parametric protocol — RECORD-nature aggregates join Struct
                    // here, exactly as they did on the `Path` arm above (Stone 1). The 293.2b
                    // Struct/Record collapse left this half narrowed: a `defrecord :S::Req<K>`
                    // named AT its instantiation (`(:S::Req :- [wat::core::String])`) was an instant
                    // mismatch, while the same record named bare walked fine. Records are what
                    // service requests ARE, so the two spellings of one type must agree.
                    Some(crate::types::TypeDef::Aggregate(a))
                        if matches!(
                            a.nature,
                            crate::types::Nature::Struct | crate::types::Nature::Record
                        ) =>
                    {
                        coerce_struct_path(&path, a, edn, types, ctx, args)
                    }
                    Some(crate::types::TypeDef::Enum(def)) => {
                        coerce_enum_path(&path, def, edn, types, ctx, args)
                    }
                    _ => Err(mismatch(target, edn)),
                }
            }
        },

        // ── Tuple: positional coercion against each element ──────
        TypeExpr::Tuple(elements) => {
            // `:()` (empty tuple = unit) accepts Nil.
            if elements.is_empty() {
                return match edn {
                    Edn::Nil => Ok(Value::Unit),
                    other => Err(mismatch(target, other)),
                };
            }
            match edn {
                Edn::Vector(items) | Edn::List(items) => {
                    if items.len() != elements.len() {
                        return Err(EdnCoerceError {
                            expected: crate::check::format_type(target),
                            got: format!("Vector(len={})", items.len()),
                            path: String::new(),
                        });
                    }
                    let mut walked = Vec::with_capacity(items.len());
                    for (i, (elem_ty, item)) in elements.iter().zip(items.iter()).enumerate() {
                        let v = edn_to_typed_value_inner(elem_ty, item, types, ctx)
                            .map_err(|e| e.at(&format!(".[{}]", i)))?;
                        walked.push(v);
                    }
                    Ok(Value::Tuple(Arc::new(walked)))
                }
                other => Err(mismatch(target, other)),
            }
        }

        // ── Fn type: not EDN-coercible by design ─────────────────
        TypeExpr::Fn { .. } => Err(EdnCoerceError {
            expected: crate::check::format_type(target),
            got: "(function types have no EDN encoding)".into(),
            path: String::new(),
        }),

        // ── Var: fresh unification variable shouldn't reach
        //   the coercion arm (the runtime always knows T concretely
        //   from the call-site `-> :T` annotation). Defensive arm.
        TypeExpr::Var(_) => Err(EdnCoerceError {
            expected: crate::check::format_type(target),
            got: "(unresolved type variable)".into(),
            path: String::new(),
        }),
    }
}

// Arc 293.2b — AggregateDef with kind==Struct replaces StructDef.
/// Arc 278 the PARAMETRIC PROTOCOL — substitute a generic declaration's own type PARAMS out of
/// a field / variant-field `TypeExpr` before the EDN walk descends into it.
///
/// The walk resolves every `TypeExpr::Path` against the type registry. A declaration's type
/// PARAMETER (`:K` inside `defrecord :S::GetRequest<K,V> [probes <- Vector<K>]`) is not a
/// registered type and never will be, so without this it resolved to nothing and the walk
/// reported `expected=:K got=String` — rejecting a WELL-FORMED request. That is how a parametric
/// message met the request-sanitization wall (`:wat::edn::validate`): every one of them refused.
///
/// Two cases, both honest:
///
/// * **Args known** (`args` non-empty — the caller named the instantiation, e.g.
///   `(:wat::edn::validate v (:S::GetRequest :- [wat::core::String wat::core::i64]))`): each param is
///   replaced by its actual argument and the walk enforces it EXACTLY, to the leaf.
///
/// * **Args unknown** (`args` empty — the caller named the bare generic): each param becomes
///   `:wat::core::Value`, which accepts any EDN shape. This is not a weakening, it is the
///   truth: the one production caller is the generated serve-loop guard inside `serve<K,V>`,
///   where `K` is a BINDER, not a type — wat erases type params at runtime, so the server
///   cannot know what `K` was instantiated to and has no basis to reject any value in a
///   `K`-typed position. Every CONCRETE field of the message is still enforced exactly; what a
///   generic service can check at its boundary is checked, and what it cannot is not pretended.
///   The static discipline pins `K` at the client's call site instead.
fn substitute_type_params(
    ty: &crate::types::TypeExpr,
    params: &[String],
    args: &[crate::types::TypeExpr],
) -> crate::types::TypeExpr {
    use crate::types::TypeExpr;
    match ty {
        TypeExpr::Path(p) => {
            let bare = p.trim_start_matches(':');
            match params.iter().position(|param| param == bare) {
                Some(i) => args
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| TypeExpr::Path(":wat::core::Value".into())),
                None => ty.clone(),
            }
        }
        TypeExpr::Parametric { head, args: inner } => TypeExpr::Parametric {
            head: head.clone(),
            args: inner
                .iter()
                .map(|a| substitute_type_params(a, params, args))
                .collect(),
        },
        TypeExpr::Tuple(elems) => TypeExpr::Tuple(
            elems
                .iter()
                .map(|e| substitute_type_params(e, params, args))
                .collect(),
        ),
        TypeExpr::Fn { args: fargs, ret } => TypeExpr::Fn {
            args: fargs
                .iter()
                .map(|a| substitute_type_params(a, params, args))
                .collect(),
            ret: Box::new(substitute_type_params(ret, params, args)),
        },
        other => other.clone(),
    }
}

fn coerce_struct_path(
    type_path: &str,
    def: &crate::types::AggregateDef,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
    type_args: &[crate::types::TypeExpr],
) -> Result<Value, EdnCoerceError> {
    use wat_edn::Value as Edn;
    // Tagged struct form — `#ns/Name {map}` matches the writer
    // produced by `value_to_edn_with`. Tagless `{map}` is rejected
    // (the writer never emits one for a struct target; tagless EDN
    // is a writer-side option via `write-notag`, not a reader-side
    // expectation).
    let body = match edn {
        Edn::Tagged(tag, body) => {
            let expected_tag = struct_tag_for(type_path);
            if tag.namespace() != expected_tag.0 || tag.name() != expected_tag.1 {
                return Err(EdnCoerceError {
                    expected: type_path.to_string(),
                    got: format!("Tagged({}/{})", tag.namespace(), tag.name()),
                    path: String::new(),
                });
            }
            body.as_ref()
        }
        other => {
            return Err(EdnCoerceError {
                expected: type_path.to_string(),
                got: edn_shape_name(other).into(),
                path: String::new(),
            });
        }
    };
    let entries = match body {
        Edn::Map(entries) => entries.as_slice(),
        other => {
            return Err(EdnCoerceError {
                expected: type_path.to_string(),
                got: format!("Tagged-body {}", edn_shape_name(other)),
                path: String::new(),
            });
        }
    };
    // Build keyword-name → value lookup.
    let mut by_key: std::collections::HashMap<String, &wat_edn::OwnedValue> =
        std::collections::HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        if let Edn::Keyword(kw) = k {
            by_key.insert(kw.name().to_string(), v);
        }
    }
    let mut fields: Vec<Value> = Vec::with_capacity(def.fields.len());
    for (fname, fty) in &def.fields {
        let fv = by_key.get(fname.as_str()).ok_or_else(|| EdnCoerceError {
            expected: type_path.to_string(),
            got: format!("missing field :{}", fname),
            path: String::new(),
        })?;
        // Arc 278 the parametric protocol — a generic declaration's params are not registry
        // types; substitute them out first (no params ⇒ the identity, allocation aside).
        let fty = substitute_type_params(fty, &def.type_params, type_args);
        let v = edn_to_typed_value_inner(&fty, fv, types, ctx)
            .map_err(|e| e.at(&format!(".{}", fname)))?;
        fields.push(v);
    }
    // Arc 278 the REQUEST-MALFORMED wall (Stone 1) — build with the DECLARED nature.
    // A record rebuilt as a Struct-nature aggregate would lie about its purity
    // (`Nature::is_pure`: Struct permits impurity, Record guarantees it) — the caller's
    // reconstructed value must carry the same nature `reconstruct_record` gives it.
    let class = type_path.trim_start_matches(':').to_string();
    Ok(Value::Aggregate(Arc::new(match def.nature {
        crate::types::Nature::Record => AggregateValue::record(class, def.names_arc(), Arc::new(fields)),
        _ => AggregateValue::struct_(class, def.names_arc(), fields),
    })))
}

fn coerce_enum_path(
    type_path: &str,
    def: &crate::types::EnumDef,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
    type_args: &[crate::types::TypeExpr],
) -> Result<Value, EdnCoerceError> {
    use wat_edn::Value as Edn;
    // User-enum tag is `<ns>/<Variant>` where `<ns>` derives from the
    // enum's qualified path plus its name (mirroring
    // `tag_from_type_path(format!("{}::{}", type_path, variant_name))`
    // in `value_to_edn_with`'s Enum arm).
    let (tag_ns, tag_name, body) = match edn {
        Edn::Tagged(tag, body) => (tag.namespace().to_string(), tag.name().to_string(), body.as_ref()),
        other => {
            return Err(EdnCoerceError {
                expected: type_path.to_string(),
                got: edn_shape_name(other).into(),
                path: String::new(),
            });
        }
    };
    // The expected enum-tag namespace mirrors the writer:
    // `tag_from_type_path(<enum_path>::<Variant>)` → ns = the enum
    // path's dotted form (typename included), name = variant name.
    let expected_ns = enum_variant_ns(type_path);
    if tag_ns != expected_ns {
        return Err(EdnCoerceError {
            expected: format!("{} (ns={})", type_path, expected_ns),
            got: format!("Tagged ns={}/{}", tag_ns, tag_name),
            path: String::new(),
        });
    }
    let variant = def.variants.iter().find(|v| match v {
        crate::types::EnumVariant::Unit(n) => n == &tag_name,
        crate::types::EnumVariant::Tagged { name, .. } => name == &tag_name,
    });
    let variant = variant.ok_or_else(|| EdnCoerceError {
        expected: type_path.to_string(),
        got: format!("unknown variant {}", tag_name),
        path: String::new(),
    })?;
    match variant {
        crate::types::EnumVariant::Unit(_) => {
            // Arc 278 Stone A.0 — unit variant body must be an EMPTY vector `[]`
            // (bare-nil bodies are retired; `nil` is the unit value only).
            match body {
                Edn::Vector(items) | Edn::List(items) if items.is_empty() => {
                    Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
                        type_path: type_path.to_string(),
                        variant_name: tag_name,
                        names: crate::runtime::no_field_names(),
                        fields: vec![],
                    })))
                }
                other => Err(EdnCoerceError {
                    expected: format!("{}::{} (unit → `[]`)", type_path, tag_name),
                    got: format!("Tagged-body {}", edn_shape_name(other)),
                    path: String::new(),
                }),
            }
        }
        crate::types::EnumVariant::Tagged { fields, .. } => {
            // Arc 278 Stone A.0 — tagged variant body must be a Vector matching arity.
            // Zero-field tagged variants serialize as `[]` (the writer emits an empty
            // vector for any `fields.is_empty()` variant); bare-nil bodies are retired.
            let items: &[wat_edn::OwnedValue] = match body {
                Edn::Vector(items) | Edn::List(items) => items.as_slice(),
                other => {
                    return Err(EdnCoerceError {
                        expected: format!("{}::{} (tagged)", type_path, tag_name),
                        got: format!("Tagged-body {}", edn_shape_name(other)),
                        path: String::new(),
                    });
                }
            };
            if items.len() != fields.len() {
                return Err(EdnCoerceError {
                    expected: format!(
                        "{}::{} (fields={})",
                        type_path,
                        tag_name,
                        fields.len()
                    ),
                    got: format!("Vector(len={})", items.len()),
                    path: String::new(),
                });
            }
            let mut walked = Vec::with_capacity(items.len());
            for (i, ((fname, fty), item)) in fields.iter().zip(items.iter()).enumerate() {
                // Arc 278 the parametric protocol — see `substitute_type_params`.
                let fty = substitute_type_params(fty, &def.type_params, type_args);
                let v = edn_to_typed_value_inner(&fty, item, types, ctx)
                    .map_err(|e| e.at(&format!(".{}", fname)))?;
                let _ = i; // path uses field name, index reserved for future
                walked.push(v);
            }
            // `def` holds the registry directly (we're already inside the `Tagged` arm this
            // very `def.variants` walk matched above, so `variant_names_arc` cannot miss).
            let names = def.variant_names_arc(&tag_name).unwrap_or_else(|| {
                panic!(
                    "edn_to_enum_value: `{type_path}::{tag_name}` matched Tagged above but \
                     variant_names_arc returned None — def and its own match arm disagree"
                )
            });
            Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
                type_path: type_path.to_string(),
                variant_name: tag_name,
                names,
                fields: walked,
            })))
        }
    }
}

/// Compute the EDN tag namespace + name for a struct's wire form.
/// Mirrors `tag_from_type_path` (file-local helper) but extracted
/// for the coercion side.
///
/// Arc 294.k — decode-side mirror of `tag_from_type_path`'s fabrication
/// fix. The old code fabricated a "no-home" placeholder namespace when
/// `type_path` had no `::`; that is the same silent lie, moved in the
/// same change. `panic!` for the same reason (see `tag_from_type_path`).
#[track_caller]
fn struct_tag_for(type_path: &str) -> (String, String) {
    let stripped = type_path.strip_prefix(':').unwrap_or(type_path);
    if !stripped.contains("::") {
        panic!(
            "struct_tag_for: type path {type_path:?} has no `::` namespace separator — no \
             derivable EDN home (fabricating a namespace would silently erase this type's \
             identity on the wire)"
        );
    }
    let ns = wat_reader::identifier::path(stripped).replace("::", ".");
    let name = wat_reader::identifier::leaf(stripped).to_string();
    (ns, name)
}

/// EDN tag namespace for an enum variant. The writer emits
/// `tag_from_type_path(format!("{type_path}::{variant_name}"))` →
/// namespace derived from the enum's full path + variant-name as the
/// tag's terminal segment. For the READ side, the namespace IS the
/// enum's dotted path (including the type name), and the tag name IS
/// the variant identifier.
fn enum_variant_ns(type_path: &str) -> String {
    let stripped = type_path.strip_prefix(':').unwrap_or(type_path);
    stripped.replace("::", ".")
}

// ─── Natural / tagless renderers ──────────────────────────────────

/// Natural-JSON walker. Same tagless transforms as the tagless EDN
/// renderer, plus:
/// - keywords downgrade to plain strings (no `:` prefix)
/// - Instants render as bare ISO-8601 strings (no `#inst` sentinel wrapper)
/// - enum unit variants render as plain strings
///
/// Designed for ingestion-tooling consumers (ELK / DataDog / CloudWatch Logs).
pub fn value_to_json_natural(
    v: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> OwnedValue {
    use std::borrow::Cow;
    match v {
        Value::Instant(t) => OwnedValue::String(Cow::Owned(
            t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        )),
        Value::Duration(ns) => OwnedValue::Integer(*ns),
        Value::wat__core__keyword(k) => {
            OwnedValue::String(Cow::Owned(strip_keyword_colon(k)))
        }
        Value::Aggregate(sv) if sv.nature == crate::types::Nature::Struct => {
            // Arc 296 G-2 — names are carried on the value; no registry lookup, no fallback.
            // Use String keys (plain strings — JSON-friendly).
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .names
                .iter()
                .zip(sv.fields.iter())
                .map(|(name, fv)| {
                    (
                        OwnedValue::String(Cow::Owned(name.clone())),
                        value_to_json_natural(fv, types),
                    )
                })
                .collect();
            OwnedValue::Map(entries)
        }
        Value::Enum(ev) => {
            // FQDN discriminator: `<dotted-ns>/<Variant>`. Bare variant
            // names lose information when the same name appears in
            // multiple enums.
            let qualified = format!(
                "{}/{}",
                type_path_to_namespace(&ev.type_path),
                ev.variant_name
            );
            if ev.fields.is_empty() {
                // Unit variant — emit the qualified string.
                OwnedValue::String(Cow::Owned(qualified))
            } else {
                // Arc 296 G′ — names are carried on the value; no registry lookup, no fallback.
                let mut entries: Vec<(OwnedValue, OwnedValue)> =
                    Vec::with_capacity(ev.fields.len() + 1);
                entries.push((
                    OwnedValue::String(Cow::Owned("_type".into())),
                    OwnedValue::String(Cow::Owned(qualified)),
                ));
                for (name, fv) in ev.names.iter().zip(ev.fields.iter()) {
                    entries.push((
                        OwnedValue::String(Cow::Owned(name.clone())),
                        value_to_json_natural(fv, types),
                    ));
                }
                OwnedValue::Map(entries)
            }
        }
        Value::Vec(xs) => OwnedValue::Vector(
            xs.iter().map(|x| value_to_json_natural(x, types)).collect(),
        ),
        Value::Tuple(xs) => OwnedValue::Vector(
            xs.iter().map(|x| value_to_json_natural(x, types)).collect(),
        ),
        // Stone 216.5c — iterate m.iter() for (k, v) directly (native HashMap<Value, Value>).
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.iter()
                .map(|(k, v)| {
                    let key_v = value_to_json_natural(k, types);
                    // JSON keys must be strings; coerce keywords/ints/etc.
                    let key_s = match &key_v {
                        OwnedValue::String(_) => key_v,
                        other => OwnedValue::String(Cow::Owned(wat_edn::write(other))),
                    };
                    (key_s, value_to_json_natural(v, types))
                })
                .collect(),
        ),
        // Arc 278 Stone A.0 — Option gets the uniform vector-bodied variant form in JSON too.
        Value::Option(opt) => match &**opt {
            None => OwnedValue::Tagged(
                Tag::ns("wat.core.Option", "None"),
                Box::new(OwnedValue::Vector(vec![])),
            ),
            Some(inner) => OwnedValue::Tagged(
                Tag::ns("wat.core.Option", "Some"),
                Box::new(OwnedValue::Vector(vec![value_to_json_natural(inner, types)])),
            ),
        },
        // Fallback: use the tagged walker. Result now falls through to
        // value_to_edn_with which emits #wat.core.Result/Ok|Err (arc 298.1).
        _ => value_to_edn_with(v, types),
    }
}

/// Convert a wat type path like `:demo::Event` to an EDN-friendly
/// namespace `demo.Event`. The leading `:` drops; `::` segments map
/// to `.` so EDN keyword/tag namespace conventions hold.
fn type_path_to_namespace(type_path: &str) -> String {
    type_path
        .strip_prefix(':')
        .unwrap_or(type_path)
        .replace("::", ".")
}

fn strip_keyword_colon(k: &str) -> String {
    // Wat keywords are stored with leading `:` and `::` separators.
    // For natural JSON we want a plain string.
    let stripped = k.strip_prefix(':').unwrap_or(k);
    // Convert `::` separators to `.` so JSON readers see a familiar
    // dotted-namespace form (e.g. `:wat::time::Instant` → `wat.time.Instant`).
    stripped.replace("::", ".")
}

/// Arc 278 Stone A.0 — read the single field of a one-arity vector-bodied
/// variant (`#tag [v]`). Enforces the vector body + exactly-one-item arity so a
/// malformed body (bare value, wrong arity) fails loudly (no-hidden-failures).
fn variant_single_field<F>(
    ns: &str,
    name: &str,
    body: &OwnedValue,
    decode: F,
) -> Result<Value, EdnReadError>
where
    F: FnOnce(&OwnedValue) -> Result<Value, EdnReadError>,
{
    use wat_edn::Value as Edn;
    match body {
        Edn::Vector(items) if items.len() == 1 => decode(&items[0]),
        _ => Err(EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::UnsupportedTag(format!(
                "{ns}/{name} body must be a one-element vector `[v]` (arc 278 A.0)"
            )),
        }),
    }
}

fn tagged_to_value(
    tag: &Tag,
    body: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    let ns = tag.namespace();
    let name = tag.name();

    // Arc 294.m — PORTABLE CAPABILITY tags. A capability now wears its real type home
    // (`#wat.kernel/Address`, not the retired `wat-edn.cap` marker namespace), so `wat.kernel`
    // no longer means "capability" by itself — it also hosts ordinary data (`Frame`,
    // `Location`). The refusal therefore asks the REGISTRY whether THIS EXACT type path is a
    // registered capability codec — arc 198's ruling, verbatim: "ask the registry whether the
    // key is live; never ask a string what it looks like." A registered capability tag IS
    // reconstructable — but ONLY off a TRUSTED channel (`allow_caps`, set by the peer wire). On
    // the general decode path (`:wat::edn::read`, config, any parsed data) it is REFUSED: an
    // object-capability is obtained by being handed it over a channel, never forged from data
    // (ocap unforgeability + transfer-only).
    let cap_type_path = ns_to_wat_path(ns, name);
    if crate::capability::is_capability_type_path(&cap_type_path) {
        if allow_caps {
            // Arc 272 6c.2 — record-based codecs (SocketAddressWire) need the type registry.
            // The trusted peer wire always provides types (decode_trusted_wire is always called
            // with sym.types()); None here is a programming error, surfaced as a decode failure.
            let t = types.ok_or_else(|| EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::NoTypeRegistry,
            })?;
            return crate::capability::decode_capability(&cap_type_path, body, t);
        }
        return Err(EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::UnsupportedTag(format!(
                "{ns}/{name} (capability tags reconstruct only off the trusted peer wire, never from parsed data)"
            )),
        });
    }

    // Arc 294.i — no explicit opaque-bucket refusal check needed anymore: every ex-opaque
    // handle now writes under its own per-type home (`#wat.kernel/Sender nil`, …) with a
    // bare-nil (or, for HandlePool, a String) body, and neither shape reconstructs by accident.
    // A nil body already refuses generically below (`Edn::Nil` arm — arc 278 A.0, "bare-nil body
    // — retired"); a non-nil, non-Map, non-Vector body (HandlePool's String; a Stream head that
    // happens not to be a Map/Vector) falls to the generic `UnknownTag` arm. Neither path
    // reconstructs a live opaque handle from data — the property this check used to assert by
    // name is now structural, not a namespace string match.
    //
    // Arc 294.j — the sibling holon-tag dispatch that used to sit here (`if ns == the-dead-tag
    // { edn_holon_tag_to_ast(...) }`) is GONE the same way, and for the same reason: the tag
    // family it dispatched on no longer exists on the write side (DESIGN-STONE-294.j), so a
    // value arriving here bearing the old namespace has NO arm — it falls through this whole
    // `if`-chain to the generic `UnknownTag` refusal below. Dead, not dormant (gate 5): there is
    // no name check doing the refusing, only the absence of a matching arm.
    // Arc 278 Stone A.0 — Option wire form is VECTOR-bodied: `#wat.core.Option/None []`
    // / `#wat.core.Option/Some [v]`. `None` accepts `[]`; `Some` reads the single field.
    if ns == "wat.core.Option" {
        return Ok(Value::Option(Arc::new(match name {
            "None" => None,
            "Some" => {
                let inner = variant_single_field(ns, name, body, |b| edn_to_value_caps(b, types, allow_caps, foreign, ctx))?;
                Some(inner)
            }
            // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
            _ => return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnsupportedTag(format!("{ns}/{name}")) }),
        })));
    }
    // Arc 278 Stone A.0 — Result is VECTOR-bodied: `#wat.core.Result/Ok [v]` / `.../Err [e]`.
    if ns == "wat.core.Result" {
        return Ok(Value::Result(Arc::new(match name {
            "Ok" => Ok(variant_single_field(ns, name, body, |b| edn_to_value_caps(b, types, allow_caps, foreign, ctx))?),
            "Err" => Err(variant_single_field(ns, name, body, |b| edn_to_value_caps(b, types, allow_caps, foreign, ctx))?),
            // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
            _ => return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnsupportedTag(format!("{ns}/{name}")) }),
        })));
    }

    // Arc-278-0a — `#wat.core/PersistentMap {…}` tagged literal → PersistentMap.
    // Round-trip identity: a tagged form reads back as wat__core__PersistentMap (never
    // conflated with std HashMap which reads from untagged `{…}`). Body must be a Map.
    if ns == "wat.core" && name == "PersistentMap" {
        use wat_edn::Value as Edn;
        let entries = match body {
            Edn::Map(e) => e,
            _ => return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnsupportedTag(
                "wat.core/PersistentMap body must be a map, got non-map".to_string()
            ) }),
        };
        let mut pairs: Vec<(Value, Value)> = Vec::new();
        for (k, v) in entries {
            let k_val = edn_to_value_caps(k, types, allow_caps, foreign, ctx)?;
            let v_val = edn_to_value_caps(v, types, allow_caps, foreign, ctx)?;
            if !crate::runtime::value_is_key_hashable(&k_val) {
                return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other(format!("non-hashable PersistentMap key: {}", k_val.type_name())) });
            }
            pairs.push((k_val, v_val));
        }
        return Ok(Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs)));
    }

    // Arc-278-0b — `#wat.core/PersistentVector [...]` tagged literal → PersistentVector.
    // Round-trip identity: a bare `[…]` reads back as std Vec; the tagged form reads back
    // as PersistentVector (distinct identity per the DESIGN contract). Body must be a Vector.
    if ns == "wat.core" && name == "PersistentVector" {
        use wat_edn::Value as Edn;
        let items = match body {
            Edn::Vector(xs) => xs,
            _ => return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnsupportedTag(
                "wat.core/PersistentVector body must be a vector, got non-vector".to_string()
            ) }),
        };
        let mut acc = Vec::with_capacity(items.len());
        for item in items {
            acc.push(edn_to_value_caps(item, types, allow_caps, foreign, ctx)?);
        }
        return Ok(Value::wat__core__PersistentVector(
            crate::value::pvec::PVec::from_vec(acc),
        ));
    }

    // Arc 294.j RELAND — `#wat.holon/Thermometer {…}` / `#wat.holon/SlotMarker {…}`, the two
    // encoding DIRECTIVES, reconstruct here too, NOT only through the narrow
    // `:wat::holon::HolonAST` typed-coercion arm (`edn_derive_holon`). MEASURED: the process
    // tier's message decode goes through THIS general (untyped) path — a service request
    // carrying a Thermometer nested in its fields hit `reconstruct_struct`'s registry lookup
    // (`:wat::holon::Thermometer` is a Rust intrinsic, never a registered struct) and refused
    // as `UnknownTag`, which is the far-side crash `wat-tests/service-cache-hologram.wat:121`
    // reproduced on the `on_process` tier. Registry-independent, same as
    // `wat.core.Option`/`wat.core.Result`/`wat.core/PersistentMap` immediately above — a
    // directive tag is recognised by NAME, not by a registered type.
    if ns == "wat.holon" {
        if let Some(holon) = decode_holon_directive_tag(name, body)? {
            return Ok(Value::holon__HolonAST(Arc::new(holon)));
        }
    }

    // Arc 294.j CORRECTION 2 — `#wat/holon <data>`, the DATA tag (distinct namespace/name
    // split from the `wat.holon/*` directives immediately above: namespace "wat", name
    // "holon", so it renders `#wat/holon` not `#wat.holon/holon`). Same MEASURED reason as
    // the directive check: the process tier's message decode goes through THIS general
    // (untyped) path, not only the narrow `:wat::holon::HolonAST` typed-coercion arm
    // (`edn_derive_holon`) — a struct field carrying a data holon needs to re-lift here too.
    if ns == "wat" && name == "holon" {
        let inner = edn_to_value_caps(body, types, allow_caps, foreign, ctx)?;
        return Ok(Value::holon__HolonAST(Arc::new(decode_holon_data_tag(inner)?)));
    }

    // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
    let types = types.ok_or(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::NoTypeRegistry })?;

    // Body shape disambiguates struct vs enum.
    // Arc 293.2b: For Map bodies, resolve the TypeDef to route:
    //   Aggregate(kind!=Struct) → reconstruct_record,
    //   Aggregate(kind==Struct) or unknown → reconstruct_struct (returns UnknownTag on miss).
    match body {
        // Arc 294.g — the discriminator moved from BODY SHAPE to the REGISTRY. Before this
        // stone a holon record had a distinct tagged-HolonAST body (the serialized hologram,
        // under the tag family 294.j later killed outright) so the decoder could tell it
        // apart from a base record's
        // `Edn::Map` body by shape alone. Now every record — Record or HolonRecord — wears
        // the SAME `Edn::Map` body (class tag + fields), so the only signal left is what the
        // type registry says `a.nature` is. `Nature::HolonRecord` routes to
        // `reconstruct_holon_record`, which builds fields from the map exactly as
        // `reconstruct_record` does and then DERIVES the hologram (`build_holon_hologram`,
        // the same fn `aggregate-new` calls at construction) — never sniffs the body, never
        // reads a marker key.
        Edn::Map(entries) => {
            let path = ns_to_wat_path(ns, name);
            match types.get(&path) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.nature == crate::types::Nature::HolonRecord => {
                    reconstruct_holon_record(ns, name, entries, types, allow_caps, foreign, ctx)
                }
                Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => {
                    reconstruct_record(ns, name, entries, types, allow_caps, foreign, ctx)
                }
                _ => reconstruct_struct(ns, name, entries, types, allow_caps, foreign, ctx),
            }
        }
        Edn::Vector(items) => reconstruct_enum_tagged(ns, name, items, types, allow_caps, foreign, ctx),
        // Arc 278 Stone A.0 — a bare-nil body is no longer a variant. Unit variants
        // are now `#tag []` (empty vector, handled above); `nil` is the unit value ONLY.
        // A generic `#tag nil` is malformed post-cutover → loud error (no-hidden-failures).
        Edn::Nil => Err(EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::UnsupportedTag(format!(
                "{ns}/{name} has a bare-nil body — retired (arc 278 A.0); unit variants are `#tag []`"
            )),
        }),
        other => {
            let shape = match other {
                Edn::Bool(_) => "bool",
                Edn::Integer(_) => "integer",
                Edn::Float(_) => "float",
                Edn::String(_) => "string",
                Edn::Keyword(_) => "keyword",
                Edn::Inst(_) => "inst",
                _ => "other",
            };
            // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
            Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: name.to_string(), body_shape: shape } })
        }
    }
}

pub(crate) fn ns_to_wat_path(ns: &str, name: &str) -> String {
    format!(":{}::{}", ns.replace('.', "::"), name)
}

/// Inverse of [`ns_to_wat_path`]: a wat rust-scheme call-head/reference KEYWORD
/// (`:wat::core::if`) → a faithful-Clojure SYMBOL string (`wat.core/if`). The `::`↔`.`/`/`
/// path grammar lives here, beside its forward — never re-encoded in wat (that would be a
/// duplicated-encoding braid; the 251.1 keystone pulled exactly that class out).
///
/// Returns `None` for keywords that are NOT call-heads/references — bare data keywords
/// (no `::`, e.g. `:else`) and namespace-prefix markers (trailing `::`, e.g. `:counter::`
/// used in `:restricted-to` whitelists) — so the caller decides whether to leave them.
///
/// The rule (derived + pressure-tested by an intueri cast, total over the corpus): strip the
/// leading `:`; split on `::`; the last segment is the NAME unless it is `Type/method` (a `/`
/// with a non-empty part before it), in which case `Type` folds into the namespace and
/// `method` is the name; join the namespace with `.`; result `namespace/name`. Division
/// (`:wat::core::/`) → `wat.core//` (the final segment IS `/`, so it is the name) — exactly
/// Clojure's `clojure.core//`.
pub(crate) fn wat_keyword_to_clojure_symbol(kw: &str) -> Option<String> {
    let body = kw.strip_prefix(':')?;
    // Not a head/reference: a bare data keyword (`:else`) or a namespace-prefix marker
    // (`:counter::`, trailing `::` — the final segment is empty).
    if !body.contains("::") || body.ends_with("::") {
        return None;
    }
    // `body` contains "::" and has no trailing "::", so there are ≥2 non-empty segments.
    let final_seg = wat_reader::identifier::leaf(body);
    let mut ns_parts: Vec<&str> = wat_reader::identifier::path(body).split("::").collect();
    let name: &str = if final_seg.contains('/') && !wat_reader::identifier::receiver(final_seg).is_empty() {
        // `Type/method` — fold `Type` into the namespace; the method is the name.
        ns_parts.push(wat_reader::identifier::receiver(final_seg));
        wat_reader::identifier::method(final_seg)
    } else {
        // A bare `/` (division → name `/`) or no slash: the final segment IS the name.
        final_seg
    };
    Some(format!("{}/{}", ns_parts.join("."), name))
}

fn ns_to_enum_path(ns: &str) -> String {
    format!(":{}", ns.replace('.', "::"))
}

fn reconstruct_struct(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature == crate::types::Nature::Struct => a,
        _ => {
            // Arc 278 Stone A — the UNKNOWN-tag miss. In foreign mode, a map body
            // under an unregistered tag reconstructs a self-describing ForeignRecord
            // (the consumer LACKS the type); strict mode is UNCHANGED — it errors.
            if foreign {
                return build_foreign_record(ns, name, entries, types, ctx);
            }
            // arc 138: no span — reconstruct_struct operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: name.to_string(), body_shape: "map" } });
        }
    };
    // Build a key → value lookup from the EDN map.
    let mut by_key: std::collections::HashMap<String, &OwnedValue> =
        std::collections::HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        if let OwnedValue::Keyword(kw) = k {
            // We render fields with bare-name keywords (no namespace).
            // Match on `name()`.
            by_key.insert(kw.name().to_string(), v);
        }
    }
    // Walk declared fields in declaration order; build positional
    // field values that StructValue expects.
    //
    // Arc 113 slice 3 — Option-aware re-wrapping. wat-edn's writer
    // unwraps `Value::Option(Some(x))` → bare `x` on the wire (and
    // `None` → Nil). To round-trip cleanly, the reader needs to put
    // the Option layer back when the declared field type is
    // `Option<T>`. Without this, a Failure with `actual:
    // Option<String>` reads back as `Value::String` instead of
    // `Value::Option(Some(String))`, and downstream pattern-matches
    // (`(Some a)` / `(:None ...)`) hit `PatternMatchFailed`. Same
    // logic applies for any struct field declared as Option-of-T.
    let mut fields: Vec<Value> = Vec::with_capacity(def.fields.len());
    for (fname, fty) in &def.fields {
        let fv = by_key.get(fname.as_str()).ok_or_else(|| {
            // arc 138: no span — reconstruct_struct operates on parsed OwnedValue, no WatAST
            EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnknownStructField { type_path: path.clone(), key: fname.clone() } }
        })?;
        let inner = edn_to_value_caps(fv, Some(types), allow_caps, foreign, ctx)?;
        let wrapped = rewrap_option_field(fty, inner);
        fields.push(wrapped);
    }
    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        path.trim_start_matches(':').to_string(),
        def.names_arc(),
        fields,
    ))))
}

/// Arc 234 Stone 234.7a — Decode a base-record tagged-map back to `Value::Aggregate(nature=Record)`.
///
/// Arc 293.2b: uses `AggregateDef` (kind=Record|HolonRecord) instead of the annihilated
/// `RecordDef`. Fields are always-typed (D2), so `rewrap_option_field` applies.
fn reconstruct_record(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    // Arc 293.2b — record aggregates (kind != Struct) replace TypeDef::Record.
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => a,
        _ => {
            // Arc 278 Stone A — foreign-mode miss (map body, unregistered tag) →
            // ForeignRecord. (In the live dispatch this arm is reached only if the
            // registry changed between the tagged_to_value routing check and here;
            // handled symmetrically with reconstruct_struct for robustness.)
            if foreign {
                return build_foreign_record(ns, name, entries, types, ctx);
            }
            return Err(EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::UnknownTag {
                    ns: ns.to_string(),
                    name: name.to_string(),
                    body_shape: "map",
                },
            });
        }
    };
    // Build a key → value lookup from the EDN map (bare keyword names).
    let mut by_key: std::collections::HashMap<String, &OwnedValue> =
        std::collections::HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        if let OwnedValue::Keyword(kw) = k {
            by_key.insert(kw.name().to_string(), v);
        }
    }
    // Walk declared fields in declaration order.
    let mut fields: Vec<Value> = Vec::with_capacity(def.fields.len());
    for (fname, fty) in def.fields.iter() {
        let fv = by_key.get(fname.as_str()).ok_or_else(|| EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::UnknownStructField {
                type_path: path.clone(),
                key: fname.clone(),
            },
        })?;
        let inner = edn_to_value_caps(fv, Some(types), allow_caps, foreign, ctx)?;
        // Apply Option-rewrapping when the field is Option<T>.
        let wrapped = rewrap_option_field(fty, inner);
        fields.push(wrapped);
    }
    // class stored without leading ':'; path has it — strip.
    let class = path.strip_prefix(':').unwrap_or(&path).to_string();
    Ok(Value::Aggregate(Arc::new(AggregateValue::record(class, def.names_arc(), Arc::new(fields)))))
}

/// Arc 234 Stone 234.7b (REWRITTEN by Arc 294.g) — Decode a holon-record tagged-MAP back to
/// `Value::Aggregate(nature=HolonRecord)`.
///
/// Before 294.g the wire body was the serialized hologram (`Bind(_, Bundle(children))`) and
/// this function PROJECTED fields out of it. 294.g collapsed the encode side — a holon
/// record's wire form is now IDENTICAL to a base record's (class tag + field map; the
/// hologram never crosses the wire; see `value_to_edn_with`'s `Value::Aggregate` arm) — so
/// that projection is retired. Fields now come from the map exactly as `reconstruct_record`
/// builds them; the hologram is then DERIVED from those fields via `build_holon_hologram`
/// (`crate::runtime`), the SAME function `aggregate-new` calls at construction time
/// (arc 294.c.2a) — no second implementation, and the index is derived, never read off the
/// wire (the non-vacuity guard this stone's probe rows 3-4 exist to enforce).
///
/// `ctx` is the ambient `EncodingCtx` `build_holon_hologram` needs to derive the index
/// (construction-capacity budget). It comes from the `SymbolTable` at the decode entry point
/// (threaded down from `eval_edn_read` et al.). A decode call with no live program's
/// `EncodingCtx` attached cannot derive an index for a HolonRecord class — this errors loudly
/// rather than fabricate a wrong-dimension context or silently skip the derivation.
fn reconstruct_holon_record(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature == crate::types::Nature::HolonRecord => a,
        _ => {
            // Arc 278 Stone A — foreign-mode miss (map body, unregistered/mismatched-nature
            // tag) → ForeignRecord. Symmetric with reconstruct_struct/reconstruct_record.
            if foreign {
                return build_foreign_record(ns, name, entries, types, ctx);
            }
            return Err(EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::UnknownTag {
                    ns: ns.to_string(),
                    name: name.to_string(),
                    body_shape: "map",
                },
            });
        }
    };
    // Build a key → value lookup from the EDN map (bare keyword names) — identical to
    // reconstruct_record; a holon record's wire body IS a base record's body.
    let mut by_key: std::collections::HashMap<String, &OwnedValue> =
        std::collections::HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        if let OwnedValue::Keyword(kw) = k {
            by_key.insert(kw.name().to_string(), v);
        }
    }
    // Walk declared fields in declaration order.
    let mut field_names: Vec<String> = Vec::with_capacity(def.fields.len());
    let mut fields: Vec<Value> = Vec::with_capacity(def.fields.len());
    for (fname, fty) in def.fields.iter() {
        let fv = by_key.get(fname.as_str()).ok_or_else(|| EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::UnknownStructField {
                type_path: path.clone(),
                key: fname.clone(),
            },
        })?;
        let inner = edn_to_value_caps(fv, Some(types), allow_caps, foreign, ctx)?;
        let wrapped = rewrap_option_field(fty, inner);
        field_names.push(fname.clone());
        fields.push(wrapped);
    }
    // class stored without leading ':'; path has it — strip.
    let class = path.strip_prefix(':').unwrap_or(&path).to_string();

    // Derive the hologram from the decoded fields — the index is NEVER read off the wire.
    let ctx = ctx.ok_or_else(|| EdnReadError {
        span: crate::rust_caller_span!(),
        kind: EdnReadErrorKind::Other(format!(
            "reconstruct_holon_record: decoding {path} (a holon record) requires an \
             EncodingCtx to derive its hologram, and none is attached to this decode call — \
             the hologram is derived on arrival, never read off the wire (arc 294.g), so a \
             decode door with no live program's EncodingCtx cannot reconstruct a holon record"
        )),
    })?;
    let span = crate::rust_caller_span!();
    let hologram = crate::runtime::build_holon_hologram(&class, &field_names, &fields, ctx, &span)
        .map_err(|e| EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::Other(format!(
                "reconstruct_holon_record: build_holon_hologram failed: {e}"
            )),
        })?;

    Ok(Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class,
        Arc::new(field_names),
        Arc::new(fields),
        hologram,
    ))))
}

/// Arc 113 slice 3 — when a declared field type is `Option<T>` but
/// the EDN-bridged value isn't already a `Value::Option`, wrap it.
/// `Value::Unit` (Nil round-trip) → `None`; anything else → `Some`.
/// Already-Option values pass through. Non-Option declared types
/// pass through unchanged.
fn rewrap_option_field(fty: &crate::types::TypeExpr, v: Value) -> Value {
    let is_option = matches!(
        fty,
        crate::types::TypeExpr::Parametric { head, .. } if head == "wat::core::Option"
    );
    if !is_option {
        return v;
    }
    match v {
        Value::Option(_) => v, // already wrapped
        Value::Unit => Value::Option(Arc::new(None)),
        other => Value::Option(Arc::new(Some(other))),
    }
}

fn reconstruct_enum_tagged(
    ns: &str,
    variant_name: &str,
    items: &[OwnedValue],
    types: &crate::types::TypeEnv,
    allow_caps: bool,
    foreign: bool,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let path = ns_to_enum_path(ns);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Enum(d)) => d,
        _ => {
            // Arc 278 Stone A — the UNKNOWN-tag miss for a vector body. In foreign
            // mode, reconstruct a self-describing ForeignVariant (enum-class +
            // variant + positional fields, recursively decoded); strict mode errors.
            if foreign {
                return build_foreign_variant(ns, variant_name, items, types, ctx);
            }
            // arc 138: no span — reconstruct_enum_tagged operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: variant_name.to_string(), body_shape: "vector" } });
        }
    };
    let variant = def
        .variants
        .iter()
        .find(|v| match v {
            crate::types::EnumVariant::Unit(n) => n == variant_name,
            crate::types::EnumVariant::Tagged { name, .. } => name == variant_name,
        })
        .ok_or_else(|| {
            // arc 138: no span — reconstruct_enum_tagged operates on parsed OwnedValue, no WatAST
            EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::EnumVariantNotFound { type_path: path.clone(), variant: variant_name.to_string() } }
        })?;
    // Arc 113 slice 3 — Option-aware field wrapping (same shape as
    // reconstruct_struct). Variant field types come from
    // `EnumVariant::Tagged.fields`; bridge each item, then rewrap
    // Option layers wat-edn dropped on the wire.
    let declared_fields: &[(String, crate::types::TypeExpr)] = match variant {
        crate::types::EnumVariant::Tagged { fields, .. } => fields.as_slice(),
        crate::types::EnumVariant::Unit(_) => &[],
    };
    // `def` holds the registry directly (`variant` was already matched out of it above).
    let names = match variant {
        crate::types::EnumVariant::Tagged { .. } => {
            def.variant_names_arc(variant_name).unwrap_or_else(|| {
                panic!(
                    "reconstruct_enum_tagged: `{path}::{variant_name}` matched Tagged above but \
                     variant_names_arc returned None — def and its own match arm disagree"
                )
            })
        }
        crate::types::EnumVariant::Unit(_) => crate::runtime::no_field_names(),
    };
    let mut fields: Vec<Value> = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        let inner = edn_to_value_caps(item, Some(types), allow_caps, foreign, ctx)?;
        let wrapped = match declared_fields.get(idx) {
            Some((_, fty)) => rewrap_option_field(fty, inner),
            None => inner,
        };
        fields.push(wrapped);
    }
    Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
        type_path: path,
        variant_name: variant_name.to_string(),
        names,
        fields,
    })))
}

/// Arc 278 Stone A — build a self-describing [`Value::ForeignRecord`] from an
/// UNKNOWN map-bodied tag (`#ns/name {…}`). The class is the colon-free
/// fully-qualified tag path (`some::unknown::Rec`); each field key is
/// self-carried (the bare keyword name) and each value is recursively decoded
/// in FOREIGN mode, so nesting decodes all the way down. Field order is
/// preserved as read, so re-serialization reproduces the same `#ns/name {…}`.
/// `allow_caps=false` — foreign decode is untrusted parsed data.
fn build_foreign_record(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    let class = path.strip_prefix(':').unwrap_or(&path).to_string();
    let mut fields: Vec<(String, Value)> = Vec::with_capacity(entries.len());
    for (k, v) in entries {
        // Foreign records carry keyword-named fields (mirrors record/struct
        // decode + the `ForeignRecord/get : (_, Keyword) -> Option<Value>` accessor).
        // A non-keyword key is out of contract → loud error (no-hidden-failures).
        let key = match k {
            OwnedValue::Keyword(kw) => kw.name().to_string(),
            other => {
                return Err(EdnReadError {
                    span: crate::rust_caller_span!(),
                    kind: EdnReadErrorKind::Other(format!(
                        "read-foreign: ForeignRecord field key must be a keyword, got {}",
                        edn_shape_name(other)
                    )),
                });
            }
        };
        let val = edn_to_value_caps(v, Some(types), /*allow_caps*/ false, /*foreign*/ true, ctx)?;
        fields.push((key, val));
    }
    Ok(Value::ForeignRecord(Arc::new(ForeignRecordValue { class, fields })))
}

/// Arc 278 Stone A — build a self-describing [`Value::ForeignVariant`] from an
/// UNKNOWN vector-bodied tag (`#<enum-path>/<Variant> [...]`). The enum class
/// is the colon-free FQDN of the tag namespace (`some::unknown::Kind`), the
/// variant is the tag name (`Click`), and each positional field is recursively
/// decoded in FOREIGN mode. Re-serializes to the same tag + vector body.
fn build_foreign_variant(
    ns: &str,
    variant_name: &str,
    items: &[OwnedValue],
    types: &crate::types::TypeEnv,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let enum_path = ns_to_enum_path(ns);
    let enum_class = enum_path.strip_prefix(':').unwrap_or(&enum_path).to_string();
    let mut fields: Vec<Value> = Vec::with_capacity(items.len());
    for item in items {
        fields.push(edn_to_value_caps(item, Some(types), /*allow_caps*/ false, /*foreign*/ true, ctx)?);
    }
    Ok(Value::ForeignVariant(Arc::new(ForeignVariantValue {
        enum_class,
        variant: variant_name.to_string(),
        fields,
    })))
}

// ─── The walker ──────────────────────────────────────────────────

/// Convert a wat `Value` to a `wat_edn::OwnedValue`. Back-compat
/// shim that calls [`value_to_edn_with`] without a type registry —
/// renders structs with positional `:field-N` keys. Prefer
/// `value_to_edn_with` when a registry is reachable so structs
/// render with their declared field names.
pub fn value_to_edn(v: &Value) -> OwnedValue {
    value_to_edn_with(v, None)
}

// ⛔ `value_to_edn_string(v)` — the types-less door — is DELETED (2026-08-14).
//
// It hardcoded `None` for the type registry, so every caller silently rendered record
// fields POSITIONALLY (`{:field-0 1 :field-1 2}`) instead of by name. The names were
// never missing: `:wat::core::EvalError` is registered as `Aggregate`/`Nature::Struct`
// WITH its fields, and the sibling `value_to_edn_string_with` reaches them fine — the
// lookup simply was not wired through this door. Three unrelated symptoms, one cause:
// the `field-N` diagnostics blob (296/NOTE-value-to-edn-renders-fields-positionally.md),
// `send'`'s `field-0`/`field-1` (bridged with a thread-local in 258.5b, killed by
// 258.5b-ii), and `(:wat::core::str <record>)` (introduced by 279.2 and fixed here).
//
// A default you cannot see at the call site is a default nobody audits. There is now ONE
// door — `value_to_edn_string_with` — and a caller with no registry passes `None`
// EXPLICITLY, in the open, where the next reader can ask why.

/// Encode a `Value` to a compact EDN `String` with an optional type registry.
///
/// Arc 258.5b-ii: called by `eval_peer_send_prime` (PEER_TYPE_PATH socket-tier
/// arm) to encode with `sym.types()` so records cross the wire with named
/// fields rather than positional `:field-{i}` fallback.  The resulting `String`
/// is shipped via `Peer::send_wire` — no thread-local involved.
pub(crate) fn value_to_edn_string_with(
    v: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> String {
    wat_edn::write(&value_to_edn_with(v, types))
}

/// Decode a compact EDN `String` back to a `Value` — the inverse of
/// [`value_to_edn_string`]. Used by the process-tier apply-loop to
/// deserialize the parent's encoded messages in the child, and by
/// `EdnRepresentable::from_wire for Value` (`comms/mod.rs`).
///
/// Passes `None` for the type registry — reconstructs only primitive
/// Values (i64, f64, bool, nil, String, keyword, Vec, HashMap). User-
/// defined structs/enums are not reconstructed without a TypeEnv; the
/// process tier's program fn works on the decoded primitive scaffold.
pub(crate) fn edn_string_to_value(s: &str) -> Result<Value, EdnReadError> {
    // types=None ⇒ no tagged/registered value (incl. no HolonRecord) is ever reachable here —
    // ctx=None is therefore exact, not a shortcut (see `read_edn`'s doc).
    read_edn(s, None, None)
}

/// Arc 272 6a-i — **THE ONE TRUSTED-WIRE DECODE DOOR.** The sole entry that reconstructs portable
/// capability tags into live capabilities. Object-capability transfer-only: a
/// capability is obtained only by being handed it over a trusted channel — the process peer wire
/// (`recv'` / `select'`, whose bytes came from a lineage peer) — NEVER forged from parsed data.
///
/// Every other decode entry (`edn_to_value` / `read_edn` / `edn_string_to_value` / `:wat::edn::read`)
/// is **structurally incapable** of minting a capability: `read_edn_caps` is private, so no
/// `allow_caps` flag exists for general code to set. To mint a capability you MUST come through this
/// one named, greppable, audited door — the trap door nailed shut. A decode site that forgets to use
/// it **safe-fails to refuse** (a regression, never a forge-hole).
///
/// **v3/v4 seam:** granularity lands HERE, additively — the door grows to take the sending peer's
/// verified `SO_PEERCRED` (per-peer powerbox, v3) and ultimately a `fn(cap_type, peer, body) ->
/// Decision` predicate (the capability-decode policy language, v4). Today: coarse "trusted ⇒ caps
/// reconstruct." Enriching the policy never re-threads the decode.
pub(crate) fn decode_trusted_wire(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Value, EdnReadError> {
    let v = read_edn_caps(s, types, true, ctx)?;
    // ── RETIRED arc 293.W.2a (deleted by arc 293.W.2d) ───────────────────────
    // The §7 runtime backstop that refused a top-level Nature::Struct at the
    // wire-decode door is gone. The compile-time purity wall at wire-peer
    // PRODUCERS (peer-pair', connect', accept', program-self-peer')
    // makes the reachable struct-on-wire case structurally unrepresentable. The
    // untyped pprintln path is a trust-boundary concern outside our scope.
    Ok(v)
}

#[cfg(test)]
mod cap_decode_boundary {
    //! Arc 272 6a-i / 6c.2, updated 294.m — the trap-door ward. A capability now wears its real
    //! type home (`#wat.kernel/Address`, not the retired `wat-edn.cap` marker namespace) and
    //! reconstructs ONLY through the trusted door; the general/untrusted decode path REFUSES it
    //! because the refusal consults the REGISTRY (is this type path a registered capability
    //! codec?), never a namespace string. If this ever flips, the forge-hole reopens (parsed data
    //! minting live capabilities). This is the regression alarm bolted onto the exact trap we
    //! fell through — it must never open again.
    use super::{decode_trusted_wire, edn_string_to_value};

    // Arc 272 6c.2 — the wire format is now a SocketAddressWire record (not a bare byte vector).
    // Arc 294.m — the outer tag is the capability's real type home, `#wat.kernel/Address`, not
    // the retired `wat-edn.cap` marker namespace.
    const CAP_TAG_GENERAL: &str = "#wat.kernel/Address #wat.kernel/SocketAddressWire {:minter-pid 1 :name [1 2 3 4 5]}";

    fn make_types() -> crate::types::TypeEnv {
        use crate::types::{AggregateDef, Nature, TypeDef, TypeExpr};
        // with_builtins seeds :wat::core::Record (required parent for SocketAddressWire).
        let mut env = crate::types::TypeEnv::with_builtins();
        // Arc 293.2b — use AggregateDef (nature=Record) instead of the annihilated RecordDef.
        env.register_stdlib(TypeDef::Aggregate(AggregateDef {
            name: ":wat::kernel::SocketAddressWire".to_string(),
            type_params: vec![],
            nature: Nature::Record,
            restrictions: None,
            // minter-pid <- :wat::core::i64
            // name       <- (:wat::core::Vector :- [wat::core::i64])
            fields: vec![
                ("minter-pid".to_string(), TypeExpr::Path(":wat::core::i64".to_string())),
                ("name".to_string(), TypeExpr::Parametric {
                    head: "wat::core::Vector".to_string(),
                    args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
                }),
            ],
        }))
        .expect("SocketAddressWire registration must succeed");
        env
    }

    #[test]
    fn general_decode_refuses_capability_tags() {
        assert!(
            edn_string_to_value(CAP_TAG_GENERAL).is_err(),
            "general/untrusted decode MUST refuse a capability tag — a capability is handed over a \
             trusted channel, never forged from parsed data (ocap transfer-only)"
        );
    }

    #[test]
    fn trusted_door_reconstructs_capability_tags() {
        let types = make_types();
        assert!(
            decode_trusted_wire(CAP_TAG_GENERAL, Some(&types), None).is_ok(),
            "the trusted-wire door MUST reconstruct a capability tag into a live capability"
        );
    }
}

/// Convert a wat `Value` to `wat_edn::OwnedValue` consulting the
/// frozen type registry for struct field names. When a struct's
/// `StructDef` is found in `types`, fields render as a Map keyed by
/// the declared field name (`:caller`, `:level`, etc); otherwise
/// falls back to positional `:field-N` keys.
///
/// The registry comes through `SymbolTable.types` (arc 085's
/// capability carrier).
pub fn value_to_edn_with(
    v: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> OwnedValue {
    match v {
        // ── Primitive leaves ─────────────────────────────────────
        Value::Unit => OwnedValue::Nil,
        Value::bool(b) => OwnedValue::Bool(*b),
        Value::i64(n) => OwnedValue::Integer(*n),
        Value::u8(n) => OwnedValue::Integer(*n as i64),
        Value::f64(x) => OwnedValue::Float(*x),
        Value::String(s) => OwnedValue::String(std::borrow::Cow::Owned((**s).clone())),
        Value::wat__core__keyword(k) => keyword_from_wat_path(k),

        // ── Option / Result ──────────────────────────────────────
        // Arc 278 Stone A.0 — uniform VECTOR-bodied variant encoding.
        // Every enum variant (including Option/Result) is `#tag [field-vec]`:
        // `None → []`, `Some(v) → [v]`, `Ok(v) → [v]`, `Err(e) → [e]`. The
        // arc-298.1 direct-body special-case (`#Some v`, `#None nil`) is retired
        // so `Some(nil) → [nil]` (arity visible) never collides with `None → []`.
        Value::Option(opt) => match &**opt {
            None => OwnedValue::Tagged(
                Tag::ns("wat.core.Option", "None"),
                Box::new(OwnedValue::Vector(vec![])),
            ),
            Some(inner) => OwnedValue::Tagged(
                Tag::ns("wat.core.Option", "Some"),
                Box::new(OwnedValue::Vector(vec![value_to_edn_with(inner, types)])),
            ),
        },
        Value::Result(r) => match &**r {
            Ok(inner) => OwnedValue::Tagged(
                Tag::ns("wat.core.Result", "Ok"),
                Box::new(OwnedValue::Vector(vec![value_to_edn_with(inner, types)])),
            ),
            Err(inner) => OwnedValue::Tagged(
                Tag::ns("wat.core.Result", "Err"),
                Box::new(OwnedValue::Vector(vec![value_to_edn_with(inner, types)])),
            ),
        },

        // ── Compound containers ──────────────────────────────────
        Value::Vec(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_with(x, types)).collect())
        }
        // Arc 220 Stone 220.4 — List → EDN parens form (OwnedValue::List).
        // Preserves the List/Vector distinction on the wire so Clojure sees
        // a proper list `(1 2 3)` rather than a vector `[1 2 3]`.
        Value::wat__core__List(xs) => {
            OwnedValue::List(xs.iter().map(|x| value_to_edn_with(x, types)).collect())
        }
        Value::Tuple(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_with(x, types)).collect())
        }
        // Stone 216.5c — iterate m.iter() for (k, v) directly (native HashMap<Value, Value>).
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.iter()
                .map(|(k, v)| (value_to_edn_with(k, types), value_to_edn_with(v, types)))
                .collect(),
        ),
        // Arc-278-0a — PersistentMap writes as a TAGGED literal `#wat.core/PersistentMap {…}`
        // so round-trip IDENTITY is preserved: a std-HashMap `{}` reads back as wat__std__HashMap;
        // the tagged form reads back as PersistentMap (distinct identity per the DESIGN contract).
        Value::wat__core__PersistentMap(m) => OwnedValue::Tagged(
            Tag::ns("wat.core", "PersistentMap"),
            Box::new(OwnedValue::Map(
                m.iter()
                    .map(|(k, v)| (value_to_edn_with(k, types), value_to_edn_with(v, types)))
                    .collect(),
            )),
        ),
        // Arc-278-0b — PersistentVector writes as a TAGGED literal `#wat.core/PersistentVector [...]`
        // so round-trip IDENTITY is preserved: a bare `[…]` reads back as wat::Vec (std Vector);
        // the tagged form reads back as PersistentVector (distinct identity per the DESIGN contract).
        Value::wat__core__PersistentVector(pv) => OwnedValue::Tagged(
            Tag::ns("wat.core", "PersistentVector"),
            Box::new(OwnedValue::Vector(
                pv.iter()
                    .map(|x| value_to_edn_with(x, types))
                    .collect(),
            )),
        ),
        Value::wat__std__HashSet(s) => OwnedValue::Set(
            // Stone 216.5b — iterate s.iter() (Values directly, not String keys).
            s.iter().map(|x| value_to_edn_with(x, types)).collect(),
        ),

        // ── User-declared struct / record / holon-record ─────────
        // Arc 293.R2.1 — all three collapsed into Value::Aggregate.
        Value::Aggregate(sv) if sv.nature == crate::types::Nature::Struct => {
            let type_key = format!(":{}", sv.class);
            let tag = tag_from_type_path(&type_key);
            // Arc 296 G-2 — names are carried on the value; no registry lookup, no fallback.
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .names
                .iter()
                .zip(sv.fields.iter())
                .map(|(name, fv)| {
                    (
                        OwnedValue::Keyword(Keyword::new(name.clone())),
                        value_to_edn_with(fv, types),
                    )
                })
                .collect();
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(entries)))
        }
        Value::Enum(ev) => {
            let tag_name = format!("{}::{}", ev.type_path, ev.variant_name);
            let tag = tag_from_type_path(&tag_name);
            if ev.fields.is_empty() {
                // Arc 278 Stone A.0 — unit / zero-field variant renders as `#tag []`
                // (empty field-vector), NEVER a bare-nil body. `nil` is now the unit
                // value ONLY, so body-shape is a perfect discriminator (map=record,
                // vector=variant, nil=unit).
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Vector(vec![])))
            } else {
                let payload: Vec<OwnedValue> = ev
                    .fields
                    .iter()
                    .map(|x| value_to_edn_with(x, types))
                    .collect();
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Vector(payload)))
            }
        }

        // ── Arc 278 Stone A — foreign dynamic values (self-describing) ──
        // Re-serialize FAITHFULLY to the SAME `#tag {…}` / `#tag [...]` the
        // reader consumed. Keys/fields are SELF-carried (not registry-looked-up,
        // which would fall to `field-{i}` and lose the foreign names). Recursive:
        // nested foreign values re-emit via `value_to_edn_with`.
        Value::ForeignRecord(fr) => {
            let type_key = format!(":{}", fr.class);
            let tag = tag_from_type_path(&type_key);
            let entries: Vec<(OwnedValue, OwnedValue)> = fr
                .fields
                .iter()
                .map(|(k, v)| {
                    (
                        OwnedValue::Keyword(Keyword::new(k.clone())),
                        value_to_edn_with(v, types),
                    )
                })
                .collect();
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(entries)))
        }
        Value::ForeignVariant(fv) => {
            let tag_name = format!(":{}::{}", fv.enum_class, fv.variant);
            let tag = tag_from_type_path(&tag_name);
            let payload: Vec<OwnedValue> = fv
                .fields
                .iter()
                .map(|x| value_to_edn_with(x, types))
                .collect();
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Vector(payload)))
        }

        // ── Substrate compound values — opaque or structural ─────
        // Arc 294.j RELAND — `edn_shim` forgets the algebra, and renders
        // DATA, never a wat source form (DESIGN-STONE-294.j, the ⛔
        // CORRECTION section). The first strike reached for `holon_to_watast`
        // because it was total and adjacent — it renders wat SOURCE, which
        // is exactly the defect the builder caught (a Thermometer wire form
        // that round-trips to a Bundle and crashes the far side). The job
        // needs a renderer of DATA: `from_holon_item` (runtime.rs), the
        // holon→data inverse `:wat::holon::from-holon` already uses, which
        // refuses (Err) on anything that is not data — Thermometer/SlotMarker
        // included, because they are constructor directives, not data. See
        // `holon_ast_to_edn_data` below for the three-case dispatch.
        Value::holon__HolonAST(h) => holon_ast_to_edn_data(h, types),
        // Arc 294.j — the realized VSA vector is the algebra's OWN terminal
        // artifact (the materialized `holon::Vector` a Bind/Bundle tree
        // evaluates to), so it shares the "derived, not shipped" disposition
        // the DESIGN STONE's classification table gives the algebra family —
        // it was the one member of that family not living in
        // `holon_ast_to_edn` (it is a `Value::Vector`, not a `HolonAST`
        // variant — holon-rs has no such variant). Its OLD tag shared the
        // now-dead namespace by accident of authorship, not by kinship with
        // the tag/reader pair this stone kills; it never had a reader arm at
        // all (`Vector` never appeared in `edn_holon_tag_to_ast`), so there
        // is no decode path to remove. Body is unchanged (`:dim`, the one
        // legitimate non-secret fact about an opaque handle — same
        // "preserve real data" call 294.i made for `HandlePool`'s name);
        // only the home moves off the dead namespace, to the same
        // `wat.holon` per-type home the VSA five already use.
        Value::Vector(vec) => OwnedValue::Tagged(
            Tag::ns("wat.holon", "Vector"),
            Box::new(OwnedValue::Map(vec![(
                OwnedValue::Keyword(Keyword::new("dim")),
                OwnedValue::Integer(vec.dimensions() as i64),
            )])),
        ),

        // ── Opaque substrate handles — type-tagged nil ───────────
        // A WatAST is a parsed form — by definition an EDN value (watast_to_edn/edn_to_watast
        // are a total bijection). Render it faithfully as its form (legible + recoverable);
        // opaque-nil was a lie. Round-trip-as-WatAST is type-directed (from-edn :T / the typed slot).
        Value::wat__WatAST(a) => crate::wat_edn_bridge::watast_to_edn(a.as_ref()),
        Value::wat__core__fn(_) => opaque_nil("wat.core", "fn"),
        Value::wat__kernel__Sender(_) => opaque_nil("wat.kernel", "Sender"),
        Value::wat__kernel__Receiver(_) => opaque_nil("wat.kernel", "Receiver"),
        // Arc 294.i — the ONE exception to "everything decorates nil": HandlePool carries its
        // pool name as the body today. Preserve it; flattening to nil would silently drop data.
        Value::wat__kernel__HandlePool { name, .. } => OwnedValue::Tagged(
            Tag::ns("wat.kernel", "HandlePool"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(
                (**name).clone(),
            ))),
        ),
        Value::wat__kernel__ChildHandle(_) => opaque_nil("wat.kernel", "ChildHandle"),
        Value::io__IOReader(_) => opaque_nil("wat.io", "IOReader"),
        Value::io__IOWriter(_) => opaque_nil("wat.io", "IOWriter"),
        Value::RustOpaque(inner) => {
            // Arc 272 narrow-waist — GENERIC capability dispatch (the FROZEN waist; never changes
            // per-capability). If this opaque is a registered PORTABLE capability with a portable
            // form, emit its `#wat.kernel/<Name>` tag; otherwise it is a process-local handle (an fd,
            // a `Sender`) that must NOT cross → the payload-less per-type-home tag (the decoder
            // refuses any bare-nil tagged value; arc 278 A.0). The per-capability codecs live in
            // `crate::capability::registry`. `types` is required by record-based codecs (arc 272
            // 6c.2 SocketAddressWire field naming); when `types` is None (display/logging paths,
            // and the process-tier `send'` path when no registry is threaded through — see arc
            // 294.i STOP-2 finding), capability encoding is skipped and the value falls to its
            // per-type-home tag. Arc 294.i: `RustOpaque` is a Rust carrier word, never a tag name —
            // route the fallback through `tag_from_type_path`, the same fn already used at five
            // other sites in this file (structs, enums, records).
            if let Some(t) = types {
                if let Some(cap_tag) = crate::capability::encode_capability(inner, t) {
                    return cap_tag;
                }
            }
            OwnedValue::Tagged(
                tag_from_type_path(inner.type_path),
                Box::new(OwnedValue::Nil),
            )
        }
        // Arc 294.i — the VSA five: not derivable from the `Value` variant name (bare
        // `Hologram`, `Engram`, …), but the inner Rust type says it — all five are `holon::X`,
        // and the codebase already names them `wat::holon::X` (see `value.rs` type_name/gate
        // entries for OnlineSubspace/Reckoner/Engram/EngramLibrary/Hologram). Home measured
        // from that existing convention, not invented.
        Value::OnlineSubspace(_) => opaque_nil("wat.holon", "OnlineSubspace"),
        Value::Reckoner(_) => opaque_nil("wat.holon", "Reckoner"),
        Value::Engram(_) => opaque_nil("wat.holon", "Engram"),
        Value::EngramLibrary(_) => opaque_nil("wat.holon", "EngramLibrary"),
        Value::Hologram(_) => opaque_nil("wat.holon", "Hologram"),
        Value::Instant(t) => OwnedValue::Inst(*t),
        Value::Duration(ns) => OwnedValue::Integer(*ns),
        // Arc 207 — typed Uuid → EDN `#uuid "..."` reader literal.
        // Mirrors `Value::Instant → OwnedValue::Inst` pattern.
        // `uuid::Uuid` is `Copy`; `OwnedValue::Uuid` already exists
        // in wat-edn (no crates/wat-edn/ edits needed).
        Value::wat__core__Uuid(u) => OwnedValue::Uuid(*u),
        // Arc 220 — typed Char → EDN character literal.
        // `char` is `Copy`; `OwnedValue::Char` already exists in wat-edn.
        Value::wat__core__Char(c) => OwnedValue::Char(*c),
        // Arc 300 stone B — typed Rational → EDN rational literal round-trip.
        Value::wat__core__Rational(r) => OwnedValue::Rational(Box::new((**r).clone())),
        // Arc 300 stone C1 — typed BigInt → EDN bigint literal round-trip (mirrors
        // Rational immediately above, one type over).
        Value::wat__core__BigInt(n) => OwnedValue::BigInt(Box::new((**n).clone())),
        // Arc 293.R2.1 — Record/HolonRecord: Aggregate with nature != Struct.
        // No guard here — the Struct arm above catches nature==Struct; this arm is reached
        // only for Record/HolonRecord. Guard dropped so Rust's exhaustiveness checker sees
        // Value::Aggregate(_) as fully covered.
        //
        // Arc 294.g — ONE arm, not two. A holon record's wire form is IDENTICAL to a base
        // record's: the class tag and its fields. The hologram (`a.holon`) is a DERIVED
        // INDEX — 294.c.1 landed identity-as-EDN-data (Eq/Hash keyed on (holder, class,
        // fields)), so the hologram has no business crossing the wire; the receiver knows
        // `:t::Holo` is holon-held from the type registry and derives its own
        // (`build_holon_hologram`). This is the body every base record already produced
        // (Arc 234 Stone 234.7a); the old `HolonForm::Hologram` arm (Stone 234.7b, riding
        // the hologram as a serialized tagged-HolonAST tree) is ANNIHILATED.
        Value::Aggregate(a) => {
            let type_key = format!(":{}", a.class);
            let tag = tag_from_type_path(&type_key);
            // Arc 296 G-2 — names are carried on the value; no registry lookup, no fallback.
            let entries: Vec<(OwnedValue, OwnedValue)> = a
                .names
                .iter()
                .zip(a.fields.iter())
                .map(|(name, fv)| {
                    (
                        OwnedValue::Keyword(Keyword::new(name.clone())),
                        value_to_edn_with(fv, types),
                    )
                })
                .collect();
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(entries)))
        }
        // Arc 118 — Stream: opaque (lazy; realizing for EDN would diverge on infinite seqs).
        // Render the forced prefix if available, otherwise as an opaque lazy sentinel.
        Value::wat__stream__Stream(seq) => {
            use crate::stream::Stream;
            match seq.as_ref() {
                Stream::Empty => OwnedValue::List(vec![]),
                Stream::Cons { head, .. } => {
                    // Only render the head (forced); tail may be infinite. NOT flattened to nil —
                    // like HandlePool, the forced head is real data the model does not discard;
                    // only the namespace moves home (arc 294.i).
                    OwnedValue::Tagged(
                        Tag::ns("wat.stream", "Stream"),
                        Box::new(value_to_edn_with(head, types)),
                    )
                }
                // Arc 294.i — lazy-seq is a Stream::Thunk|NativeThunk sub-state, not its own
                // Value variant, so it shares Stream's home namespace.
                Stream::Thunk(_) | Stream::NativeThunk(_) => opaque_nil("wat.stream", "lazy-seq"),
            }
        }
        // Stone 237.2 — wat__core__clauses: opaque (multi-arity dispatcher;
        // not directly serializable to EDN).
        Value::wat__core__clauses(cs) => opaque_nil("wat.core", {
            let _ = cs;
            "clauses"
        }),
        // Arc 232 Stone 232.1 — registry carriers: opaque (not value-serializable).
        Value::wat__core__extend_def(ed) => opaque_nil("wat.core", {
            let _ = ed;
            "extend-def"
        }),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────

/// Parse a wat keyword path (e.g. `:foo`, `:trading::cache::next`)
/// into an EDN Keyword. Wat uses `::` as the segment separator;
/// EDN uses `/` to split namespace from name. The wat-side
/// `:a::b::c` becomes EDN `:a.b/c` (last segment is the name; the
/// rest joined with `.` is the namespace, per common Clojure
/// convention). Single-segment wat keywords (`:foo`) become
/// non-namespaced EDN keywords.
pub(crate) fn keyword_from_wat_path(k: &str) -> OwnedValue {
    let stripped = k.strip_prefix(':').unwrap_or(k);
    if stripped.contains("::") {
        let ns = wat_reader::identifier::path(stripped).replace("::", ".");
        let name = wat_reader::identifier::leaf(stripped);
        match Keyword::try_ns(&ns, name) {
            Ok(kw) => OwnedValue::Keyword(kw),
            // EDN cannot spell this one — carry it VERBATIM rather than
            // stringify it. This used to return a bare `OwnedValue::String`
            // ("better to render than to panic on a logger call"), which is a
            // SILENT TYPE CHANGE: a Keyword goes in, a String comes out, and
            // anything that reads the value back gets the wrong type with no
            // diagnostic. Measured over the corpus, this arm fires for 10
            // distinct keywords out of 72,510 — all of them trailing-`::`
            // namespace-prefix markers (`:restricted-to` whitelists), whose
            // EDN name is empty. Still total, still non-panicking, no longer
            // a lie.
            Err(_) => crate::wat_edn_bridge::verbatim_keyword(k),
        }
    } else {
        match Keyword::try_new(stripped) {
            Ok(kw) => OwnedValue::Keyword(kw),
            Err(_) => crate::wat_edn_bridge::verbatim_keyword(k),
        }
    }
}

/// Build a tag from a type path like `:trading::cache::L1`. Drops the
/// leading colon (if present) and translates `::` to `.` for the
/// namespace; the last segment becomes the tag name.
///
/// Arc 294.k — a type whose home cannot be derived has no honest tag.
/// The old code fabricated a placeholder "no-home" namespace when `path`
/// had no `::`, and silently erased the name to the literal word
/// "unnamed" under a placeholder "opaque" namespace when `Tag::try_ns`
/// rejected the split. Both were lies that raised nothing. `value_to_edn_with` (this
/// fn's only callers) is infallible across 72 call sites outside
/// `edn_shim.rs`, so threading `Result` here would escape this module
/// (STOP-2) — `panic!` is the wall, matching the house convention
/// `holon_ast_to_edn_data` already uses for the same class of defect
/// (DESIGN-STONE-294.j).
#[track_caller]
pub(crate) fn tag_from_type_path(path: &str) -> Tag {
    let stripped = path.strip_prefix(':').unwrap_or(path);
    if !stripped.contains("::") {
        panic!(
            "tag_from_type_path: type path {path:?} has no `::` namespace separator — no \
             derivable EDN home (fabricating a namespace would silently erase this type's \
             identity on the wire)"
        );
    }
    let ns = wat_reader::identifier::path(stripped).replace("::", ".");
    let name = wat_reader::identifier::leaf(stripped);
    Tag::try_ns(&ns, name).unwrap_or_else(|e| {
        panic!(
            "tag_from_type_path: type path {path:?} has no derivable EDN home — \
             namespace {ns:?} / name {name:?} rejected: {e} (fabricating one would \
             silently erase this type's identity on the wire)"
        )
    })
}

/// Build a tagged-nil for an opaque handle.
fn opaque_nil(ns: &str, name: &str) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(ns, name), Box::new(OwnedValue::Nil))
}

/// Arc 294.j RELAND — encode a `HolonAST` as DATA (DESIGN-STONE-294.j, the
/// ⛔ CORRECTION section). Three cases, no algebra on the wire:
///
/// - `Thermometer` / `SlotMarker` — the two encoding DIRECTIVES (the data
///   cannot say "build a thermometer, not a 3-key map"). Render as
///   `#wat.holon/Thermometer {:value :min :max}` /
///   `#wat.holon/SlotMarker {:min :max}` — legible, self-describing, plain
///   EDN, never a call form.
/// - anything else that IS data — [`crate::runtime::from_holon_item`] (the
///   holon→data inverse `:wat::holon::from-holon` already uses) recovers the
///   `Value` it derived from; that `Value` renders through the SAME
///   `value_to_edn_with` this function is itself an arm of, wrapped in
///   `#wat/holon <data>` (arc 294.j CORRECTION 2 — `#wat.holon <data>`
///   cannot parse, `wat.holon` being a namespace with no name; `Tag::ns("wat",
///   "holon")` is the only spelling that does). The tag is what makes a data
///   holon re-liftable in EVERY position — struct field, vector element, map
///   value, service argument — with no declared type in scope required.
/// - anything else — the algebra (`Bind`/`Bundle`/`Atom`/`Permute`/`Blend`)
///   is neither data nor a directive and MUST NOT cross the wire. RAISE
///   (`panic!`) rather than fall back to a Bundle/nil/best-effort rendering —
///   `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`,
///   unrepresentable beats guarded. `value_to_edn_with` has no `Result` in
///   its signature (72 call sites, all infallible today; threading `Result`
///   through is out of this stone's scope) so `panic!` is the wall, matching
///   the house convention for a "fails closed" wat-visible condition
///   (`tests/collection/bundle_capacity.rs`'s `:panic` mode).
fn holon_ast_to_edn_data(h: &holon::HolonAST, types: Option<&crate::types::TypeEnv>) -> OwnedValue {
    use holon::HolonAST;
    match h {
        HolonAST::Thermometer { value, min, max } => OwnedValue::Tagged(
            Tag::ns("wat.holon", "Thermometer"),
            Box::new(OwnedValue::Map(vec![
                (OwnedValue::Keyword(Keyword::new("value")), OwnedValue::Float(*value)),
                (OwnedValue::Keyword(Keyword::new("min")), OwnedValue::Float(*min)),
                (OwnedValue::Keyword(Keyword::new("max")), OwnedValue::Float(*max)),
            ])),
        ),
        HolonAST::SlotMarker { min, max } => OwnedValue::Tagged(
            Tag::ns("wat.holon", "SlotMarker"),
            Box::new(OwnedValue::Map(vec![
                (OwnedValue::Keyword(Keyword::new("min")), OwnedValue::Float(*min)),
                (OwnedValue::Keyword(Keyword::new("max")), OwnedValue::Float(*max)),
            ])),
        ),
        other => match crate::runtime::from_holon_item(
            other,
            ":wat::edn::write",
            &crate::rust_caller_span!(),
        ) {
            // Arc 294.j CORRECTION 2 — the data tag is `#wat/holon <data>`, NOT `#wat.holon
            // <data>`. `#wat.holon` cannot parse: `wat.holon` is a namespace with no name,
            // the same violation as a bare `#holon` (`crates/wat-edn/src/parser.rs:355`,
            // `ErrorKind::UserTagMissingNamespace`). `Tag::ns("wat", "holon")` renders
            // `#wat/holon` — namespace "wat", name "holon" — which DOES parse, and is a
            // real tag visible in every position (struct field, vector element, map value,
            // service argument), unlike the bare-untagged data the first RELAND emitted.
            Ok(v) => OwnedValue::Tagged(
                Tag::ns("wat", "holon"),
                Box::new(value_to_edn_with(&v, types)),
            ),
            Err(e) => panic!(
                "cannot encode HolonAST to the wire — {e} — the algebra \
                 (Bind/Bundle/Atom/Permute/Blend) never crosses the wire in any form, per \
                 DESIGN-STONE-294.j; only DATA and the two directives (Thermometer/SlotMarker) do"
            ),
        },
    }
}

/// Arc 294.j RELAND — the ONE collapsed HolonAST reader (DESIGN-STONE-294.j,
/// the ⛔ CORRECTION section). Mirrors [`holon_ast_to_edn_data`]'s three
/// cases exactly:
///
/// - `#wat.holon/Thermometer {…}` / `#wat.holon/SlotMarker {…}` — the two
///   directive tags reconstruct DIRECTLY (they carry constructor details a
///   generic data walk cannot recover).
/// - `#wat/holon <data>` (arc 294.j CORRECTION 2) — a tagged data holon.
///   Unwrap the tag and lift the body via [`decode_holon_data_tag`].
/// - anything else (untagged data, for a slot that received plain EDN with
///   no wrapper) — the SAME lift applied directly to the whole value, via
///   [`edn_to_value`] (the SAME untyped decode `:wat::edn::read` uses) then
///   [`decode_holon_data_tag`]. Composing functions that already exist and
///   are already total, not a third HolonAST reader.
fn edn_derive_holon(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    ctx: Option<&crate::value::EncodingCtx>,
) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    use wat_edn::Value as Edn;
    if let Edn::Tagged(tag, body) = edn {
        if tag.namespace() == "wat.holon" {
            if let Some(holon) = decode_holon_directive_tag(tag.name(), body)? {
                return Ok(Arc::new(holon));
            }
        }
        if tag.namespace() == "wat" && tag.name() == "holon" {
            let value = edn_to_value(body, types, ctx).map_err(|e| EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::Other(format!("HolonAST decode: {e}")),
            })?;
            return Ok(Arc::new(decode_holon_data_tag(value)?));
        }
    }
    let value = edn_to_value(edn, types, ctx).map_err(|e| EdnReadError {
        span: crate::rust_caller_span!(),
        kind: EdnReadErrorKind::Other(format!("HolonAST decode: {e}")),
    })?;
    Ok(Arc::new(decode_holon_data_tag(value)?))
}

/// Derive a `HolonAST` from an already-decoded `Value` — the shared tail of
/// both `#wat/holon <data>` decode arms (`edn_derive_holon`'s typed slot and
/// `tagged_to_value`'s general/untyped dispatch): lift the `Value` via
/// `to_holon_inner`, the SAME holon-side lift `:wat::holon::literal` (`#holon
/// <form>`, arc 294.b) uses to derive a HolonAST from an arbitrary `Value`.
/// Shared so the two decode doors agree by construction, not by two
/// hand-kept-in-sync copies (the same reasoning as
/// [`decode_holon_directive_tag`] for the directive tags).
fn decode_holon_data_tag(value: Value) -> Result<holon::HolonAST, EdnReadError> {
    match crate::runtime::to_holon_inner(value, &crate::rust_caller_span!()) {
        Ok(Value::holon__HolonAST(h)) => Ok((*h).clone()),
        Ok(other) => unreachable!(
            "to_holon_inner always returns holon__HolonAST on Ok; got {other:?}"
        ),
        Err(e) => Err(EdnReadError {
            span: crate::rust_caller_span!(),
            kind: EdnReadErrorKind::Other(format!(
                "HolonAST decode: {e} (the algebra never crosses the wire — a HolonAST is \
                 derived from plain EDN data, not read back from a serialized tag)"
            )),
        }),
    }
}

/// Decode a `#wat.holon/<name> <body>` DIRECTIVE tag (`Thermometer` /
/// `SlotMarker`) into its `HolonAST`. `Ok(None)` for any other name under the
/// `wat.holon` namespace — NOT an error; the caller (both [`edn_derive_holon`]
/// and [`tagged_to_value`]'s general dispatch) falls through to its own
/// generic handling (data, or `UnknownTag`) for anything that isn't one of
/// the two directives. Shared so the typed (`:wat::holon::HolonAST` slot) and
/// untyped (any `#tag {…}` on any wire, incl. the process tier's message
/// decode — MEASURED: `service-cache-hologram.wat`'s far-side crash was this
/// exact path missing the untyped case) decode doors agree by construction,
/// not by two hand-kept-in-sync copies.
fn decode_holon_directive_tag(
    name: &str,
    body: &OwnedValue,
) -> Result<Option<holon::HolonAST>, EdnReadError> {
    match name {
        "Thermometer" => {
            let value = edn_holon_directive_field(body, "Thermometer", "value")?;
            let min = edn_holon_directive_field(body, "Thermometer", "min")?;
            let max = edn_holon_directive_field(body, "Thermometer", "max")?;
            Ok(Some(holon::HolonAST::Thermometer { value, min, max }))
        }
        "SlotMarker" => {
            let min = edn_holon_directive_field(body, "SlotMarker", "min")?;
            let max = edn_holon_directive_field(body, "SlotMarker", "max")?;
            Ok(Some(holon::HolonAST::SlotMarker { min, max }))
        }
        _ => Ok(None),
    }
}

/// Read one `f64` field out of a `#wat.holon/<tag_name>` directive's map
/// body. Shared by the `Thermometer` / `SlotMarker` decode arms above.
fn edn_holon_directive_field(
    body: &OwnedValue,
    tag_name: &str,
    field: &str,
) -> Result<f64, EdnReadError> {
    use wat_edn::Value as Edn;
    let entries = match body {
        Edn::Map(e) => e,
        other => {
            return Err(EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::UnsupportedTag(format!(
                    "wat.holon/{tag_name} body must be a map, got {}",
                    edn_shape_name(other)
                )),
            })
        }
    };
    for (k, v) in entries {
        let is_field = matches!(k, Edn::Keyword(kw) if kw.name() == field);
        if !is_field {
            continue;
        }
        return match v {
            Edn::Float(f) => Ok(*f),
            Edn::Integer(i) => Ok(*i as f64),
            other => Err(EdnReadError {
                span: crate::rust_caller_span!(),
                kind: EdnReadErrorKind::UnsupportedTag(format!(
                    "wat.holon/{tag_name} :{field} must be a number, got {}",
                    edn_shape_name(other)
                )),
            }),
        };
    }
    Err(EdnReadError {
        span: crate::rust_caller_span!(),
        kind: EdnReadErrorKind::UnsupportedTag(format!(
            "wat.holon/{tag_name} body missing field :{field}"
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::SymbolTable;
    use crate::types::TypeExpr;

    // ─── Arc 138 canary ─────────────────────────────────────────────────

    #[test]
    fn arc138_edn_read_error_message_carries_span() {
        // Trigger NoTypeRegistry — call read_edn with a tagged value but
        // no type registry. The error variant carries crate::rust_caller_span!()
        // (pattern E — raw EDN string has no WatAST origin). The Display
        // arm prefixes span_prefix, which returns "" for unknown spans.
        // This canary verifies the variant structurally carries a span and
        // that Display still renders without panic.
        let result = read_edn("#unknown/Type {}", None, None);
        let err = result.unwrap_err();
        let rendered = format!("{}", err);
        assert!(
            matches!(err, EdnReadError { kind: EdnReadErrorKind::NoTypeRegistry, .. }),
            "expected NoTypeRegistry, got: {:?}",
            err
        );
        // rune:lint(loose-assert) — Display embeds rust_caller_span!() (Rust file:line:col of the
        // read_edn call-site inside edn_shim.rs); the file:line:col prefix shifts whenever lines
        // are added above that site, making full assert_eq! infeasible
        assert!(
            rendered.contains("no type registry"),
            "expected NoTypeRegistry message; got: {}",
            rendered
        );
    }

    // ─── Arc 170 slice 1f-ι — edn_to_typed_value coercion ──────────────

    fn coerce(target: &TypeExpr, edn_text: &str) -> Result<Value, EdnCoerceError> {
        let edn = wat_edn::parse_owned(edn_text).expect("parse EDN test input");
        let sym = SymbolTable::default();
        edn_to_typed_value(target, &edn, &sym)
    }

    #[test]
    fn arc170_1fi_coerce_i64_from_integer() {
        let t = TypeExpr::Path(":wat::core::i64".into());
        let v = coerce(&t, "42").unwrap();
        assert!(matches!(v, Value::i64(42)));
    }

    #[test]
    fn arc170_1fi_coerce_string_from_quoted() {
        let t = TypeExpr::Path(":wat::core::String".into());
        let v = coerce(&t, "\"hello\"").unwrap();
        match v {
            Value::String(s) => assert_eq!(&*s, "hello"),
            other => panic!("expected Value::String; got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_bool() {
        let t = TypeExpr::Path(":wat::core::bool".into());
        let v = coerce(&t, "true").unwrap();
        assert!(matches!(v, Value::bool(true)));
    }

    #[test]
    fn arc170_1fi_coerce_f64_widens_integer() {
        let t = TypeExpr::Path(":wat::core::f64".into());
        let v = coerce(&t, "3").unwrap();
        match v {
            Value::f64(x) => assert!((x - 3.0).abs() < 1e-12),
            other => panic!("expected Value::f64; got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_nil_to_unit() {
        let t = TypeExpr::Path(":wat::core::nil".into());
        let v = coerce(&t, "nil").unwrap();
        assert!(matches!(v, Value::Unit));
    }

    #[test]
    fn arc170_1fi_coerce_option_nil_to_none() {
        // Arc 278 Stone A.0 — Option wire form is `#wat.core.Option/None []` (vector body).
        let t = TypeExpr::Parametric {
            head: "wat::core::Option".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let v = coerce(&t, "#wat.core.Option/None []").unwrap();
        match v {
            Value::Option(o) => assert!(o.is_none()),
            other => panic!("expected Value::Option(None); got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_option_some() {
        // Arc 278 Stone A.0 — Option wire form is `#wat.core.Option/Some [v]` (vector body).
        let t = TypeExpr::Parametric {
            head: "wat::core::Option".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let v = coerce(&t, "#wat.core.Option/Some [7]").unwrap();
        match v {
            Value::Option(o) => match &*o {
                Some(Value::i64(7)) => {}
                other => panic!("expected Some(Value::i64(7)); got {:?}", other),
            },
            other => panic!("expected Value::Option(Some); got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_vector_of_i64() {
        let t = TypeExpr::Parametric {
            head: "wat::core::Vector".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let v = coerce(&t, "[1 2 3]").unwrap();
        match v {
            Value::Vec(xs) => {
                assert_eq!(xs.len(), 3);
                assert!(matches!(xs[0], Value::i64(1)));
                assert!(matches!(xs[2], Value::i64(3)));
            }
            other => panic!("expected Value::Vec; got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_tuple_heterogeneous() {
        let t = TypeExpr::Tuple(vec![
            TypeExpr::Path(":wat::core::i64".into()),
            TypeExpr::Path(":wat::core::String".into()),
        ]);
        let v = coerce(&t, "[1 \"x\"]").unwrap();
        match v {
            Value::Tuple(xs) => {
                assert_eq!(xs.len(), 2);
                assert!(matches!(xs[0], Value::i64(1)));
                match &xs[1] {
                    Value::String(s) => assert_eq!(&**s, "x"),
                    other => panic!("expected Value::String; got {:?}", other),
                }
            }
            other => panic!("expected Value::Tuple; got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_mismatch_surfaces_path() {
        // Vector<i64> + first element is a String → mismatch at .[0].
        let t = TypeExpr::Parametric {
            head: "wat::core::Vector".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let err = coerce(&t, "[\"oops\" 2]").unwrap_err();
        assert_eq!(err.expected, ":wat::core::i64");
        assert_eq!(err.got, "String");
        assert_eq!(err.path, ".[0]");
    }

    #[test]
    fn arc170_1fi_coerce_top_level_mismatch_no_path() {
        let t = TypeExpr::Path(":wat::core::i64".into());
        let err = coerce(&t, "\"not an int\"").unwrap_err();
        assert_eq!(err.expected, ":wat::core::i64");
        assert_eq!(err.got, "String");
        assert_eq!(err.path, "");
    }

    #[test]
    fn arc170_1fi_coerce_result_ok() {
        // Arc 278 Stone A.0 — Result wire form is `#wat.core.Result/Ok [v]` (vector body).
        let t = TypeExpr::Parametric {
            head: "wat::core::Result".into(),
            args: vec![
                TypeExpr::Path(":wat::core::i64".into()),
                TypeExpr::Path(":wat::core::String".into()),
            ],
        };
        let v = coerce(&t, "#wat.core.Result/Ok [42]").unwrap();
        match v {
            Value::Result(r) => match &*r {
                Ok(Value::i64(42)) => {}
                other => panic!("expected Ok(i64 42); got {:?}", other),
            },
            other => panic!("expected Value::Result; got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_result_err() {
        // Arc 278 Stone A.0 — Result wire form is `#wat.core.Result/Err [e]` (vector body).
        let t = TypeExpr::Parametric {
            head: "wat::core::Result".into(),
            args: vec![
                TypeExpr::Path(":wat::core::i64".into()),
                TypeExpr::Path(":wat::core::String".into()),
            ],
        };
        let v = coerce(&t, "#wat.core.Result/Err [\"boom\"]").unwrap();
        match v {
            Value::Result(r) => match &*r {
                Err(Value::String(s)) => assert_eq!(&**s, "boom"),
                other => panic!("expected Err(String); got {:?}", other),
            },
            other => panic!("expected Value::Result; got {:?}", other),
        }
    }

    // ─── Arc 278 stone 0b — PersistentVector EDN round-trip ─────────────

    #[test]
    fn persistent_vector_edn_round_trip() {
        // Build a PersistentVector with three elements.
        let orig = Value::wat__core__PersistentVector(crate::value::pvec::PVec::from_vec(vec![
            Value::i64(10),
            Value::i64(20),
            Value::i64(30),
        ]));

        // Serialize → tagged EDN string.
        let s = value_to_edn_string_with(&orig, None);

        // Parse back.
        let back = edn_string_to_value(&s).expect("round-trip parse");

        // STOP-1 gate: the tag must reconstruct a PersistentVector, not collapse to a std Vec.
        assert!(
            matches!(back, Value::wat__core__PersistentVector(_)),
            "must round-trip to PersistentVector, not {back:?}"
        );
        // Value equality: same elements, same order.
        assert_eq!(back, orig, "EDN round-trip must preserve the vector");
    }

    // ─── WatAST renders as its form, not opaque-nil ────────────────────
    // A WatAST is, by definition, an EDN form — `watast_to_edn`/`edn_to_watast` are a total
    // bijection (a parsed s-expr IS edn). Rendering it as opaque-nil is lossy and unintuitive.
    // RED before the fix (rendered as an opaque-bucket-tagged nil); GREEN after (renders the form).
    #[test]
    fn watast_renders_as_its_form_not_opaque_nil() {
        let forms = crate::parser::parse_all_with_file("(:wat::core::< -5 0)", "<watast-render-probe>")
            .expect("parse the form");
        let ast = forms.into_iter().next().expect("one form");
        let v = Value::wat__WatAST(Arc::new(ast));
        let s = value_to_edn_string_with(&v, None);
        assert_eq!(s, "(:wat.core/< -5 0)", "a WatAST must render as its form (with operands), not opaque-nil");
    }

    // ─── Arc 278 stone 0a — PersistentMap EDN round-trip ───────────────

    #[test]
    fn persistent_map_edn_round_trip() {
        // Build a PersistentMap with two entries.
        let m = crate::value::pmap::PMap::from_pairs([
            (Value::String(Arc::new("a".to_string())), Value::i64(1)),
            (Value::String(Arc::new("b".to_string())), Value::i64(2)),
        ]);
        let pm = Value::wat__core__PersistentMap(m);

        // Serialize → tagged EDN string.
        let s = value_to_edn_string_with(&pm, None);

        // Parse back.
        let back = edn_string_to_value(&s).expect("round-trip parse");

        // STOP-1 gate: the tag must reconstruct a PersistentMap, not collapse to a std HashMap.
        assert!(
            matches!(back, Value::wat__core__PersistentMap(_)),
            "must round-trip to PersistentMap, not {back:?}"
        );
        // Value equality: same keys, same values.
        assert_eq!(back, pm, "EDN round-trip must preserve the map");
    }

    // ─── Arc 296 G′ gate row 3 ──────────────────────────────────────────

    /// A tagged enum variant must render with its DECLARED field names on the
    /// `write-json-natural` surface (`value_to_json_natural`'s Enum arm) — never the
    /// `field-N` positional fallback `enum_variant_field_names` used to fabricate.
    /// DESIGN-STONE-G-prime-the-enum-value-carries-its-own-names.md, gate row 3.
    #[test]
    fn arc296_gprime_tagged_variant_renders_declared_names_not_field_n() {
        use crate::types::{EnumDef, EnumVariant, Purity, TypeDef, TypeEnv};

        let mut types = TypeEnv::new();
        types
            .register(TypeDef::Enum(EnumDef {
                name: ":probe::Point".to_string(),
                type_params: vec![],
                purity: Purity::Pure,
                variants: vec![EnumVariant::Tagged {
                    name: "At".to_string(),
                    fields: vec![
                        ("x".into(), TypeExpr::Path(":wat::core::i64".into())),
                        ("y".into(), TypeExpr::Path(":wat::core::i64".into())),
                    ],
                }],
            }))
            .expect("register :probe::Point");
        let def = match types.get(":probe::Point") {
            Some(TypeDef::Enum(e)) => e,
            _ => unreachable!(),
        };
        let names = def.variant_names_arc("At").expect("At is Tagged");

        let ev = Value::Enum(Arc::new(crate::runtime::EnumValue {
            type_path: ":probe::Point".to_string(),
            variant_name: "At".to_string(),
            names,
            fields: vec![Value::i64(3), Value::i64(4)],
        }));

        let json = value_to_json_natural(&ev, Some(&types));
        let OwnedValue::Map(entries) = &json else {
            panic!("expected a JSON object, got {json:?}");
        };
        let keys: Vec<&str> = entries
            .iter()
            .map(|(k, _)| match k {
                OwnedValue::String(s) => s.as_ref(),
                other => panic!("non-string key: {other:?}"),
            })
            .collect();
        // Exact, not loose: the field-N fallback would produce "field-0"/"field-1" keys,
        // which this byte-identical comparison against the DECLARED x/y already excludes —
        // a separate `starts_with("field-")` check would be a redundant loose assertion.
        assert_eq!(
            keys,
            vec!["_type", "x", "y"],
            "declared field names x/y must appear verbatim, never field-0/field-1"
        );
    }

    // ─── Arc 294.k — tag_from_type_path / struct_tag_for: raise, don't fabricate ───
    //
    // Both functions used to fabricate a placeholder "no-home" namespace for a type
    // path with no `::`, and to erase the type's name to the literal word "unnamed"
    // under a placeholder "opaque" namespace when the split's namespace/name failed
    // `Tag::try_ns`'s validation. Both fallbacks raised nothing — a value could cross
    // with its identity silently lost. DESIGN-STONE-294.k: both fabrications become a
    // `panic!` that names the offending path.
    //
    // Downcast a `catch_unwind` panic payload to the message string, house pattern
    // from `tests/lint/no_inlined_wat_in_tests.rs` / `tests/lint/no_inlined_edn.rs`.
    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "<non-string panic payload>".to_string())
    }

    /// Row 3 (the load-bearing gate row): `tag_from_type_path` (encode side, bare
    /// `Tag`) and `struct_tag_for` (decode side, `(String, String)`) are one concept
    /// implemented twice — a pattern that has diverged three times already in this
    /// arc (294.j's `holon_to_watast` vs `from_holon_item`; task #102's
    /// `watast_to_holon` vs `to_holon_inner`). Feed both the same paths; assert
    /// identical `(ns, name)` on every success, and that both raise together.
    #[test]
    fn arc294k_tag_from_type_path_and_struct_tag_for_agree() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let paths = [
            // Real, observed corpus — DESIGN-STONE-294.k's measurement (every
            // `type_path` on the tree is `::`-separated).
            ":probe::Point",
            ":test::Token",
            ":wat::edn::Validation",
            ":wat::eval::StepResult",
            ":wat::kernel::RunResult",
            ":wat::kernel::LociDiedError",
            ":wat::sqlite::Cell",
            ":wat::io::IOReader::ReadFrameOutcome",
            // No `::` at all — the dead `.local` fallback's old input.
            ":NoNamespace",
            "NoNamespace",
            ":",
            "",
        ];

        for path in paths {
            let a = std::panic::catch_unwind(|| {
                let tag = tag_from_type_path(path);
                (tag.namespace().to_string(), tag.name().to_string())
            });
            let b = std::panic::catch_unwind(|| struct_tag_for(path));

            match (a, b) {
                (Ok(a), Ok(b)) => assert_eq!(
                    a, b,
                    "tag_from_type_path and struct_tag_for disagree on the SUCCESSFUL path \
                     {path:?}: {a:?} vs {b:?}"
                ),
                (Err(_), Err(_)) => {} // both raised — agreement
                (a, b) => panic!(
                    "tag_from_type_path and struct_tag_for DISAGREE on whether {path:?} has a \
                     derivable home: tag_from_type_path -> {:?}, struct_tag_for -> {:?}",
                    a.is_ok(),
                    b.is_ok()
                ),
            }
        }

        std::panic::set_hook(prev_hook);
    }

    /// Row 4/5 — a path with no derivable home RAISES on the encode side, and the
    /// message names the offending path (never a generic "invalid type path", which
    /// would reproduce the exact defect — an identity lost silently — one layer up).
    /// A kept negative control, not deleted after the write-up
    /// (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`).
    #[test]
    fn arc294k_tag_from_type_path_raises_and_names_the_path() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| tag_from_type_path(":NoNamespaceHere"));
        std::panic::set_hook(prev_hook);

        let msg = panic_message(result.expect_err(
            "tag_from_type_path must RAISE, not fabricate a placeholder no-home namespace, for \
             a path with no `::`",
        ));
        // Exact, not loose (`tests/lint/no_loose_string_assert.rs`) — the message is a
        // deterministic scalar this test itself authors, not a value that varies per run.
        assert_eq!(
            msg,
            "tag_from_type_path: type path \":NoNamespaceHere\" has no `::` namespace \
             separator — no derivable EDN home (fabricating a namespace would silently erase \
             this type's identity on the wire)"
        );
    }

    /// Row 4/5, decode side — same control for `struct_tag_for`, which must move
    /// together with `tag_from_type_path` (one concept, one change).
    #[test]
    fn arc294k_struct_tag_for_raises_and_names_the_path() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| struct_tag_for(":NoNamespaceHere"));
        std::panic::set_hook(prev_hook);

        let msg = panic_message(result.expect_err(
            "struct_tag_for must RAISE, not fabricate a placeholder no-home namespace, for a \
             path with no `::`",
        ));
        // Exact, not loose (`tests/lint/no_loose_string_assert.rs`) — same rationale as
        // `arc294k_tag_from_type_path_raises_and_names_the_path`.
        assert_eq!(
            msg,
            "struct_tag_for: type path \":NoNamespaceHere\" has no `::` namespace separator \
             — no derivable EDN home (fabricating a namespace would silently erase this \
             type's identity on the wire)"
        );
    }

    /// STOP-1 negative control: a path that DOES have a `::` separator but whose
    /// namespace/name fails `Tag::try_ns`'s validation (name starts with a digit) —
    /// the OTHER fallback this stone killed (the placeholder "opaque"/"unnamed" pair).
    /// Also raises, also names the path. `struct_tag_for` never validated this branch (it has no
    /// `try_ns` call at all — a pre-existing asymmetry with `tag_from_type_path`,
    /// not something this stone introduces), so this probe is encode-side only.
    #[test]
    fn arc294k_tag_from_type_path_raises_on_invalid_name_after_split() {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = std::panic::catch_unwind(|| tag_from_type_path(":wat::kernel::0bad"));
        std::panic::set_hook(prev_hook);

        let msg = panic_message(result.expect_err(
            "tag_from_type_path must RAISE, not fabricate the opaque/unnamed placeholder, when \
             the split namespace/name fails Tag::try_ns",
        ));
        // Exact, not loose (`tests/lint/no_loose_string_assert.rs`) — deterministic scalar.
        assert_eq!(
            msg,
            "tag_from_type_path: type path \":wat::kernel::0bad\" has no derivable EDN home \
             — namespace \"wat.kernel\" / name \"0bad\" rejected: first character must be \
             non-numeric (fabricating one would silently erase this type's identity on the \
             wire)"
        );
    }
}

#[cfg(test)]
mod next_complete_frame_negatives {
    //! Stone 259-killed — pure unit tests for `next_complete_frame` negative paths.
    //!
    //! These replace the integration tests that relied on the annihilated `print-raw'`
    //! verb. All four coverage axes are pure functions — no processes, no pipes.
    //!
    //! 1. over-cap, un-terminated    → TooLarge
    //! 2. over-cap, complete frame   → TooLarge (semantics B)
    //! 3. anti-smuggle two-value line → Frame + EDN decode fails
    //! 4. incomplete partial           → Incomplete

    use super::{next_complete_frame, FrameScan};

    /// A buffer larger than `max_bytes` with no newline → `TooLarge(buf.len())`.
    ///
    /// Exercises the `None` (no newline) branch in `next_complete_frame` where
    /// `buf.len() > max_bytes`. The exact length is returned so the caller can
    /// log the offending size.
    #[test]
    fn over_cap_unterminated_is_too_large() {
        let buf = vec![b'x'; 100];
        match next_complete_frame(&buf, 64) {
            FrameScan::TooLarge(n) => assert_eq!(
                n, 100,
                "TooLarge must carry the actual buffer length (100); got {n}"
            ),
            other => panic!(
                "over-cap un-terminated: expected TooLarge(100); got {other:?}"
            ),
        }
    }

    /// A complete, newline-terminated frame whose `end` exceeds `max_bytes` →
    /// `TooLarge(end)` (semantics B: reject oversized frames even when complete).
    ///
    /// A frame of 10 bytes + `\n` = 11 bytes; budget = 5 → `TooLarge(11)`.
    #[test]
    fn over_cap_complete_frame_is_too_large() {
        // "{:a 1}\n" = 7 bytes; budget = 5 → TooLarge(7).
        let buf = b"{:a 1}\n";
        match next_complete_frame(buf, 5) {
            FrameScan::TooLarge(n) => assert_eq!(
                n, 7,
                "TooLarge must carry end (7 = frame length incl. newline); got {n}"
            ),
            other => panic!(
                "over-cap complete frame: expected TooLarge(7); got {other:?}"
            ),
        }
    }

    /// Two EDN values smuggled on one physical line: `{:a 1} {:b 2}\n`.
    ///
    /// `next_complete_frame` MUST return `Frame` (the whole line up to and
    /// including the `\n`) — it does not split on the first value. The framer
    /// uses `edn_frame_status` to detect non-Incomplete prefixes; `{:a 1} {:b 2}`
    /// is `EdnFrameStatus::Malformed` (trailing content after the first complete
    /// map) → `FrameScan::Frame(end)`. The decode step then REJECTS it.
    ///
    /// The second assertion confirms that `wat_edn::parse_owned` refuses the
    /// smuggled content — no silent acceptance, no silent drop of `{:b 2}`.
    #[test]
    fn anti_smuggle_two_values_on_one_line_is_frame_but_decode_fails() {
        let buf = b"{:a 1} {:b 2}\n";
        let end = buf.len(); // 14 bytes incl. \n
        match next_complete_frame(buf, usize::MAX) {
            FrameScan::Frame(n) => assert_eq!(
                n, end,
                "anti-smuggle: Frame end must span the whole line ({end}); got {n}"
            ),
            other => panic!(
                "anti-smuggle: expected Frame({end}); got {other:?} — \
                 STOP-3: next_complete_frame does not return Frame for this input; \
                 report the actual FrameScan, do not rewrite the assertion"
            ),
        }
        // The framed content (newline stripped) must fail EDN decode.
        let content = std::str::from_utf8(&buf[..end - 1]).expect("valid UTF-8");
        assert!(
            wat_edn::parse_owned(content).is_err(),
            "anti-smuggle: EDN parse of '{content}' must fail (trailing content after first value); \
             the smuggled second value must be rejected, not silently accepted"
        );
    }

    /// A partial value with no newline and length under `max_bytes` → `Incomplete`.
    ///
    /// The caller must accumulate more bytes before a frame can be produced.
    #[test]
    fn incomplete_partial_is_incomplete() {
        let buf = b"{:a 1";
        match next_complete_frame(buf, usize::MAX) {
            FrameScan::Incomplete => {}
            other => panic!(
                "incomplete partial: expected Incomplete; got {other:?}"
            ),
        }
    }
}
