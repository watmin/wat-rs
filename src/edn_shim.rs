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
use crate::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::value::{AggregateValue, HolonForm};
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
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    eval(&args[0], env, sym).map(|tv| tv.value_owned())
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
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write-notag";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
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
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
        }
    };
    let edn = wat_edn::parse_owned(&s).map_err(|e| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("EDN parse error: {e}")
    } })?;
    // Arc 233 Stone 233.2.c — wrap result in Tracked with RuntimeBuilt provenance
    // so that errors flowing from edn::read-produced Values surface the producer origin.
    let result = edn_to_value(&edn, sym.types().map(|a| a.as_ref())).map_err(|e| {
        RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: e.to_string()
        } }
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
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            } });
        }
    };
    let forms = crate::parser::parse_all_with_file(&s, "<read-string>").map_err(|e| {
        RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("parse error: {e}"),
        } }
    })?;
    let ast = WatAST::List(forms, crate::span::Span::unknown());
    let value = Value::wat__WatAST(std::sync::Arc::new(ast));
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
/// what the general `edn::write` is NOT for forms: `value_to_edn` renders a
/// `wat__WatAST` as opaque-nil (an AST is opaque to general EDN serialization);
/// `write-forms` serializes the AST faithfully — so `read-string → transform →
/// write-forms` is the wat-to-wat fixer's full read→rewrite→write cycle, all in
/// wat's own primitives.
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
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            } });
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

/// `(:wat::core::ast->children <ast>)` — arc 251 Stone 251.5a-iii (the bridge).
///
/// The AST↔walkable bridge: decompose a `:wat::WatAST` node into a
/// `Vector<:wat::WatAST>` of its children — the SAME walkable shape `:wat::core::forms`
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
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            } });
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
/// carrying `children` (a `Vector<:wat::WatAST>`, as `ast->children` yields) as its
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
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(), expected: 2, got: args.len(),
        } });
    }
    let template_v = eval(&args[0], env, sym)?.value_owned();
    let children_v = eval(&args[1], env, sym)?.value_owned();
    // template must be a forms-value
    let template: &WatAST = match &template_v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        } }),
    };
    // children must be a Vec of forms-values; unwrap each to WatAST
    let child_vals: &Vec<Value> = match &children_v {
        Value::Vec(v) => v.as_ref(),
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::Vector<:wat::WatAST>",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        } }),
    };
    let mut kids: Vec<WatAST> = Vec::with_capacity(child_vals.len());
    for cv in child_vals.iter() {
        match cv {
            Value::wat__WatAST(a) => kids.push(a.as_ref().clone()),
            other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(), expected: ":wat::WatAST (child)",
                got: Box::new(crate::runtime::ValueSnapshot::of(other)),
            } }),
        }
    }
    // rebuild the SAME KIND as the template, preserving its span
    let rebuilt: WatAST = match template {
        WatAST::List(_, span) => WatAST::List(kids, span.clone()),
        WatAST::Vector(_, span) => WatAST::Vector(kids, span.clone()),
        WatAST::Set(_, span) => WatAST::Set(kids, span.clone()),
        WatAST::Map(_, span) => {
            if kids.len() % 2 != 0 {
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("Map rebuild needs an even child count (k/v interleaved); got {}", kids.len()),
                } });
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
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("leaf node has no children; cannot rebuild with {} child(ren)", kids.len()),
                } });
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let kind = match ast {
        WatAST::IntLit(..) => "int",
        WatAST::FloatLit(..) => "float",
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let name: String = match ast {
        WatAST::Symbol(ident, _) => ident.as_str().to_string(),
        WatAST::Keyword(s, _) => s.clone(),
        // Arc 279 — format macro needs the string content from a StringLit node.
        // "Does a macro need it?" → YES: format extracts the template text at expand time.
        // ast-name on a StringLit returns the string VALUE (unquoted content), matching the
        // natural meaning of "name" for literal nodes alongside Symbol/Keyword.
        WatAST::StringLit(s, _) => s.clone(),
        _ => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "ast-name requires a Symbol, Keyword, or StringLit node".to_string(),
        } }),
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let span = ast.span();
    #[allow(clippy::mutable_key_type)]
    let mut map: std::collections::HashMap<Value, Value> = std::collections::HashMap::new();
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":line".to_string())),
        Value::i64(span.end_line),
    );
    map.insert(
        Value::wat__core__keyword(std::sync::Arc::new(":col".to_string())),
        Value::i64(span.end_col),
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let node = WatAST::Symbol(Identifier::bare(s), crate::span::Span::unknown());
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let ident = Identifier::bare(s).add_scope(crate::scope::fresh_scope());
    let node = WatAST::Symbol(ident, crate::span::Span::unknown());
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
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::core::String", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    if !s.starts_with(':') {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("keyword-node requires a ':'-prefixed string; got {s:?}"),
        } });
    }
    let node = WatAST::Keyword(s, crate::span::Span::unknown());
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
            _ => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "keyword/to-symbol requires a Keyword node".to_string(),
            } }),
        },
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let symbol_name = wat_keyword_to_clojure_symbol(&kw).ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "not a convertible call-head/reference keyword (bare data keyword or namespace-prefix marker): {kw:?}"
            ),
        },
    })?;
    let node = WatAST::Symbol(Identifier::bare(symbol_name), crate::span::Span::unknown());
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
}

/// Arc 251 type-position rendering — convert a closed [`crate::types::TypeExpr`] into a
/// faithful WatAST node for the type FORM surface:
///
/// 4-way discriminator (Path; Parametric head mirrors it):
/// 1. core FQDN (`wat::core::X`) — flat reserved `wat.type/X`.
/// 2. bare legacy primitive (`:i64`, `:String`, ...) — `wat.type/X`.
/// 3. user/library type (has `::`, not core) — namespace-preserving (`wat.holon/HolonAST`).
/// 4. type-var (no `::`, not a primitive) — bare symbol (`T`, `K`, `V`).
/// - `Parametric{head,args}`: same 4-way ladder on head; recurse on args.
/// - `Fn{args,ret}`: `Vector([…args, Keyword(":->"), ret])`.
/// - `Tuple(items)`: `List([Symbol("wat.type/Tuple"), …rendered-items])`. The empty `:()`
///   renders as `(wat.type/Tuple)` — a distinct zero-arity product, NOT unit (`nil` is wat's
///   unit, Rust's `()`; wat's `()` empty list is distinct from `nil`).
/// - `Var`: synthetic — NEVER produced by parsing source (the `TypeExpr` doc guarantees it).
///
/// Fallible: the two unmodeled shapes (a malformed trailing-`::` path, and a bare/higher-kinded
/// Parametric head like `(Stream …)`/`(T …)`) return a clean `Err` — NEVER a panic. This renderer
/// backs the runtime verb `keyword/to-type-form` AND the corpus drive; both shapes are reachable
/// (`parse_type_expr` accepts `:foo::` and `:Stream<i64>`), so a panic would crash wat / the drive.
pub(crate) fn type_expr_to_clojure_form(t: &crate::types::TypeExpr) -> Result<WatAST, String> {
    use crate::types::TypeExpr;
    let unk = crate::span::Span::unknown();
    Ok(match t {
        TypeExpr::Path(s) => {
            // 4-way ladder: core FQDN > bare primitive > user type (::) > type-var.
            let body = s.strip_prefix(':').unwrap_or(s);
            if let Some(tail) = body.strip_prefix("wat::core::") {
                // Case 1: core FQDN -> flat wat.type/ namespace.
                WatAST::Symbol(Identifier::bare(format!("wat.type/{tail}")), unk)
            } else if crate::check::BARE_PRIMITIVES.iter().any(|(bare, _)| *bare == format!(":{body}").as_str()) {
                // Case 2: bare legacy primitive (:i64, :String, ...) -> wat.type/{body}.
                WatAST::Symbol(Identifier::bare(format!("wat.type/{body}")), unk)
            } else if body.contains("::") {
                // Case 3: user/library type -> namespace-preserving. `None` only on a malformed
                // trailing-`::` path (e.g. `:foo::`) -> clean error, never panic.
                let sym = wat_keyword_to_clojure_symbol(&format!(":{body}")).ok_or_else(|| {
                    format!("cannot render type `:{body}` to a faithful form (malformed namespaced path — trailing `::` or empty segment)")
                })?;
                WatAST::Symbol(Identifier::bare(sym), unk)
            } else {
                // Case 4: type-var -- stays as a bare symbol (T, K, V, ...).
                WatAST::Symbol(Identifier::bare(body.to_string()), unk)
            }
        }
        TypeExpr::Parametric { head, args } => {
            // head is stored WITHOUT a leading colon (e.g. "wat::core::Vector").
            // 4-way ladder mirrors Path.
            let sym = if let Some(tail) = head.strip_prefix("wat::core::") {
                // Case 1: core FQDN -> flat wat.type/ namespace.
                format!("wat.type/{tail}")
            } else if let Some((_bare, fqdn)) = crate::check::BARE_CONTAINER_HEADS.iter().find(|(bare, _)| *bare == head.as_str()) {
                // Case 2: bare container head (Option, Vec, ...) -> use canonical FQDN's last segment.
                // Note: Vec -> wat::core::Vector (rename), so we use the FQDN tail, not `head`.
                let tail = fqdn.rsplit("::").next().unwrap();
                format!("wat.type/{tail}")
            } else if head.contains("::") {
                // Case 3: user/library type -> namespace-preserving.
                wat_keyword_to_clojure_symbol(&format!(":{head}")).ok_or_else(|| {
                    format!("cannot render parametric head `:{head}` (malformed namespaced path)")
                })?
            } else {
                // Case 4: bare/higher-kinded head (`(Stream …)`, `(T …)`) — not in the model.
                // Clean error (the source should use the FQDN form), never panic.
                return Err(format!(
                    "cannot render parametric type with bare head `{head}` — not a core container and not FQDN; \
                     use the fully-qualified type name (bare/higher-kinded heads are unsupported)"
                ));
            };
            let mut items = vec![WatAST::Symbol(Identifier::bare(sym), unk.clone())];
            for a in args {
                items.push(type_expr_to_clojure_form(a)?);
            }
            WatAST::List(items, unk)
        }
        TypeExpr::Fn { args, ret } => {
            let mut items: Vec<WatAST> = Vec::with_capacity(args.len() + 2);
            for a in args {
                items.push(type_expr_to_clojure_form(a)?);
            }
            items.push(WatAST::Keyword(":->".into(), unk.clone()));
            items.push(type_expr_to_clojure_form(ret)?);
            WatAST::Vector(items, unk)
        }
        TypeExpr::Tuple(items) => {
            // Faithful form `(wat.type/Tuple …items)`. Empty `:()` → `(wat.type/Tuple)`, a
            // distinct zero-arity product (NOT unit — `nil` is wat's unit).
            let mut elems = vec![WatAST::Symbol(Identifier::bare("wat.type/Tuple".to_string()), unk.clone())];
            for it in items {
                elems.push(type_expr_to_clojure_form(it)?);
            }
            WatAST::List(elems, unk)
        }
        // Var is synthetic — NEVER produced by parsing source (the TypeExpr doc guarantees it),
        // so this verb (which only ever sees parsed-from-source types) cannot reach it.
        TypeExpr::Var(_) => unreachable!("type_expr_to_clojure_form: Var is never produced by parsing source"),
    })
}

/// `(:wat::core::keyword/to-type-form <keyword-node>)` — arc 251 type-position rendering.
/// Convert an old rust-scheme TYPE keyword (`:wat::core::Vector<wat::core::i64>`) into the
/// faithful-Clojure type FORM (`(wat.type/Vector wat.type/i64)`). Parses the keyword string
/// via the EXISTING type parser ([`crate::types::parse_type_expr_with_span`] → `TypeExpr`),
/// then renders the closed `TypeExpr` enum via [`type_expr_to_clojure_form`].
pub fn eval_keyword_to_type_form(
    args: &[WatAST],
    list_span: &crate::span::Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::keyword/to-type-form";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let kw: String = match &v {
        Value::wat__WatAST(a) => match a.as_ref() {
            WatAST::Keyword(s, _) => s.clone(),
            _ => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "keyword/to-type-form requires a Keyword node".to_string(),
            } }),
        },
        other => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(), expected: ":wat::WatAST", got: Box::new(crate::runtime::ValueSnapshot::of(other)) } }),
    };
    let te = crate::types::parse_type_expr(&kw).map_err(|e| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("type-keyword parse failed: {:?}", e.kind),
        },
    })?;
    let node = type_expr_to_clojure_form(&te).map_err(|reason| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm { head: OP.into(), reason },
    })?;
    Ok(crate::value::TrackedValue::new(
        Value::wat__WatAST(std::sync::Arc::new(node)),
        crate::value::Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() },
    ))
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
    // purity wall at wire-peer PRODUCERS (peer-pair', socket-pair', connect',
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
pub fn read_edn(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
) -> Result<Value, EdnReadError> {
    // General (untrusted) decode — capability tags are REFUSED (allow_caps = false).
    read_edn_caps(s, types, false)
}

/// Arc 272 6a-i — the capability-aware decode worker. PRIVATE by design: when `allow_caps` is true,
/// portable `wat-edn.cap` tags reconstruct into live capabilities. There is intentionally NO public
/// way to pass `allow_caps = true` — the only caller that may is [`decode_trusted_wire`], the single
/// audited door. This is what makes "mint a capability from an untrusted decode" UNREPRESENTABLE:
/// general code holds no flag to flip and no fn to reach (extirpare top rung; ocap transfer-only).
fn read_edn_caps(
    s: &str,
    types: Option<&crate::types::TypeEnv>,
    allow_caps: bool,
) -> Result<Value, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_edn operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })?;
    edn_to_value_caps(&edn, types, allow_caps)
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
}

/// Accumulate physical lines from `next_line` until the buffer forms a
/// complete EDN value, then return it as a `FramedRead::Frame`.
///
/// Each call to `next_line(span)` must return:
/// - `Ok(Some(line))` — one line WITHOUT its trailing `\n`
/// - `Ok(None)` — EOF
/// - `Err(e)` — a read error (treated as EOF / disconnect)
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
    F: FnMut(Span) -> Result<Option<String>, RuntimeError>,
{
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match next_line(span.clone()) {
            Ok(Some(line)) => {
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
            Ok(None) | Err(_) => {
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
pub fn edn_to_value(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
) -> Result<Value, EdnReadError> {
    // Arc 272 6a-i gating — the GENERAL decode path REFUSES portable-capability (`wat-edn.cap`)
    // tags. Object-capability rule: a capability is obtained only by being handed it on a trusted
    // channel, NEVER forged from parsed data. The trusted peer wire opts in via the `_caps` worker
    // with `allow_caps = true` (see `read_edn_caps` / `edn_string_to_value_trusted`).
    edn_to_value_caps(edn, types, false)
}

#[allow(clippy::mutable_key_type)]
fn edn_to_value_caps(
    edn: &OwnedValue,
    types: Option<&crate::types::TypeEnv>,
    allow_caps: bool,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    match edn {
        Edn::Nil => Ok(Value::Unit),
        Edn::Bool(b) => Ok(Value::bool(*b)),
        Edn::Integer(n) => Ok(Value::i64(*n)),
        Edn::Float(x) => Ok(Value::f64(*x)),
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
        Edn::Symbol(_) => Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("EDN Symbol — wat has no symbol value type".into()) }),
        Edn::BigInt(_) | Edn::BigDec(_) => Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("EDN BigInt / BigDecimal — wat numeric tower is i64 + f64 only".into()) }),
        // Arc 220 Stone 220.4 — EDN list `(...)` → `Value::wat__core__List` (preserves
        // the parens-vs-brackets distinction for faithful Clojure round-trips).
        // Previously both List and Vector collapsed to Vec (lossy).
        Edn::List(items) => {
            let walked: std::collections::LinkedList<Value> = items
                .iter()
                .map(|x| edn_to_value_caps(x, types, allow_caps))
                .collect::<Result<_, _>>()?;
            Ok(Value::wat__core__List(Arc::new(walked)))
        }
        Edn::Vector(items) => {
            let walked: Vec<Value> = items
                .iter()
                .map(|x| edn_to_value_caps(x, types, allow_caps))
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
                let k_val = edn_to_value_caps(k, types, allow_caps)?;
                let v_val = edn_to_value_caps(v, types, allow_caps)?;
                if !crate::runtime::value_is_key_hashable(&k_val) {
                    return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("non-hashable map key: {}", k_val.type_name())) });
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
                let v_val = edn_to_value_caps(x, types, allow_caps)?;
                backing.insert(v_val);
            }
            Ok(Value::wat__std__HashSet(Arc::new(backing)))
        }
        Edn::Inst(t) => Ok(Value::Instant(*t)),
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        // Arc 207 slice 2: `#uuid "..."` EDN reader literal → typed `:wat::core::Uuid`.
        // `uuid::Uuid` is `Copy`; mirrors `Edn::Inst(t) → Value::Instant(*t)` pattern.
        Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u)),
        Edn::Tagged(tag, body) => tagged_to_value(tag, body, types, allow_caps),
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

/// Coerce an already-parsed EDN tree to a runtime [`Value`] whose
/// type matches the caller's declared `target` annotation.
///
/// Arc 170 slice 1f-ι — the load-bearing piece of the EDN-only
/// `(:wat::kernel::readln -> :T)` contract.
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
/// | `:wat::core::Option<T>` | `Nil` → `None`; else recurse to `Some(T)` | enum variant |
/// | `:wat::core::Result<T,E>` | `Tagged #wat-edn.result/{ok|err}` | recurse on payload |
/// | user `Struct` | `Tagged #ns/Name {map}` | recurse per field |
/// | user `Enum` (Unit variant) | `Tagged #ns/Variant nil` | enum variant |
/// | user `Enum` (Tagged variant) | `Tagged #ns/Variant [items]` | recurse per field |
/// | `:wat::holon::HolonAST` | any | call [`edn_to_holon_ast_natural`] / tagged path |
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
    edn_to_typed_value_inner(target, edn, types)
}

fn edn_to_typed_value_inner(
    target: &crate::types::TypeExpr,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
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
                        return edn_to_typed_value_inner(&a.expr, edn, types);
                    }
                    crate::types::TypeDef::Newtype(n) => {
                        return edn_to_typed_value_inner(&n.inner, edn, types);
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
            // Universal top (arc 278 R7): UP is free — ANY EDN value IS a
            // `:wat::core::Value`. Decode structurally via the untyped bridge
            // (no concrete-type coercion), so a heterogeneous value (e.g. the
            // `metadata-of` map) reads back. This makes `Value` SYMMETRIC: it is
            // `EdnRepresentable` (write side) and now an EDN coerce target (read
            // side) — closing the write-but-not-read asymmetry. `edn_to_value`
            // honours `types` so `#ns/Variant` enum tags rebuild as `Value::Enum`.
            ":wat::core::Value" => edn_to_value(edn, types).map_err(|e| EdnCoerceError {
                expected: ":wat::core::Value".into(),
                got: format!("{e}"),
                path: String::new(),
            }),
            ":wat::holon::HolonAST" => {
                // Tagged round-trip OR natural-form lift to a leaf —
                // mirrors `edn_shim`'s two-mode reader.
                let ast = match edn {
                    Edn::Tagged(tag, _) if tag.namespace() == "wat-edn.holon" => {
                        edn_to_holon_ast(edn).map_err(|e| EdnCoerceError {
                            expected: ":wat::holon::HolonAST".into(),
                            got: format!("HolonAST decode error: {e}"),
                            path: String::new(),
                        })?
                    }
                    _ => edn_to_holon_ast_natural(edn).map_err(|e| EdnCoerceError {
                        expected: ":wat::holon::HolonAST".into(),
                        got: format!("HolonAST decode error: {e}"),
                        path: String::new(),
                    })?,
                };
                Ok(Value::holon__HolonAST(ast))
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
                    Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => {
                        coerce_struct_path(p, a, edn, types)
                    }
                    Some(crate::types::TypeDef::Enum(def)) => {
                        coerce_enum_path(p, def, edn, types)
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
                            let v = edn_to_typed_value_inner(elem_ty, item, types)
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
                            let v = edn_to_typed_value_inner(elem_ty, item, types)
                                .map_err(|e| e.at(&format!(".[{}]", i)))?;
                            walked.push_back(v);
                        }
                        Ok(Value::wat__core__List(Arc::new(walked)))
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            "wat::core::Option" => {
                let inner_ty = args.first().ok_or_else(|| mismatch(target, edn))?;
                match edn {
                    Edn::Nil => Ok(Value::Option(Arc::new(None))),
                    other => {
                        let inner = edn_to_typed_value_inner(inner_ty, other, types)
                            .map_err(|e| e.at(".some"))?;
                        Ok(Value::Option(Arc::new(Some(inner))))
                    }
                }
            }
            "wat::core::Result" => {
                if args.len() != 2 {
                    return Err(mismatch(target, edn));
                }
                let ok_ty = &args[0];
                let err_ty = &args[1];
                match edn {
                    Edn::Tagged(tag, body) if tag.namespace() == "wat-edn.result" => {
                        match tag.name() {
                            "ok" => {
                                let v = edn_to_typed_value_inner(ok_ty, body, types)
                                    .map_err(|e| e.at(".ok"))?;
                                Ok(Value::Result(Arc::new(Ok(v))))
                            }
                            "err" => {
                                let v = edn_to_typed_value_inner(err_ty, body, types)
                                    .map_err(|e| e.at(".err"))?;
                                Ok(Value::Result(Arc::new(Err(v))))
                            }
                            _ => Err(mismatch(target, edn)),
                        }
                    }
                    other => Err(mismatch(target, other)),
                }
            }
            "wat::core::HashMap" | "wat::core::HashSet" => {
                // Not currently supported as a readln target; the
                // wire form has no typed-K coercion path yet.
                Err(EdnCoerceError {
                    expected: crate::check::format_type(target),
                    got: "(coercion of HashMap/HashSet not yet supported)".into(),
                    path: String::new(),
                })
            }
            _ => {
                // Parametric user type — strip `<...>` to look up the
                // base declaration; coerce against the base shape.
                let path = format!(":{}", head);
                let env = types.ok_or_else(|| mismatch(target, edn))?;
                match env.get(&path) {
                    // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
                    Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => {
                        coerce_struct_path(&path, a, edn, types)
                    }
                    Some(crate::types::TypeDef::Enum(def)) => {
                        coerce_enum_path(&path, def, edn, types)
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
                        let v = edn_to_typed_value_inner(elem_ty, item, types)
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
fn coerce_struct_path(
    type_path: &str,
    def: &crate::types::AggregateDef,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
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
        let v = edn_to_typed_value_inner(fty, fv, types)
            .map_err(|e| e.at(&format!(".{}", fname)))?;
        fields.push(v);
    }
    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        type_path.trim_start_matches(':').to_string(),
        fields,
    ))))
}

fn coerce_enum_path(
    type_path: &str,
    def: &crate::types::EnumDef,
    edn: &wat_edn::OwnedValue,
    types: Option<&crate::types::TypeEnv>,
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
            // Unit variant body must be Nil.
            match body {
                Edn::Nil => Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
                    type_path: type_path.to_string(),
                    variant_name: tag_name,
                    fields: vec![],
                }))),
                other => Err(EdnCoerceError {
                    expected: format!("{}::{} (unit)", type_path, tag_name),
                    got: format!("Tagged-body {}", edn_shape_name(other)),
                    path: String::new(),
                }),
            }
        }
        crate::types::EnumVariant::Tagged { fields, .. } => {
            // Tagged variant body must be Vector matching arity.
            // Exception: zero-field tagged variants (declared as `(VariantName)` with
            // no payload fields) are serialized with a Nil body by `value_to_edn_with`
            // (because `EnumValue.fields.is_empty()` is true at runtime regardless of
            // whether the TypeDef says Unit or Tagged). Accept Nil as equivalent to
            // an empty vector for the zero-field case so the round-trip is coherent.
            let empty_slice: &[wat_edn::OwnedValue] = &[];
            let items: &[wat_edn::OwnedValue] = match body {
                Edn::Vector(items) | Edn::List(items) => items.as_slice(),
                Edn::Nil if fields.is_empty() => empty_slice,
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
                let v = edn_to_typed_value_inner(fty, item, types)
                    .map_err(|e| e.at(&format!(".{}", fname)))?;
                let _ = i; // path uses field name, index reserved for future
                walked.push(v);
            }
            Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
                type_path: type_path.to_string(),
                variant_name: tag_name,
                fields: walked,
            })))
        }
    }
}

/// Compute the EDN tag namespace + name for a struct's wire form.
/// Mirrors `tag_from_type_path` (file-local helper) but extracted
/// for the coercion side.
fn struct_tag_for(type_path: &str) -> (String, String) {
    let stripped = type_path.strip_prefix(':').unwrap_or(type_path);
    if let Some(idx) = stripped.rfind("::") {
        let ns = stripped[..idx].replace("::", ".");
        let name = stripped[idx + 2..].to_string();
        (ns, name)
    } else {
        ("wat-edn.local".into(), stripped.to_string())
    }
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

/// Tagless EDN walker. Drops `#tag` wrappers from struct + enum
/// renders; emits flat maps for structs and discriminator-keyed
/// maps for enum tagged variants. Keywords/Insts retain EDN form.
pub fn value_to_edn_notag(
    v: &Value,
    types: Option<&crate::types::TypeEnv>,
) -> OwnedValue {
    match v {
        // ── Struct: drop tag; body is the named-field map ───────
        Value::Aggregate(sv) if sv.holder == crate::types::Holder::Struct => {
            // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
            let type_key = format!(":{}", sv.class);
            let field_names: Vec<String> = match types.and_then(|t| t.get(&type_key)) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => {
                    a.fields.iter().map(|(name, _)| name.clone()).collect()
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
        // Stone 216.5c — iterate m.iter() for (k, v) directly (native HashMap<Value, Value>).
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.iter()
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
        Value::Aggregate(sv) if sv.holder == crate::types::Holder::Struct => {
            // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
            let type_key = format!(":{}", sv.class);
            let field_names: Vec<String> = match types.and_then(|t| t.get(&type_key)) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => {
                    a.fields.iter().map(|(name, _)| name.clone()).collect()
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
    allow_caps: bool,
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    let ns = tag.namespace();
    let name = tag.name();

    // Arc 272 6a-i — PORTABLE CAPABILITY tags. Sibling to `wat-edn.opaque` (which refuses): these
    // ARE reconstructable — but ONLY off a TRUSTED channel (`allow_caps`, set by the peer wire). On
    // the general decode path (`:wat::edn::read`, config, any parsed data) a `wat-edn.cap` tag is
    // REFUSED exactly like an opaque: an object-capability is obtained by being handed it over a
    // channel, never forged from data (ocap unforgeability + transfer-only). Checked BEFORE the
    // opaque refusal below.
    if ns == "wat-edn.cap" {
        if allow_caps {
            // Arc 272 6c.2 — record-based codecs (SocketAddressWire) need the type registry.
            // The trusted peer wire always provides types (decode_trusted_wire is always called
            // with sym.types()); None here is a programming error, surfaced as a decode failure.
            let t = types.ok_or_else(|| EdnReadError {
                span: Span::unknown(),
                kind: EdnReadErrorKind::NoTypeRegistry,
            })?;
            return crate::capability::decode_capability(name, body, t);
        }
        return Err(EdnReadError {
            span: Span::unknown(),
            kind: EdnReadErrorKind::UnsupportedTag(format!(
                "{ns}/{name} (capability tags reconstruct only off the trusted peer wire, never from parsed data)"
            )),
        });
    }

    // Substrate-emitted special tags. We don't reconstruct opaque
    // handles (Sender, ProgramHandle, etc.) — they have no
    // serializable identity. Treat as errors.
    if ns == "wat-edn.opaque" {
        // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
        return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnsupportedTag(format!("{ns}/{name}")) });
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
            _ => return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnsupportedTag(format!("{ns}/{name}")) }),
        })));
    }

    // Arc-278-0a — `#wat.core/PersistentMap {…}` tagged literal → PersistentMap.
    // Round-trip identity: a tagged form reads back as wat__core__PersistentMap (never
    // conflated with std HashMap which reads from untagged `{…}`). Body must be a Map.
    if ns == "wat.core" && name == "PersistentMap" {
        use wat_edn::Value as Edn;
        let entries = match body {
            Edn::Map(e) => e,
            _ => return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnsupportedTag(
                format!("wat.core/PersistentMap body must be a map, got non-map")
            ) }),
        };
        let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
        for (k, v) in entries {
            let k_val = edn_to_value_caps(k, types, allow_caps)?;
            let v_val = edn_to_value_caps(v, types, allow_caps)?;
            if !crate::runtime::value_is_key_hashable(&k_val) {
                return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("non-hashable PersistentMap key: {}", k_val.type_name())) });
            }
            pm = pm.insert(k_val, v_val);
        }
        return Ok(Value::wat__core__PersistentMap(pm));
    }

    // Arc-278-0b — `#wat.core/PersistentVector [...]` tagged literal → PersistentVector.
    // Round-trip identity: a bare `[…]` reads back as std Vec; the tagged form reads back
    // as PersistentVector (distinct identity per the DESIGN contract). Body must be a Vector.
    if ns == "wat.core" && name == "PersistentVector" {
        use wat_edn::Value as Edn;
        let items = match body {
            Edn::Vector(xs) => xs,
            _ => return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnsupportedTag(
                format!("wat.core/PersistentVector body must be a vector, got non-vector")
            ) }),
        };
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for item in items {
            let val = edn_to_value_caps(item, types, allow_caps)?;
            pv = pv.push_back(val);
        }
        return Ok(Value::wat__core__PersistentVector(pv));
    }

    // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
    let types = types.ok_or(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::NoTypeRegistry })?;

    // Body shape disambiguates struct vs enum.
    // Arc 293.2b: For Map bodies, resolve the TypeDef to route:
    //   Aggregate(kind!=Struct) → reconstruct_record,
    //   Aggregate(kind==Struct) or unknown → reconstruct_struct (returns UnknownTag on miss).
    match body {
        Edn::Map(entries) => {
            let path = ns_to_wat_path(ns, name);
            match types.get(&path) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.holder != crate::types::Holder::Struct => {
                    reconstruct_record(ns, name, entries, types, allow_caps)
                }
                _ => reconstruct_struct(ns, name, entries, types, allow_caps),
            }
        }
        Edn::Vector(items) => reconstruct_enum_tagged(ns, name, items, types, allow_caps),
        Edn::Nil => reconstruct_enum_unit(ns, name, types),
        // Arc 234 Stone 234.7b — holon-tagged body: a #wat-edn.holon/* tagged value
        // under a class tag. If the class resolves to a record Aggregate (kind!=Struct),
        // this is a holon record (encoded by the 234.7b encode arm as hologram-as-edn).
        // Base records have Edn::Map bodies (handled above) — these are distinct.
        Edn::Tagged(inner_tag, _) if inner_tag.namespace() == "wat-edn.holon" => {
            let path = ns_to_wat_path(ns, name);
            match types.get(&path) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.holder != crate::types::Holder::Struct => {
                    reconstruct_holon_record(ns, name, body, types)
                }
                _ => {
                    // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
                    Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: name.to_string(), body_shape: "tagged-holon" } })
                }
            }
        }
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
            Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: name.to_string(), body_shape: shape } })
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
    let segments: Vec<&str> = body.split("::").collect();
    // `body` contains "::" and has no trailing "::", so there are ≥2 non-empty segments.
    let (final_seg, ns_head) = segments.split_last()?;
    let mut ns_parts: Vec<&str> = ns_head.to_vec();
    let name: &str = match final_seg.find('/') {
        // `Type/method` — fold `Type` into the namespace; the method is the name.
        Some(idx) if idx > 0 => {
            ns_parts.push(&final_seg[..idx]);
            &final_seg[idx + 1..]
        }
        // A bare `/` (division → name `/`) or no slash: the final segment IS the name.
        _ => final_seg,
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
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => a,
        _ => {
            // arc 138: no span — reconstruct_struct operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: name.to_string(), body_shape: "map" } });
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
            EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownStructField { type_path: path.clone(), key: fname.clone() } }
        })?;
        let inner = edn_to_value_caps(fv, Some(types), allow_caps)?;
        let wrapped = rewrap_option_field(fty, inner);
        fields.push(wrapped);
    }
    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        path.trim_start_matches(':').to_string(),
        fields,
    ))))
}

/// Arc 234 Stone 234.7a — Decode a base-record tagged-map back to `Value::Aggregate(holder=Record)`.
///
/// Arc 293.2b: uses `AggregateDef` (kind=Record|HolonRecord) instead of the annihilated
/// `RecordDef`. Fields are always-typed (D2), so `rewrap_option_field` applies.
fn reconstruct_record(
    ns: &str,
    name: &str,
    entries: &[(OwnedValue, OwnedValue)],
    types: &crate::types::TypeEnv,
    allow_caps: bool,
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    // Arc 293.2b — record aggregates (kind != Struct) replace TypeDef::Record.
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.holder != crate::types::Holder::Struct => a,
        _ => {
            return Err(EdnReadError {
                span: Span::unknown(),
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
            span: Span::unknown(),
            kind: EdnReadErrorKind::UnknownStructField {
                type_path: path.clone(),
                key: fname.clone(),
            },
        })?;
        let inner = edn_to_value_caps(fv, Some(types), allow_caps)?;
        // Apply Option-rewrapping when the field is Option<T>.
        let wrapped = rewrap_option_field(fty, inner);
        fields.push(wrapped);
    }
    // class stored without leading ':'; path has it — strip.
    let class = path.strip_prefix(':').unwrap_or(&path).to_string();
    Ok(Value::Aggregate(Arc::new(AggregateValue::record(class, Arc::new(fields)))))
}

/// Arc 234 Stone 234.7b — Decode a holon-record tagged body (a `#wat-edn.holon/Bind[…]`
/// inner value) back to `Value::Aggregate(holder=HolonRecord)`.
///
/// Steps:
/// 1. Reconstruct `hologram` exactly via `edn_to_holon_ast` (the proven round-trip).
/// 2. Project `fields` from the Bundle leaves:
///    `hologram` must be `Bind(_, Bundle(children))`;
///    each child must be `Bind(_, val_node)`;
///    `fields[i] = from_holon_item(val_node)` (pure; no eval context).
///    `val_node` is typically `Atom(to-holon(val))` — the Atom is unwrapped locally
///    here (confined to record projection) before calling `from_holon_item`.
/// 3. class from the wire tag path (strip leading ':').
///
/// STOP: if `hologram` is not `Bind(_, Bundle(_))` → `EdnReadError::Other`.
fn reconstruct_holon_record(
    ns: &str,
    name: &str,
    body: &OwnedValue,
    _types: &crate::types::TypeEnv,
) -> Result<Value, EdnReadError> {
    use holon::HolonAST;

    // 1. Reconstruct holon_form exactly via the proven edn round-trip.
    let holon_arc = edn_to_holon_ast(body)?;
    let holon_form: HolonAST = (*holon_arc).clone();

    // 2. Project struct_form from the Bundle leaves.
    //    Shape: Bind(_class, Bundle([Bind(_name, val_node), ...]))
    let children = match &holon_form {
        HolonAST::Bind(_, right) => match right.as_ref() {
            HolonAST::Bundle(children) => children.clone(),
            _ => {
                return Err(EdnReadError {
                    span: Span::unknown(),
                    kind: EdnReadErrorKind::Other(
                        "reconstruct_holon_record: holon_form inner (right of outer Bind) must be Bundle".into()
                    ),
                });
            }
        },
        _ => {
            return Err(EdnReadError {
                span: Span::unknown(),
                kind: EdnReadErrorKind::Other(
                    "reconstruct_holon_record: holon_form must be Bind(class, Bundle)".into()
                ),
            });
        }
    };

    // Each child is Bind(_name, val_node); project val_node → Value.
    // val_node is Atom(to-holon(field_val)) from the Record.wat macro.
    // Unwrap the opaque-identity Atom here (confined to record projection),
    // then decode the inner via from_holon_item. NOT widened into the shared
    // from_holon_item, where it would silently misdecode a collection of holon values.
    let op = "reconstruct_holon_record";
    let span = Span::unknown();
    let mut fields: Vec<Value> = Vec::with_capacity(children.len());
    for child in children.iter() {
        match child {
            HolonAST::Bind(_, val_node) => {
                // The Record.wat macro stores each field value as Atom(to-holon(val)).
                // Unwrap the opaque-identity Atom here (confined to record projection),
                // then decode the inner via from_holon_item. NOT widened into the shared
                // from_holon_item, where it would silently misdecode a collection of holon values.
                let inner = match val_node.as_ref() {
                    HolonAST::Atom(inner) => inner.as_ref(),
                    other => other,
                };
                let v = crate::runtime::from_holon_item(inner, op, &span)
                    .map_err(|e| EdnReadError {
                        span: Span::unknown(),
                        kind: EdnReadErrorKind::Other(format!(
                            "reconstruct_holon_record: from_holon_item failed: {e}"
                        )),
                    })?;
                fields.push(v);
            }
            _ => {
                return Err(EdnReadError {
                    span: Span::unknown(),
                    kind: EdnReadErrorKind::Other(
                        "reconstruct_holon_record: holon_form Bundle child must be Bind".into()
                    ),
                });
            }
        }
    }

    // 3. class from the wire tag path (strip leading ':').
    let path = ns_to_wat_path(ns, name);
    let class = path.strip_prefix(':').unwrap_or(&path).to_string();

    Ok(Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class,
        Arc::new(fields),
        Arc::new(holon_form),
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
) -> Result<Value, EdnReadError> {
    let path = ns_to_enum_path(ns);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Enum(d)) => d,
        _ => {
            // arc 138: no span — reconstruct_enum_tagged operates on parsed OwnedValue, no WatAST
            return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: variant_name.to_string(), body_shape: "vector" } });
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
            EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::EnumVariantNotFound { type_path: path.clone(), variant: variant_name.to_string() } }
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
        let inner = edn_to_value_caps(item, Some(types), allow_caps)?;
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
            return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::UnknownTag { ns: ns.to_string(), name: variant_name.to_string(), body_shape: "nil" } });
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
            EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::EnumVariantNotFound { type_path: path.clone(), variant: variant_name.to_string() } }
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

/// Encode a `Value` to a compact EDN `String` without a type registry.
///
/// This is the registry-free codec for process-tier wire serialisation when
/// no `TypeEnv` is available (e.g. `EdnRepresentable::to_wire` for Value on
/// the thread-tier, or `HolonRepresentable` paths). User-defined struct/enum
/// fields are rendered with positional `:field-{i}` keys.
///
/// Arc 258.5b-ii: the socket-tier PEER_TYPE_PATH send path now uses
/// [`value_to_edn_string_with`] (with `sym.types()`) so named record fields
/// cross the wire correctly. This function no longer reads a thread-local.
pub(crate) fn value_to_edn_string(v: &Value) -> String {
    wat_edn::write(&value_to_edn_with(v, None))
}

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
/// `HolonRepresentable for Value` for completeness.
///
/// Passes `None` for the type registry — reconstructs only primitive
/// Values (i64, f64, bool, nil, String, keyword, Vec, HashMap). User-
/// defined structs/enums are not reconstructed without a TypeEnv; the
/// process tier's program fn works on the decoded primitive scaffold.
pub(crate) fn edn_string_to_value(s: &str) -> Result<Value, EdnReadError> {
    read_edn(s, None)
}

/// Arc 272 6a-i — **THE ONE TRUSTED-WIRE DECODE DOOR.** The sole entry that reconstructs portable
/// capability (`wat-edn.cap`) tags into live capabilities. Object-capability transfer-only: a
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
) -> Result<Value, EdnReadError> {
    let v = read_edn_caps(s, types, true)?;
    // ── RETIRED arc 293.W.2a (deleted by arc 293.W.2d) ───────────────────────
    // The §7 runtime backstop that refused a top-level Holder::Struct at the
    // wire-decode door is gone. The compile-time purity wall at wire-peer
    // PRODUCERS (peer-pair', socket-pair', connect', accept', program-self-peer')
    // makes the reachable struct-on-wire case structurally unrepresentable. The
    // untyped pprintln path is a trust-boundary concern outside our scope.
    Ok(v)
}

#[cfg(test)]
mod cap_decode_boundary {
    //! Arc 272 6a-i / 6c.2 — the trap-door ward. A capability (`wat-edn.cap`) tag reconstructs ONLY
    //! through the trusted door; the general/untrusted decode path REFUSES it. If this ever flips, the
    //! forge-hole reopens (parsed data minting live capabilities). This is the regression alarm bolted
    //! onto the exact trap we fell through — it must never open again.
    use super::{decode_trusted_wire, edn_string_to_value};

    // Arc 272 6c.2 — the wire format is now a SocketAddressWire record (not a bare byte vector).
    // The address cap tag wraps a #wat.kernel/SocketAddressWire tagged map.
    const CAP_TAG_GENERAL: &str = "#wat-edn.cap/address #wat.kernel/SocketAddressWire {:minter-pid 1 :name [1 2 3 4 5]}";

    fn make_types() -> crate::types::TypeEnv {
        use crate::types::{AggregateDef, Holder, TypeDef, TypeExpr};
        // with_builtins seeds :wat::core::Record (required parent for SocketAddressWire).
        let mut env = crate::types::TypeEnv::with_builtins();
        // Arc 293.2b — use AggregateDef (holder=Record) instead of the annihilated RecordDef.
        env.register_stdlib(TypeDef::Aggregate(AggregateDef {
            name: ":wat::kernel::SocketAddressWire".to_string(),
            type_params: vec![],
            holder: Holder::Record,
            restrictions: None,
            // minter-pid <- :wat::core::i64
            // name       <- :wat::core::Vector<wat::core::i64>
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
            "general/untrusted decode MUST refuse a wat-edn.cap tag — a capability is handed over a \
             trusted channel, never forged from parsed data (ocap transfer-only)"
        );
    }

    #[test]
    fn trusted_door_reconstructs_capability_tags() {
        let types = make_types();
        assert!(
            decode_trusted_wire(CAP_TAG_GENERAL, Some(&types)).is_ok(),
            "the trusted-wire door MUST reconstruct a wat-edn.cap tag into a live capability"
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
        Value::Aggregate(sv) if sv.holder == crate::types::Holder::Struct => {
            let type_key = format!(":{}", sv.class);
            let tag = tag_from_type_path(&type_key);
            // Arc 293.2b — struct aggregates (kind==Struct) replace TypeDef::Struct.
            let field_names: Vec<String> = match types.and_then(|t| t.get(&type_key)) {
                Some(crate::types::TypeDef::Aggregate(a)) if a.holder == crate::types::Holder::Struct => {
                    a.fields.iter().map(|(name, _ty)| name.clone()).collect()
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
        // A WatAST is a parsed form — by definition an EDN value (watast_to_edn/edn_to_watast
        // are a total bijection). Render it faithfully as its form (legible + recoverable);
        // opaque-nil was a lie. Round-trip-as-WatAST is type-directed (from-edn :T / the typed slot).
        Value::wat__WatAST(a) => crate::wat_edn_bridge::watast_to_edn(a.as_ref()),
        Value::wat__core__fn(_) => opaque_nil("wat-edn.opaque", "fn"),
        Value::wat__kernel__Sender(_) => opaque_nil("wat-edn.opaque", "Sender"),
        Value::wat__kernel__Receiver(_) => opaque_nil("wat-edn.opaque", "Receiver"),
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
        Value::RustOpaque(inner) => {
            // Arc 272 narrow-waist — GENERIC capability dispatch (the FROZEN waist; never changes
            // per-capability). If this opaque is a registered PORTABLE capability with a portable
            // form, emit its `#wat-edn.cap/<name>` tag; otherwise it is a process-local handle (an fd,
            // a `Sender`) that must NOT cross → the payload-less `#wat-edn.opaque/RustOpaque` tag (the
            // decoder refuses it). The per-capability codecs live in `crate::capability::registry`.
            // `types` is required by record-based codecs (arc 272 6c.2 SocketAddressWire field
            // naming); when `types` is None (display/logging paths), capability encoding is skipped
            // and the address falls to the opaque tag (appropriate — it cannot be meaningfully
            // encoded without the type registry).
            if let Some(t) = types {
                if let Some(cap_tag) = crate::capability::encode_capability(inner, t) {
                    return cap_tag;
                }
            }
            OwnedValue::Tagged(
                Tag::ns("wat-edn.opaque", "RustOpaque"),
                Box::new(OwnedValue::String(std::borrow::Cow::Owned(
                    inner.type_path.to_string(),
                ))),
            )
        }
        Value::OnlineSubspace(_) => opaque_nil("wat-edn.opaque", "OnlineSubspace"),
        Value::Reckoner(_) => opaque_nil("wat-edn.opaque", "Reckoner"),
        Value::Engram(_) => opaque_nil("wat-edn.opaque", "Engram"),
        Value::EngramLibrary(_) => opaque_nil("wat-edn.opaque", "EngramLibrary"),
        Value::Hologram(_) => opaque_nil("wat-edn.opaque", "Hologram"),
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
        // Arc 293.R2.1 — Record/HolonRecord: Aggregate with holder != Struct.
        // No guard here — the Struct arm above catches holder==Struct; this arm is reached
        // only for Record/HolonRecord. Guard dropped so Rust's exhaustiveness checker sees
        // Value::Aggregate(_) as fully covered.
        // Dispatch on holon: Hologram → holon wire form; Empty → base named-field map.
        Value::Aggregate(a) => {
            let type_key = format!(":{}", a.class);
            let tag = tag_from_type_path(&type_key);
            match &a.holon {
                HolonForm::Hologram(hologram) => {
                    // Arc 234 Stone 234.7b — HolonRecord: ride hologram as edn.
                    // The body is a #wat-edn.holon/Bind[...] value (NOT a map) so the decode
                    // path can distinguish holon records from base records (which have Map bodies).
                    // fields projection is not read here — identity lives in the hologram.
                    OwnedValue::Tagged(tag, Box::new(holon_ast_to_edn(hologram)))
                }
                HolonForm::Empty => {
                    // Arc 234 Stone 234.7a — base Record: named-field tagged-map.
                    // class has NO leading colon; TypeEnv keys DO — prepend ':' for lookup.
                    // Arc 293.2b: use AggregateDef (kind!=Struct) instead of the annihilated RecordDef.
                    // Fallback to field-{i} when no def is found (no-types or unregistered class).
                    let field_names: Vec<String> = match types.and_then(|t| t.get(&type_key)) {
                        Some(crate::types::TypeDef::Aggregate(def)) if def.holder != crate::types::Holder::Struct => {
                            def.field_names().map(|s| s.to_string()).collect()
                        }
                        _ => (0..a.fields.len()).map(|i| format!("field-{}", i)).collect(),
                    };
                    let entries: Vec<(OwnedValue, OwnedValue)> = a
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
            }
        }
        // Arc 118 — Stream: opaque (lazy; realizing for EDN would diverge on infinite seqs).
        // Render the forced prefix if available, otherwise as an opaque lazy sentinel.
        Value::wat__stream__Stream(seq) => {
            use crate::stream::Stream;
            match seq.as_ref() {
                Stream::Empty => OwnedValue::List(vec![]),
                Stream::Cons { head, .. } => {
                    // Only render the head (forced); tail may be infinite.
                    OwnedValue::Tagged(
                        Tag::ns("wat-edn.opaque", "Stream"),
                        Box::new(value_to_edn_with(head, types)),
                    )
                }
                Stream::Thunk(_) => opaque_nil("wat-edn.opaque", "lazy-seq"),
            }
        }
        // Stone 237.2 — wat__core__clauses: opaque (multi-arity dispatcher;
        // not directly serializable to EDN).
        Value::wat__core__clauses(cs) => opaque_nil("wat-edn.opaque", {
            let _ = cs;
            "clauses"
        }),
        // Arc 232 Stone 232.1 — registry carriers: opaque (not value-serializable).
        Value::wat__core__protocol_def(pd) => opaque_nil("wat-edn.opaque", {
            let _ = pd;
            "protocol-def"
        }),
        Value::wat__core__extend_def(ed) => opaque_nil("wat-edn.opaque", {
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
///
/// Arc 230: Symbol/Keyword/Tag/Nil variants retired. Those forms now
/// exist as `Bind(Atom(String(cls)), Atom(String(val)))` compositions.
/// We intercept them before the generic Bind arm so that the EDN
/// round-trip writer emits the familiar `#wat-edn.holon/Keyword` etc.
/// tags (the reader reconstructs via the updated constructors).
fn holon_ast_to_edn(h: &holon::HolonAST) -> OwnedValue {
    use holon::HolonAST;
    // Arc 230: intercept Symbol/Keyword/Tag/Nil compositions before generic dispatch.
    if let Some(s) = h.as_symbol() {
        return OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Symbol"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(s.to_string()))),
        );
    }
    if let Some(s) = h.as_keyword() {
        return OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Keyword"),
            Box::new(OwnedValue::Keyword(Keyword::new(s))),
        );
    }
    if let Some(s) = h.as_tag() {
        return OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Tag"),
            Box::new(OwnedValue::String(std::borrow::Cow::Owned(s.to_string()))),
        );
    }
    // Note: is_nil() = as_symbol() == Some("nil"), handled by the Symbol arm above.
    match h {
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
        // Arc 221 Stone 221.2 — Char primitive leaf. Encodes as
        // #wat-edn.holon/Char containing an EDN character literal.
        HolonAST::Char(c) => OwnedValue::Tagged(
            Tag::ns("wat-edn.holon", "Char"),
            Box::new(OwnedValue::Char(*c)),
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
        _ => Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("expected #wat-edn.holon/* tagged form for HolonAST round-trip; use edn_to_holon_ast_natural for the tagless read".into()) }),
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
            // Arc 230: HolonAST::keyword() now produces Bind composition.
            // Arc 221 Stone 221.4b — EDN keyword maps to HolonAST::keyword() (no leading colon).
            // Mirror `keyword_from_wat_path`'s inverse — EDN keyword `foo/bar`
            // (namespace `foo`, name `bar`) maps to wat-path `foo::bar`.
            let s = match k.namespace() {
                Some(ns) => format!("{}::{}", ns.replace('.', "::"), k.name()),
                None => k.name().to_string(),
            };
            Ok(Arc::new(HolonAST::keyword(&s)))
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
        _ => Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("natural-form HolonAST read can't lift this EDN shape; expected primitive leaf or #wat-edn.holon/* tagged composite")) }),
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
        // Arc 230: Symbol/Keyword/Nil/Tag variants retired; constructors produce
        // Bind(Atom(String(cls)), Atom(String(val))) compositions.
        ("Symbol", OwnedValue::String(s)) => {
            Ok(Arc::new(HolonAST::symbol(s.as_ref())))
        }
        ("String", OwnedValue::String(s)) => {
            Ok(Arc::new(HolonAST::String(Arc::from(s.as_ref()))))
        }
        ("I64", OwnedValue::Integer(n)) => Ok(Arc::new(HolonAST::I64(*n))),
        ("F64", OwnedValue::Float(x)) => Ok(Arc::new(HolonAST::F64(*x))),
        ("Bool", OwnedValue::Bool(b)) => Ok(Arc::new(HolonAST::Bool(*b))),
        // Arc 221 Stone 221.2 — Char leaf round-trip (mirrors holon_ast_to_edn Char arm).
        ("Char", OwnedValue::Char(c)) => Ok(Arc::new(HolonAST::Char(*c))),
        // Arc 230: Keyword/Nil/Tag use updated constructors (produce Bind compositions).
        // Keyword: stored content has no leading colon; keyword() strips it.
        ("Keyword", OwnedValue::Keyword(kw)) => {
            Ok(Arc::new(HolonAST::keyword(kw.name())))
        }
        ("Nil", OwnedValue::Nil) => Ok(Arc::new(HolonAST::nil())),
        // Tag: stored content has no leading '#'; reconstruct via tag() constructor.
        ("Tag", OwnedValue::String(s)) => {
            Ok(Arc::new(HolonAST::tag(s.as_ref())))
        }
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
                    return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("#wat-edn.holon/Permute body[1] must be an Integer (k)".into()) });
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
                    return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("#wat-edn.holon/Blend body[2] must be a Float (w1)".into()) });
                }
            };
            let w2 = match &items[3] {
                OwnedValue::Float(x) => *x,
                OwnedValue::Integer(n) => *n as f64,
                // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
                _ => {
                    return Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("#wat-edn.holon/Blend body[3] must be a Float (w2)".into()) });
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
                EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("#wat-edn.holon/SlotMarker missing :min".into()) }
            })?;
            let max = max.ok_or_else(|| {
                EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other("#wat-edn.holon/SlotMarker missing :max".into()) }
            })?;
            Ok(Arc::new(HolonAST::SlotMarker { min, max }))
        }
        // arc 138: no span — edn_holon_tag_to_ast walks parsed OwnedValue, no WatAST
        (other, _) => Err(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("#wat-edn.holon/{other}: unrecognized tag or body shape")) }),
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
        .ok_or_else(|| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("#wat-edn.holon/{op} missing :value")) })?;
    let min = min
        .ok_or_else(|| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("#wat-edn.holon/{op} missing :min")) })?;
    let max = max
        .ok_or_else(|| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("#wat-edn.holon/{op} missing :max")) })?;
    Ok((value, min, max))
}

/// Render a HolonAST as a tagged-EDN string (single-line).
///
/// Inverse of [`read_holon_ast_tagged`]. The roundtrip `read . write`
/// is an identity on valid HolonASTs.
///
/// Output is single-line per `wat_edn::write` guarantee — embedded
/// newlines in payload strings escape as `\n` literal. This makes
/// the output safe for newline-framed wire protocols (process-tier
/// pipe framing per arc 214 Slice 3 Stone C).
pub fn write_holon_ast_tagged(h: &holon::HolonAST) -> String {
    wat_edn::write(&holon_ast_to_edn(h))
}

/// Public arc-093: parse an EDN string and reconstruct a
/// `HolonAST` from its round-trip-safe tagged form. Inverse of
/// the substrate's `:wat::edn::write` for HolonAST values; what
/// the wat-telemetry-sqlite cursor calls per `:wat::edn::Tagged`
/// column.
pub fn read_holon_ast_tagged(s: &str) -> Result<Arc<holon::HolonAST>, EdnReadError> {
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_holon_ast_tagged operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })?;
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
        .map_err(|e| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })?;
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
    // Arc 230: Symbol composition is Bind(Atom(String("Symbol")), Atom(String(s))).
    // as_symbol() recognises the composition; pass the content through keyword_from_wat_path
    // (same semantics as the old HolonAST::Symbol(s) arm — Symbol stored colon-prefixed
    // keywords in the old encoding; in the new encoding the symbol content carries the
    // raw identifier or colon-prefixed keyword string).
    if let Some(s) = h.as_symbol() {
        return keyword_from_wat_path(s);
    }
    // Arc 230: Keyword composition is Bind(Atom(String("Keyword")), Atom(String(s))).
    // as_keyword() recognises the composition; pass the content through keyword_from_wat_path
    // to translate wat-path `::` separators to EDN `/` (e.g. `test::reader` → `:test/reader`).
    // Without this arm, keyword compositions fall to the `_ => holon_ast_to_edn(h)` branch
    // which calls Keyword::new(s) without namespace translation, producing `:test::reader`
    // — invalid EDN (double-colon inside a keyword).
    if let Some(s) = h.as_keyword() {
        return keyword_from_wat_path(s);
    }
    match h {
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
    use crate::runtime::SymbolTable;
    use crate::types::TypeExpr;

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
            matches!(err, EdnReadError { kind: EdnReadErrorKind::NoTypeRegistry, .. }),
            "expected NoTypeRegistry, got: {:?}",
            err
        );
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
        let t = TypeExpr::Parametric {
            head: "wat::core::Option".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let v = coerce(&t, "nil").unwrap();
        match v {
            Value::Option(o) => assert!(o.is_none()),
            other => panic!("expected Value::Option(None); got {:?}", other),
        }
    }

    #[test]
    fn arc170_1fi_coerce_option_some() {
        let t = TypeExpr::Parametric {
            head: "wat::core::Option".into(),
            args: vec![TypeExpr::Path(":wat::core::i64".into())],
        };
        let v = coerce(&t, "7").unwrap();
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
        let t = TypeExpr::Parametric {
            head: "wat::core::Result".into(),
            args: vec![
                TypeExpr::Path(":wat::core::i64".into()),
                TypeExpr::Path(":wat::core::String".into()),
            ],
        };
        let v = coerce(&t, "#wat-edn.result/ok 42").unwrap();
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
        let t = TypeExpr::Parametric {
            head: "wat::core::Result".into(),
            args: vec![
                TypeExpr::Path(":wat::core::i64".into()),
                TypeExpr::Path(":wat::core::String".into()),
            ],
        };
        let v = coerce(&t, "#wat-edn.result/err \"boom\"").unwrap();
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
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        pv = pv.push_back(Value::i64(10));
        pv = pv.push_back(Value::i64(20));
        pv = pv.push_back(Value::i64(30));
        let orig = Value::wat__core__PersistentVector(pv);

        // Serialize → tagged EDN string.
        let s = value_to_edn_string(&orig);

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
    // RED before the fix (renders `#wat-edn.opaque/WatAST nil`); GREEN after (renders the form).
    #[test]
    fn watast_renders_as_its_form_not_opaque_nil() {
        let forms = crate::parser::parse_all_with_file("(:wat::core::< -5 0)", "<watast-render-probe>")
            .expect("parse the form");
        let ast = forms.into_iter().next().expect("one form");
        let v = Value::wat__WatAST(Arc::new(ast));
        let s = value_to_edn_string(&v);
        assert!(
            !s.contains("opaque") && s.contains("-5"),
            "a WatAST must render as its form (with operands), not opaque-nil; got: {s}"
        );
    }

    // ─── Arc 278 stone 0a — PersistentMap EDN round-trip ───────────────

    #[test]
    fn persistent_map_edn_round_trip() {
        // Build a PersistentMap with two entries.
        let mut m: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
        m = m.insert(Value::String(Arc::new("a".to_string())), Value::i64(1));
        m = m.insert(Value::String(Arc::new("b".to_string())), Value::i64(2));
        let pm = Value::wat__core__PersistentMap(m);

        // Serialize → tagged EDN string.
        let s = value_to_edn_string(&pm);

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
