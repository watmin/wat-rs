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
//! | Unit / Option(None) | `nil` |
//! | bool | `true` / `false` |
//! | i64 / u8 | `Integer` |
//! | f64 (incl. NaN/Inf) | `Float` (sentinel-tagged for non-finite) |
//! | String | quoted EDN string |
//! | keyword | `Keyword` (namespace split at last `::`) |
//! | Vec | `Vector` |
//! | Tuple | `Vector` (no tuple distinction in EDN) |
//! | Option(Some(v)) | `v` (transparent) |
//! | Result(Ok(v)) | `Tagged #wat-edn.result/ok v` |
//! | Result(Err(e)) | `Tagged #wat-edn.result/err e` |
//! | HashMap | `Map` |
//! | HashSet | `Set` |
//! | Struct | `Tagged #ns/Type {:field-0 v0 :field-1 v1 ...}` |
//! | Enum | `Tagged #ns/Variant [v0 v1 ...]` (or just the tag if no fields) |
//! | HolonAST | Tagged per variant (Symbol/String/I64/F64/Bool/Atom/Bind/Bundle/Permute/Thermometer/Blend) |
//! | All other substrate handles | `Tagged #wat-edn.opaque/<TypeName> nil` |
//!
//! # Performance
//!
//! Walks the wat value tree once; constructs an `OwnedValue` tree in
//! memory; passes to wat-edn's `write` / `write_pretty` /
//! `to_json_string`. The intermediate tree is the cost; for typical
//! log-line sizes (a struct with ~5 fields) it's well under 1µs per
//! value.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, SymbolTable, Value};
use crate::span::Span;
use std::sync::Arc;
use wat_edn::{Keyword, OwnedValue, Tag};

/// Prefix `"<file>:<line>:<col>: "` when span is known; empty string
/// when unknown. Mirrors `src/macros.rs::span_prefix` and
/// `src/types.rs::span_prefix` exactly.
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() {
        String::new()
    } else {
        format!("{}: ", span)
    }
}

// ─── Public eval entry points ────────────────────────────────────

/// `(:wat::edn::write v)` → `:String`. Compact single-line EDN.
pub fn eval_edn_write(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::write(&edn))))
}

/// `(:wat::edn::write-pretty v)` → `:String`. Multi-line indented EDN.
pub fn eval_edn_write_pretty(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-pretty";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::write_pretty(&edn))))
}

/// `(:wat::edn::write-json v)` → `:String`. JSON via wat-edn's
/// round-trip-safe sentinel-tagged-object convention.
pub fn eval_edn_write_json(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-json";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::to_json_string(&edn))))
}

fn require_one_arg(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        // arc 138: no span — require_one_arg has no list_span; callers in runtime.rs don't pass it; cross-file broadening out of scope
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
            span: crate::span::Span::unknown(),
        });
    }
    eval(&args[0], env, sym)
}

/// `(:wat::edn::write-notag v)` → `:String`. Tagless EDN. Drops
/// the `#namespace/Type` wrapper from struct + enum-variant
/// renders, producing flat maps for structs and discriminator-
/// keyed maps for enum tagged variants. Keywords + Insts retain
/// their EDN-natural form (`:foo`, `#inst "..."`).
///
/// Lossy vs `:wat::edn::write` — natural-EDN rendering can't be
/// `read` back into the original wat value (no tags ⇒ no
/// reconstruction signal). For round-trip use the tagged form.
pub fn eval_edn_write_notag(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-notag";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn_notag(&v, sym.types().map(|a| a.as_ref()));
    Ok(Value::String(Arc::new(wat_edn::write(&edn))))
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-json-natural";
    let v = require_one_arg(OP, args, env, sym)?;
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
///     reconstruct `Value::Struct` with declared field names.
///   - Tagged + Vector body → look up `:<dotted-ns>` as Enum; find
///     variant `<name>`; reconstruct `Value::Enum` with the vector
///     elements as positional fields.
///   - Tagged + Nil body → enum unit-variant; same lookup as above.
///   - `#inst` (handled by wat-edn parser) → `Value::Instant`.
///   - Other tags → `EdnReadError::UnknownTag` panic; consumer sees
///     the path that failed.
pub fn eval_edn_read(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::read";
    let v = require_one_arg(OP, args, env, sym)?;
    let s = match &v {
        Value::String(s) => (**s).clone(),
        other => {
            // arc 138: no span — eval_edn_read receives evaluated Value via require_one_arg, no WatAST trace
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":String",
                got: other.type_name(),
                span: crate::span::Span::unknown(),
            });
        }
    };
    // arc 138: no span — s is a plain String extracted from Value; no WatAST span to thread
    let edn = wat_edn::parse_owned(&s).map_err(|e| RuntimeError::MalformedForm {
        head: OP.into(),
        reason: format!("EDN parse error: {e}"),
        span: crate::span::Span::unknown(),
    })?;
    edn_to_value(&edn, sym.types().map(|a| a.as_ref())).map_err(|e| {
        // arc 138: no span — edn_to_value errors on parsed EDN, no originating WatAST
        RuntimeError::MalformedForm {
            head: OP.into(),
            reason: e.to_string(),
            span: crate::span::Span::unknown(),
        }
    })
}

/// Errors surfaced by [`read_edn`] / [`edn_to_value`] when an EDN
/// document fails to coerce to a runtime [`Value`]. Substrate-
/// consumer crates (e.g. `wat-telemetry-sqlite`'s row reify) match
/// against these to surface diagnostic messages.
#[derive(Debug)]
pub enum EdnReadError {
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
        span: Span,
    },
    /// A substrate-reserved tag the bridge doesn't currently
    /// understand. `#inst` is handled by the underlying
    /// `wat_edn` parser; everything else lands here.
    UnsupportedTag(String, Span),
    /// No type registry was attached. The bridge needs the
    /// registry to interpret `#ns/Name` tags; without one,
    /// any tagged value fails. Pass `None` only when you know
    /// the EDN document contains no tagged values.
    NoTypeRegistry(Span),
    /// `#ns/Name {map}` referenced a key that isn't a declared
    /// field of the named struct.
    UnknownStructField {
        type_path: String,
        key: String,
        span: Span,
    },
    /// `#ns/Name [body]` or `#ns/Name nil` referenced a variant
    /// name that isn't declared on the named enum.
    EnumVariantNotFound {
        type_path: String,
        variant: String,
        span: Span,
    },
    /// Catch-all — the EDN value couldn't be coerced to a wat
    /// Value for the listed structural reason (e.g. unsupported
    /// `wat_edn::Value` variant like Symbol or BigInt, or a
    /// surface-level parse error wrapped here).
    Other(String, Span),
}

impl std::fmt::Display for EdnReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTag { ns, name, body_shape, span } => write!(
                f,
                "{}unknown tag #{ns}/{name} (body shape: {body_shape}); \
                 no matching struct or enum in the type registry",
                span_prefix(span)
            ),
            Self::UnsupportedTag(t, span) => {
                write!(f, "{}unsupported substrate tag #{t}", span_prefix(span))
            }
            Self::NoTypeRegistry(span) => write!(
                f,
                "{}no type registry attached to SymbolTable; arc 085 capability missing",
                span_prefix(span)
            ),
            Self::UnknownStructField { type_path, key, span } => write!(
                f,
                "{}struct {type_path} has no field named {key}",
                span_prefix(span)
            ),
            Self::EnumVariantNotFound { type_path, variant, span } => write!(
                f,
                "{}enum {type_path} has no variant named {variant}",
                span_prefix(span)
            ),
            Self::Other(s, span) => {
                write!(f, "{}{s}", span_prefix(span))
            }
        }
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
pub fn read_edn(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
) -> Result<Value, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_edn operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError::Other(format!("EDN parse error: {e}"), Span::unknown()))?;
    edn_to_value(&edn, types)
}

/// Bridge a parsed `wat_edn::OwnedValue` to a runtime [`Value`],
/// using `types` to interpret `#ns/Name` tags. Most consumers
/// want [`read_edn`] (parse + bridge in one call); reach for
/// this directly when you already have the parsed EDN tree (e.g.
/// when bridging multiple sub-expressions of one document).
pub fn edn_to_value(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    match edn {
        Edn::Nil => Ok(Value::Unit),
        Edn::Bool(b) => Ok(Value::bool(*b)),
        Edn::Integer(n) => Ok(Value::i64(*n)),
        Edn::Float(x) => Ok(Value::f64(*x)),
        Edn::String(s) => Ok(Value::String(Arc::new(s.to_string()))),
        Edn::Char(c) => Ok(Value::String(Arc::new(c.to_string()))),
        Edn::Keyword(k) => {
            let s = match k.namespace() {
                Some(ns) => format!(":{}::{}", ns.replace('.', "::"), k.name()),
                None => format!(":{}", k.name()),
            };
            Ok(Value::wat__core__keyword(Arc::new(s)))
        }
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        Edn::Symbol(_) => Err(EdnReadError::Other(
            "EDN Symbol — wat has no symbol value type".into(),
            Span::unknown(),
        )),
        Edn::BigInt(_) | Edn::BigDec(_) => Err(EdnReadError::Other(
            "EDN BigInt / BigDecimal — wat numeric tower is i64 + f64 only".into(),
            Span::unknown(),
        )),
        Edn::List(items) | Edn::Vector(items) => {
            let walked: Vec<Value> = items
                .iter()
                .map(|x| edn_to_value(x, types))
                .collect::<Result<_, _>>()?;
            Ok(Value::Vec(Arc::new(walked)))
        }
        Edn::Map(entries) => {
            // Generic HashMap — the no-tag map case. Walk keys + values.
            let mut backing = std::collections::HashMap::with_capacity(entries.len());
            for (k, v) in entries {
                let k_val = edn_to_value(k, types)?;
                let v_val = edn_to_value(v, types)?;
                let canonical =
                    crate::runtime::hashmap_key(":wat::edn::read", &k_val)
                        // arc 138: no span — hashmap_key error has no WatAST origin
                        .map_err(|e| EdnReadError::Other(format!(
                            "non-hashable map key: {e:?}"
                        ), Span::unknown()))?;
                backing.insert(canonical, (k_val, v_val));
            }
            Ok(Value::wat__std__HashMap(Arc::new(backing)))
        }
        Edn::Set(items) => {
            let mut backing = std::collections::HashMap::with_capacity(items.len());
            for x in items {
                let v_val = edn_to_value(x, types)?;
                let canonical =
                    crate::runtime::hashmap_key(":wat::edn::read", &v_val)
                        // arc 138: no span — hashmap_key error has no WatAST origin
                        .map_err(|e| EdnReadError::Other(format!(
                            "non-hashable set element: {e:?}"
                        ), Span::unknown()))?;
                backing.insert(canonical, v_val);
            }
            Ok(Value::wat__std__HashSet(Arc::new(backing)))
        }
        Edn::Inst(t) => Ok(Value::Instant(*t)),
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        Edn::Uuid(_) => Err(EdnReadError::Other(
            "EDN Uuid — wat has no UUID value type yet".into(),
            Span::unknown(),
        )),
        Edn::Tagged(tag, body) => tagged_to_value(tag, body, types),
    }
}

// ─── Natural / tagless renderers ──────────────────────────────────

/// Tagless EDN walker. Drops `#tag` wrappers from struct + enum
/// renders; emits flat maps for structs and discriminator-keyed
/// maps for enum tagged variants. Keywords/Insts retain EDN form.
pub fn value_to_edn_notag(
    v: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> OwnedValue {
    match v {
        // ── Struct: drop tag; body is the named-field map ───────
        Value::Struct(sv) => {
            let field_names: Vec<String> = match types.and_then(|t| t.get(&sv.type_name)) {
                Some(crate::types::TypeDef::Struct(def)) => {
                    def.fields.iter().map(|(name, _)| name.clone()).collect()
                }
                _ => (0..sv.fields.len()).map(|i| format!("field-{}", i)).collect(),
            };
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .fields
                .iter()
                .enumerate()
                .map(|(i, fv)| {
                    let key = field_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("field-{}", i));
                    (
                        OwnedValue::Keyword(Keyword::new(key)),
                        value_to_edn_notag(fv, types),
                    )
                })
                .collect();
            OwnedValue::Map(entries)
        }
        // ── Enum: fully-qualified variant as discriminator ──────
        // The _type value is a namespaced keyword `:<dotted-ns>/<Variant>`
        // (e.g. `:demo.Event/Buy`) — bare variant names like `:Buy`
        // are ambiguous across enums; the FQDN is the honest identity.
        Value::Enum(ev) => {
            let ns = type_path_to_namespace(&ev.type_path);
            let qualified_kw = make_qualified_keyword(&ns, &ev.variant_name);
            if ev.fields.is_empty() {
                // Unit variant — emit just the qualified keyword.
                qualified_kw
            } else {
                let field_names = enum_variant_field_names(&ev.type_path, &ev.variant_name, types);
                let mut entries: Vec<(OwnedValue, OwnedValue)> =
                    Vec::with_capacity(ev.fields.len() + 1);
                entries.push((
                    OwnedValue::Keyword(Keyword::new("_type")),
                    qualified_kw,
                ));
                for (i, fv) in ev.fields.iter().enumerate() {
                    let key = field_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("field-{}", i));
                    entries.push((
                        OwnedValue::Keyword(Keyword::new(key)),
                        value_to_edn_notag(fv, types),
                    ));
                }
                OwnedValue::Map(entries)
            }
        }
        // ── Recurse on collections ───────────────────────────────
        Value::Vec(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_notag(x, types)).collect())
        }
        Value::Tuple(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_notag(x, types)).collect())
        }
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.values()
                .map(|(k, v)| {
                    (
                        value_to_edn_notag(k, types),
                        value_to_edn_notag(v, types),
                    )
                })
                .collect(),
        ),
        Value::Option(opt) => match &**opt {
            None => OwnedValue::Nil,
            Some(inner) => value_to_edn_notag(inner, types),
        },
        Value::Result(r) => match &**r {
            // Result keeps its tag — it's a discriminated outcome,
            // dropping that loses the ok/err signal.
            Ok(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "ok"),
                Box::new(value_to_edn_notag(inner, types)),
            ),
            Err(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "err"),
                Box::new(value_to_edn_notag(inner, types)),
            ),
        },
        // HolonAST: render in natural form — primitive leaves
        // unwrap to their bare EDN equivalent; Atom drops its
        // wrapper. Composite operators (Bind, Bundle, Permute,
        // Thermometer, SlotMarker, Blend) keep their tags because
        // dropping them loses the operation's identity.
        Value::holon__HolonAST(h) => holon_ast_to_edn_notag(h),
        // ── Everything else: same as the tagged walker ───────────
        _ => value_to_edn_with(v, types),
    }
}

/// Natural-JSON walker. Same tagless transforms as `notag`, plus:
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
        Value::Struct(sv) => {
            let field_names: Vec<String> = match types.and_then(|t| t.get(&sv.type_name)) {
                Some(crate::types::TypeDef::Struct(def)) => {
                    def.fields.iter().map(|(name, _)| name.clone()).collect()
                }
                _ => (0..sv.fields.len()).map(|i| format!("field-{}", i)).collect(),
            };
            // Use String keys (plain strings — JSON-friendly).
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .fields
                .iter()
                .enumerate()
                .map(|(i, fv)| {
                    let key = field_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("field-{}", i));
                    (
                        OwnedValue::String(Cow::Owned(key)),
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
                let field_names = enum_variant_field_names(&ev.type_path, &ev.variant_name, types);
                let mut entries: Vec<(OwnedValue, OwnedValue)> =
                    Vec::with_capacity(ev.fields.len() + 1);
                entries.push((
                    OwnedValue::String(Cow::Owned("_type".into())),
                    OwnedValue::String(Cow::Owned(qualified)),
                ));
                for (i, fv) in ev.fields.iter().enumerate() {
                    let key = field_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("field-{}", i));
                    entries.push((
                        OwnedValue::String(Cow::Owned(key)),
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
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.values()
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
        Value::Option(opt) => match &**opt {
            None => OwnedValue::Nil,
            Some(inner) => value_to_json_natural(inner, types),
        },
        // Fallback: use the tagged walker. Tagged Result variants
        // round-trip via wat-edn's natural sentinel encoding.
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

/// Build a namespaced EDN keyword, falling back to a non-namespaced
/// one if the namespace fails wat-edn's first-character validation
/// (variant names always validate; the namespace might not if the
/// type path is unusual). The fallback is `<ns>/<name>` shoved into
/// the name slot — visually identical but loses the namespace
/// distinction at the wat-edn API layer.
fn make_qualified_keyword(ns: &str, name: &str) -> OwnedValue {
    match Keyword::try_ns(ns, name) {
        Ok(kw) => OwnedValue::Keyword(kw),
        Err(_) => match Keyword::try_new(format!("{ns}/{name}")) {
            Ok(kw) => OwnedValue::Keyword(kw),
            Err(_) => OwnedValue::String(std::borrow::Cow::Owned(format!(":{ns}/{name}"))),
        },
    }
}

fn enum_variant_field_names(
    type_path: &str,
    variant_name: &str,
    types: Option<&crate::types::TypeEnv>,
) -> Vec<String> {
    let Some(types) = types else {
        return vec![];
    };
    let Some(crate::types::TypeDef::Enum(def)) = types.get(type_path) else {
        return vec![];
    };
    for variant in &def.variants {
        if let crate::types::EnumVariant::Tagged { name, fields } = variant {
            if name == variant_name {
                return fields.iter().map(|(n, _)| n.clone()).collect();
            }
        }
    }
    vec![]
}

fn strip_keyword_colon(k: &str) -> String {
    // Wat keywords are stored with leading `:` and `::` separators.
    // For natural JSON we want a plain string.
    let stripped = k.strip_prefix(':').unwrap_or(k);
    // Convert `::` separators to `.` so JSON readers see a familiar
    // dotted-namespace form (e.g. `:wat::time::Instant` → `wat.time.Instant`).
    stripped.replace("::", ".")
}

fn tagged_to_value(
    tag: &Tag,
    body: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    let ns = tag.namespace();
    let name = tag.name();

    // Substrate-emitted special tags. We don't reconstruct opaque
    // handles (Sender, ProgramHandle, etc.) — they have no
    // serializable identity. Treat as errors.
    if ns == "wat-edn.opaque" {
        // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
        return Err(EdnReadError::UnsupportedTag(format!("{ns}/{name}"), Span::unknown()));
    }
    if ns == "wat-edn.holon" {
        // Arc 093 — substrate-internal HolonAST round-trip.
        // `holon_ast_to_edn` produces these tags on the write
        // side; lift back to a Value::holon__HolonAST here so
        // EDN containing tagged HolonASTs reads cleanly.
        let ast = edn_holon_tag_to_ast(name, body)?;
        return Ok(Value::holon__HolonAST(ast));
    }
    if ns == "wat-edn.result" {
        // Tagged Result — body is the inner value.
        let inner = edn_to_value(body, types)?;
        return Ok(Value::Result(Arc::new(match name {
            "ok" => Ok(inner),
            "err" => Err(inner),
            // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
            _ => return Err(EdnReadError::UnsupportedTag(format!("{ns}/{name}"), Span::unknown())),
        })));
    }

    // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
    let types = types.ok_or(EdnReadError::NoTypeRegistry(Span::unknown()))?;

    // Body shape disambiguates struct vs enum.
    match body {
        Edn::Map(entries) => reconstruct_struct(ns, name, entries, types),
        Edn::Vector(items) => reconstruct_enum_tagged(ns, name, items, types),
        Edn::Nil => reconstruct_enum_unit(ns, name, types),
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
            Err(EdnReadError::UnknownTag {
                ns: ns.to_string(),
                name: name.to_string(),
                body_shape: shape,
                span: Span::unknown(),
            })
        }
    }
}

fn ns_to_wat_path(ns: &str, name: &str) -> String {
    format!(":{}::{}", ns.replace('.', "::"), name)
}

fn ns_to_enum_path(ns: &str) -> String {
    format!(":{}", ns.replace('.', "::"))
}

fn reconstruct_struct(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Struct(d)) => d,
        _ => {
            // arc 138: no span — reconstruct_struct operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError::UnknownTag {
                ns: ns.to_string(),
                name: name.to_string(),
                body_shape: "map",
                span: Span::unknown(),
            });
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
            EdnReadError::UnknownStructField {
                type_path: path.clone(),
                key: fname.clone(),
                span: Span::unknown(),
            }
        })?;
        let inner = edn_to_value(fv, Some(types))?;
        let wrapped = rewrap_option_field(fty, inner);
        fields.push(wrapped);
    }
    Ok(Value::Struct(Arc::new(crate::runtime::StructValue {
        type_name: path,
        fields,
    })))
}

/// Arc 113 slice 3 — when a declared field type is `Option<T>` but
/// the EDN-bridged value isn't already a `Value::Option`, wrap it.
/// `Value::Unit` (Nil round-trip) → `None`; anything else → `Some`.
/// Already-Option values pass through. Non-Option declared types
/// pass through unchanged.
fn rewrap_option_field(fty: &crate::types::TypeExpr, v: Value) -> Value {
    let is_option = matches!(
        fty,
        crate::types::TypeExpr::Parametric { head, .. } if head == "Option"
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
) -> Result<Value, EdnReadError> {
    let path = ns_to_enum_path(ns);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Enum(d)) => d,
        _ => {
            // arc 138: no span — reconstruct_enum_tagged operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError::UnknownTag {
                ns: ns.to_string(),
                name: variant_name.to_string(),
                body_shape: "vector",
                span: Span::unknown(),
            });
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
            EdnReadError::EnumVariantNotFound {
                type_path: path.clone(),
                variant: variant_name.to_string(),
                span: Span::unknown(),
            }
        })?;
    // Arc 113 slice 3 — Option-aware field wrapping (same shape as
    // reconstruct_struct). Variant field types come from
    // `EnumVariant::Tagged.fields`; bridge each item, then rewrap
    // Option layers wat-edn dropped on the wire.
    let declared_fields: &[(String, crate::types::TypeExpr)] = match variant {
        crate::types::EnumVariant::Tagged { fields, .. } => fields.as_slice(),
        crate::types::EnumVariant::Unit(_) => &[],
    };
    let mut fields: Vec<Value> = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        let inner = edn_to_value(item, Some(types))?;
        let wrapped = match declared_fields.get(idx) {
            Some((_, fty)) => rewrap_option_field(fty, inner),
            None => inner,
        };
        fields.push(wrapped);
    }
    Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
        type_path: path,
        variant_name: variant_name.to_string(),
        fields,
    })))
}

fn reconstruct_enum_unit(
    ns: &str,
    variant_name: &str,
    types: &crate::types::TypeEnv,
) -> Result<Value, EdnReadError> {
    let path = ns_to_enum_path(ns);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Enum(d)) => d,
        _ => {
            // arc 138: no span — reconstruct_enum_unit operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError::UnknownTag {
                ns: ns.to_string(),
                name: variant_name.to_string(),
                body_shape: "nil",
                span: Span::unknown(),
            });
        }
    };
    let _variant = def
        .variants
        .iter()
        .find(|v| match v {
            crate::types::EnumVariant::Unit(n) => n == variant_name,
            crate::types::EnumVariant::Tagged { name, .. } => name == variant_name,
        })
        .ok_or_else(|| {
            // arc 138: no span — reconstruct_enum_unit operates on parsed OwnedValue, no WatAST
            EdnReadError::EnumVariantNotFound {
                type_path: path.clone(),
                variant: variant_name.to_string(),
                span: Span::unknown(),
            }
        })?;
    Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
        type_path: path,
        variant_name: variant_name.to_string(),
        fields: vec![],
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
        Value::Option(opt) => match &**opt {
            None => OwnedValue::Nil,
            Some(inner) => value_to_edn_with(inner, types),
        },
        Value::Result(r) => match &**r {
            Ok(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "ok"),
                Box::new(value_to_edn_with(inner, types)),
            ),
            Err(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "err"),
                Box::new(value_to_edn_with(inner, types)),
            ),
        },

        // ── Compound containers ──────────────────────────────────
        Value::Vec(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_with(x, types)).collect())
        }
        Value::Tuple(xs) => {
            OwnedValue::Vector(xs.iter().map(|x| value_to_edn_with(x, types)).collect())
        }
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.values()
                .map(|(k, v)| (value_to_edn_with(k, types), value_to_edn_with(v, types)))
                .collect(),
        ),
        Value::wat__std__HashSet(s) => OwnedValue::Set(
            s.values().map(|x| value_to_edn_with(x, types)).collect(),
        ),

        // ── User-declared struct / enum ──────────────────────────
        Value::Struct(sv) => {
            let tag = tag_from_type_path(&sv.type_name);
            // Look up the StructDef so we can name fields.
            let field_names: Vec<String> = match types.and_then(|t| t.get(&sv.type_name)) {
                Some(crate::types::TypeDef::Struct(def)) => {
                    def.fields.iter().map(|(name, _ty)| name.clone()).collect()
                }
                _ => (0..sv.fields.len()).map(|i| format!("field-{}", i)).collect(),
            };
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .fields
                .iter()
                .enumerate()
                .map(|(i, fv)| {
                    let key = field_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("field-{}", i));
                    (
                        OwnedValue::Keyword(Keyword::new(key)),
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
                // Tagless variant — render as just the tag with nil payload.
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Nil))
            } else {
                let payload: Vec<OwnedValue> = ev
                    .fields
                    .iter()
                    .map(|x| value_to_edn_with(x, types))
                    .collect();
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Vector(payload)))
            }
        }

        // ── Substrate compound values — opaque or structural ─────
        Value::holon__HolonAST(h) => holon_ast_to_edn(h),
        Value::Vector(vec) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Vector"),
            Box::new(OwnedValue::Map(vec![(
                OwnedValue::Keyword(Keyword::new("dim")),
                OwnedValue::Integer(vec.dimensions() as i64),
            )])),
        ),

        // ── Opaque substrate handles — type-tagged nil ───────────
        Value::wat__WatAST(_) => opaque_nil("wat-edn.opaque", "WatAST"),
        Value::wat__core__lambda(_) => opaque_nil("wat-edn.opaque", "lambda"),
        Value::crossbeam_channel__Sender(_) => opaque_nil("wat-edn.opaque", "Sender"),
        Value::crossbeam_channel__Receiver(_) => opaque_nil("wat-edn.opaque", "Receiver"),
        Value::wat__kernel__ProgramHandle(_) => opaque_nil("wat-edn.opaque", "ProgramHandle"),
        Value::wat__kernel__HandlePool { name, .. } => OwnedValue::Tagged(
            Tag::ns("wat-edn.opaque", "HandlePool"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(
                (**name).clone(),
            ))),
        ),
        Value::wat__kernel__ChildHandle(_) => opaque_nil("wat-edn.opaque", "ChildHandle"),
        Value::io__IOReader(_) => opaque_nil("wat-edn.opaque", "IOReader"),
        Value::io__IOWriter(_) => opaque_nil("wat-edn.opaque", "IOWriter"),
        Value::RustOpaque(inner) => OwnedValue::Tagged(
            Tag::ns("wat-edn.opaque", "RustOpaque"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(
                inner.type_path.to_string(),
            ))),
        ),
        Value::OnlineSubspace(_) => opaque_nil("wat-edn.opaque", "OnlineSubspace"),
        Value::Reckoner(_) => opaque_nil("wat-edn.opaque", "Reckoner"),
        Value::Engram(_) => opaque_nil("wat-edn.opaque", "Engram"),
        Value::EngramLibrary(_) => opaque_nil("wat-edn.opaque", "EngramLibrary"),
        Value::Hologram(_) => opaque_nil("wat-edn.opaque", "Hologram"),
        Value::Instant(t) => OwnedValue::Inst(*t),
        Value::Duration(ns) => OwnedValue::Integer(*ns),
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
fn keyword_from_wat_path(k: &str) -> OwnedValue {
    let stripped = k.strip_prefix(':').unwrap_or(k);
    if let Some(idx) = stripped.rfind("::") {
        let ns = stripped[..idx].replace("::", ".");
        let name = &stripped[idx + 2..];
        match Keyword::try_ns(&ns, name) {
            Ok(kw) => OwnedValue::Keyword(kw),
            // Fallback to a string if the keyword fails wat-edn's
            // first-character validation. Better to render than to
            // panic on a logger call.
            Err(_) => OwnedValue::String(std::borrow::Cow::Owned(k.to_string())),
        }
    } else {
        match Keyword::try_new(stripped) {
            Ok(kw) => OwnedValue::Keyword(kw),
            Err(_) => OwnedValue::String(std::borrow::Cow::Owned(k.to_string())),
        }
    }
}

/// Build a tag from a type path like `:trading::cache::L1`. Drops the
/// leading colon (if present) and translates `::` to `.` for the
/// namespace; the last segment becomes the tag name.
fn tag_from_type_path(path: &str) -> Tag {
    let stripped = path.strip_prefix(':').unwrap_or(path);
    if let Some(idx) = stripped.rfind("::") {
        let ns = stripped[..idx].replace("::", ".");
        let name = &stripped[idx + 2..];
        Tag::try_ns(&ns, name).unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
    } else {
        // No namespace separator — fabricate a "wat-edn.local" namespace
        // so wat-edn's spec-required namespace constraint is met.
        Tag::try_ns("wat-edn.local", stripped)
            .unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
    }
}

/// Build a tagged-nil for an opaque handle.
fn opaque_nil(ns: &str, name: &str) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(ns, name), Box::new(OwnedValue::Nil))
}

/// Render a HolonAST as a tagged EDN value. Primitives unwrap to
/// their EDN equivalent inside the tag; composites recurse.
fn holon_ast_to_edn(h: &holon::HolonAST) -> OwnedValue {
    use holon::HolonAST;
    match h {
        HolonAST::Symbol(s) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Symbol"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(s.to_string()))),
        ),
        HolonAST::String(s) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "String"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(s.to_string()))),
        ),
        HolonAST::I64(n) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "I64"),
            Box::new(OwnedValue::Integer(*n)),
        ),
        HolonAST::F64(x) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "F64"),
            Box::new(OwnedValue::Float(*x)),
        ),
        HolonAST::Bool(b) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Bool"),
            Box::new(OwnedValue::Bool(*b)),
        ),
        HolonAST::Atom(inner) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Atom"),
            Box::new(holon_ast_to_edn(inner)),
        ),
        HolonAST::Bind(role, filler) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Bind"),
            Box::new(OwnedValue::Vector(vec![
                holon_ast_to_edn(role),
                holon_ast_to_edn(filler),
            ])),
        ),
        HolonAST::Bundle(xs) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Bundle"),
            Box::new(OwnedValue::Vector(
                xs.iter().map(holon_ast_to_edn).collect(),
            )),
        ),
        HolonAST::Permute(child, k) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Permute"),
            Box::new(OwnedValue::Vector(vec![
                holon_ast_to_edn(child),
                OwnedValue::Integer(*k as i64),
            ])),
        ),
        HolonAST::Thermometer { value, min, max } => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Thermometer"),
            Box::new(OwnedValue::Map(vec![
                (
                    OwnedValue::Keyword(Keyword::new("value")),
                    OwnedValue::Float(*value),
                ),
                (
                    OwnedValue::Keyword(Keyword::new("min")),
                    OwnedValue::Float(*min),
                ),
                (
                    OwnedValue::Keyword(Keyword::new("max")),
                    OwnedValue::Float(*max),
                ),
            ])),
        ),
        HolonAST::Blend(a, b, w1, w2) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Blend"),
            Box::new(OwnedValue::Vector(vec![
                holon_ast_to_edn(a),
                holon_ast_to_edn(b),
                OwnedValue::Float(*w1),
                OwnedValue::Float(*w2),
            ])),
        ),
        HolonAST::SlotMarker { min, max } => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "SlotMarker"),
            Box::new(OwnedValue::Map(vec![
                (
                    OwnedValue::Keyword(Keyword::new("min")),
                    OwnedValue::Float(*min),
                ),
                (
                    OwnedValue::Keyword(Keyword::new("max")),
                    OwnedValue::Float(*max),
                ),
            ])),
        ),
    }
}

/// Inverse of [`holon_ast_to_edn`] — reconstruct a HolonAST from
/// a round-trip-safe tagged EDN form (`#wat-edn.holon/*`). The
/// arc-091/092 read counterpart that the original write side
/// shipped without; arc 093's reader-cursor needs this to lift
/// `:wat::edn::Tagged` columns back to their original HolonAST.
///
/// The body shape disambiguates per-tag:
/// - leaves (`Symbol`/`String`/`I64`/`F64`/`Bool`) carry a single
///   primitive payload;
/// - `Atom` carries a single nested HolonAST EDN form;
/// - `Bind` / `Permute` / `Bundle` / `Blend` carry a Vector of
///   children (with the right arity per variant);
/// - `Thermometer` / `SlotMarker` carry a Map keyed on field
///   names (`:value`, `:min`, `:max`).
fn edn_to_holon_ast(edn: &OwnedValue) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    match edn {
        OwnedValue::Tagged(tag, body) if tag.namespace() == "wat-edn.holon" => {
            edn_holon_tag_to_ast(tag.name(), body)
        }
        // arc 138: no span — edn_to_holon_ast walks parsed OwnedValue, no WatAST in scope
        _ => Err(EdnReadError::Other(
            "expected #wat-edn.holon/* tagged form for HolonAST round-trip; \
             use edn_to_holon_ast_natural for the tagless read"
                .into(),
            Span::unknown(),
        )),
    }
}

/// Tagless-friendly HolonAST read — primitives unwrap from their
/// bare EDN form (mirroring [`holon_ast_to_edn_notag`]);
/// composite operators still need their `#wat-edn.holon/*` tag
/// (the natural form keeps these tags because dropping them
/// would lose the operation's identity). Used by arc-093's
/// reader cursor for `:wat::edn::NoTag` columns where the writer
/// stripped tags from primitive HolonASTs.
fn edn_to_holon_ast_natural(edn: &OwnedValue) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    use holon::HolonAST;
    match edn {
        // Tagged composite ops — same path as the strict round-trip read.
        OwnedValue::Tagged(tag, body) if tag.namespace() == "wat-edn.holon" => {
            edn_holon_tag_to_ast(tag.name(), body)
        }
        // Bare primitives — best-effort lift to the matching leaf.
        OwnedValue::Keyword(k) => {
            // Mirror `keyword_from_wat_path`'s inverse — wat
            // colon-prefixed `:foo::bar` lowers to EDN keyword
            // `foo/bar`; here we go back to `:foo::bar`.
            let s = match k.namespace() {
                Some(ns) => format!(":{}::{}", ns.replace('.', "::"), k.name()),
                None => format!(":{}", k.name()),
            };
            Ok(Arc::new(HolonAST::Symbol(Arc::from(s))))
        }
        OwnedValue::String(s) => {
            Ok(Arc::new(HolonAST::String(Arc::from(s.as_ref()))))
        }
        OwnedValue::Integer(n) => Ok(Arc::new(HolonAST::I64(*n))),
        OwnedValue::Float(x) => Ok(Arc::new(HolonAST::F64(*x))),
        OwnedValue::Bool(b) => Ok(Arc::new(HolonAST::Bool(*b))),
        // Anything else (Map, Vector, Tagged with non-holon ns,
        // Nil, Char, Symbol, BigInt, BigDec, Inst, Set) doesn't
        // correspond to a HolonAST shape in the natural form.
        // arc 138: no span — edn_to_holon_ast_natural walks parsed OwnedValue, no WatAST
        _ => Err(EdnReadError::Other(format!(
            "natural-form HolonAST read can't lift this EDN shape; \
             expected primitive leaf or #wat-edn.holon/* tagged composite"
        ), Span::unknown())),
    }
}

/// Inner switch — given a tag-name from the `wat-edn.holon`
/// namespace and its body, reconstruct the HolonAST variant.
/// Mirrors [`holon_ast_to_edn`] arm-for-arm.
fn edn_holon_tag_to_ast(
    name: &str,
    body: &OwnedValue,
) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    use holon::HolonAST;
    match (name, body) {
        ("Symbol", OwnedValue::String(s)) => {
            Ok(Arc::new(HolonAST::Symbol(Arc::from(s.as_ref()))))
        }
        ("String", OwnedValue::String(s)) => {
            Ok(Arc::new(HolonAST::String(Arc::from(s.as_ref()))))
        }
        ("I64", OwnedValue::Integer(n)) => Ok(Arc::new(HolonAST::I64(*n))),
        ("F64", OwnedValue::Float(x)) => Ok(Arc::new(HolonAST::F64(*x))),
        ("Bool", OwnedValue::Bool(b)) => Ok(Arc::new(HolonAST::Bool(*b))),
        ("Atom", inner) => {
            let child = edn_to_holon_ast(inner)?;
            Ok(Arc::new(HolonAST::Atom(child)))
        }
        ("Bind", OwnedValue::Vector(items)) if items.len() == 2 => {
            let role = edn_to_holon_ast(&items[0])?;
            let filler = edn_to_holon_ast(&items[1])?;
            Ok(Arc::new(HolonAST::Bind(role, filler)))
        }
        ("Bundle", OwnedValue::Vector(items)) => {
            let xs: Vec<holon::HolonAST> = items
                .iter()
                .map(|x| edn_to_holon_ast(x).map(|a| (*a).clone()))
                .collect::<Result<_, _>>()?;
            Ok(Arc::new(HolonAST::Bundle(Arc::new(xs))))
        }
        ("Permute", OwnedValue::Vector(items)) if items.len() == 2 => {
            let child = edn_to_holon_ast(&items[0])?;
            let k = match &items[1] {
                OwnedValue::Integer(n) => *n as i32,
                // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
                _ => {
                    return Err(EdnReadError::Other(
                        "#wat-edn.holon/Permute body[1] must be an Integer (k)"
                            .into(),
                        Span::unknown(),
                    ));
                }
            };
            Ok(Arc::new(HolonAST::Permute(child, k)))
        }
        ("Thermometer", OwnedValue::Map(entries)) => {
            let (value, min, max) = read_three_floats(entries, "Thermometer")?;
            Ok(Arc::new(HolonAST::Thermometer { value, min, max }))
        }
        ("Blend", OwnedValue::Vector(items)) if items.len() == 4 => {
            let a = edn_to_holon_ast(&items[0])?;
            let b = edn_to_holon_ast(&items[1])?;
            let w1 = match &items[2] {
                OwnedValue::Float(x) => *x,
                OwnedValue::Integer(n) => *n as f64,
                // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
                _ => {
                    return Err(EdnReadError::Other(
                        "#wat-edn.holon/Blend body[2] must be a Float (w1)".into(),
                        Span::unknown(),
                    ));
                }
            };
            let w2 = match &items[3] {
                OwnedValue::Float(x) => *x,
                OwnedValue::Integer(n) => *n as f64,
                // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
                _ => {
                    return Err(EdnReadError::Other(
                        "#wat-edn.holon/Blend body[3] must be a Float (w2)".into(),
                        Span::unknown(),
                    ));
                }
            };
            Ok(Arc::new(HolonAST::Blend(a, b, w1, w2)))
        }
        ("SlotMarker", OwnedValue::Map(entries)) => {
            // SlotMarker has just min/max — read_three_floats expects
            // value/min/max; specialized read here.
            let mut min = None;
            let mut max = None;
            for (k, v) in entries {
                let key = match k {
                    OwnedValue::Keyword(kw) => kw.name().to_string(),
                    _ => continue,
                };
                let f = match v {
                    OwnedValue::Float(x) => *x,
                    OwnedValue::Integer(n) => *n as f64,
                    _ => continue,
                };
                match key.as_str() {
                    "min" => min = Some(f),
                    "max" => max = Some(f),
                    _ => {}
                }
            }
            // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
            let min = min.ok_or_else(|| {
                EdnReadError::Other("#wat-edn.holon/SlotMarker missing :min".into(), Span::unknown())
            })?;
            let max = max.ok_or_else(|| {
                EdnReadError::Other("#wat-edn.holon/SlotMarker missing :max".into(), Span::unknown())
            })?;
            Ok(Arc::new(HolonAST::SlotMarker { min, max }))
        }
        // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
        (other, _) => Err(EdnReadError::Other(format!(
            "#wat-edn.holon/{other}: unrecognized tag or body shape"
        ), Span::unknown())),
    }
}

/// Pull `value` / `min` / `max` Float entries from a `Thermometer`
/// body Map. Substrate writer always emits these three keys; if
/// any are missing or non-numeric we surface a parse error.
fn read_three_floats(
    entries: &[(OwnedValue, OwnedValue)],
    op: &str,
) -> Result<(f64, f64, f64), EdnReadError> {
    let mut value = None;
    let mut min = None;
    let mut max = None;
    for (k, v) in entries {
        let key = match k {
            OwnedValue::Keyword(kw) => kw.name().to_string(),
            _ => continue,
        };
        let f = match v {
            OwnedValue::Float(x) => *x,
            OwnedValue::Integer(n) => *n as f64,
            _ => continue,
        };
        match key.as_str() {
            "value" => value = Some(f),
            "min" => min = Some(f),
            "max" => max = Some(f),
            _ => {}
        }
    }
    // arc 138: no span — read_three_floats operates on parsed OwnedValue Map entries, no WatAST
    let value = value
        .ok_or_else(|| EdnReadError::Other(format!("#wat-edn.holon/{op} missing :value"), Span::unknown()))?;
    let min = min
        .ok_or_else(|| EdnReadError::Other(format!("#wat-edn.holon/{op} missing :min"), Span::unknown()))?;
    let max = max
        .ok_or_else(|| EdnReadError::Other(format!("#wat-edn.holon/{op} missing :max"), Span::unknown()))?;
    Ok((value, min, max))
}

/// Public arc-093: parse an EDN string and reconstruct a
/// `HolonAST` from its round-trip-safe tagged form. Inverse of
/// the substrate's `:wat::edn::write` for HolonAST values; what
/// the wat-telemetry-sqlite cursor calls per `:wat::edn::Tagged`
/// column.
pub fn read_holon_ast_tagged(s: &str) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_holon_ast_tagged operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError::Other(format!("EDN parse error: {e}"), Span::unknown()))?;
    edn_to_holon_ast(&edn)
}

/// Public arc-093: parse an EDN string and reconstruct a
/// `HolonAST` from its tagless-friendly natural form (primitives
/// unwrap; composite ops keep their `#wat-edn.holon/*` tag).
/// What the wat-telemetry-sqlite cursor calls per
/// `:wat::edn::NoTag` column.
pub fn read_holon_ast_natural(s: &str) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_holon_ast_natural operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError::Other(format!("EDN parse error: {e}"), Span::unknown()))?;
    edn_to_holon_ast_natural(&edn)
}

/// Render a HolonAST as a tagless EDN value — primitives unwrap to
/// their bare EDN form; `Atom` drops its wrapper. Composite operators
/// (Bind, Bundle, Permute, Thermometer, SlotMarker, Blend) keep their
/// `#wat-edn.holon/...` tag because dropping it would lose the
/// operation's identity (Bind vs Bundle vs Blend all carry vectors of
/// children — only the tag tells them apart).
///
/// Used by `value_to_edn_notag` (arc 091) when a `:wat::edn::NoTag`
/// field of a struct is a HolonAST. Indexed-column queries match
/// against the natural form: `:metrics` instead of
/// `#wat-edn.holon/Symbol "metrics"`; `"request_count"` instead of
/// `#wat-edn.holon/String "request_count"`.
fn holon_ast_to_edn_notag(h: &holon::HolonAST) -> OwnedValue {
    use holon::HolonAST;
    match h {
        // HolonAST::Symbol stores the colon-prefixed form for keywords
        // (e.g. ":asset") — same convention runtime.rs:6865 keys off of
        // for the to-watast round-trip. Pass s through directly; the
        // older `format!(":{}", s)` here was double-prefixing the colon
        // and producing `::asset`-shaped EDN output.
        HolonAST::Symbol(s) => keyword_from_wat_path(s),
        HolonAST::String(s) => OwnedValue::String(std::borrow::Cow::Owned(s.to_string())),
        HolonAST::I64(n) => OwnedValue::Integer(*n),
        HolonAST::F64(x) => OwnedValue::Float(*x),
        HolonAST::Bool(b) => OwnedValue::Bool(*b),
        HolonAST::Atom(inner) => holon_ast_to_edn_notag(inner),
        // Composites: keep the tag so the operation's identity
        // survives the strip — same rule that keeps :Result tagged.
        _ => holon_ast_to_edn(h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Arc 138 canary ─────────────────────────────────────────────────

    #[test]
    fn arc138_edn_read_error_message_carries_span() {
        // Trigger NoTypeRegistry — call read_edn with a tagged value but
        // no type registry. The error variant carries Span::unknown()
        // (pattern E — raw EDN string has no WatAST origin). The Display
        // arm prefixes span_prefix, which returns "" for unknown spans.
        // This canary verifies the variant structurally carries a span and
        // that Display still renders without panic.
        let result = read_edn("#unknown/Type {}", None);
        let err = result.unwrap_err();
        let rendered = format!("{}", err);
        assert!(
            matches!(err, EdnReadError::NoTypeRegistry(_)),
            "expected NoTypeRegistry, got: {:?}",
            err
        );
        assert!(
            rendered.contains("no type registry"),
            "expected NoTypeRegistry message; got: {}",
            rendered
        );
    }
}
