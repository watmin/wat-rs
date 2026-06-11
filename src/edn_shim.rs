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
///     reconstruct `Value::Struct` with declared field names.
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
        _ => return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "ast-name requires a Symbol or Keyword node (no other node has a name)".to_string(),
        } }),
    };
    Ok(crate::value::TrackedValue::new(
        Value::String(std::sync::Arc::new(name)),
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
    let edn = wat_edn::parse_owned(s)
        // arc 138: no span — read_edn operates on a raw &str with no WatAST trace
        .map_err(|e| EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })?;
    edn_to_value(&edn, types)
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
#[allow(clippy::mutable_key_type)]
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
                .map(|x| edn_to_value(x, types))
                .collect::<Result<_, _>>()?;
            Ok(Value::wat__core__List(Arc::new(walked)))
        }
        Edn::Vector(items) => {
            let walked: Vec<Value> = items
                .iter()
                .map(|x| edn_to_value(x, types))
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
                let k_val = edn_to_value(k, types)?;
                let v_val = edn_to_value(v, types)?;
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
                let v_val = edn_to_value(x, types)?;
                backing.insert(v_val);
            }
            Ok(Value::wat__std__HashSet(Arc::new(backing)))
        }
        Edn::Inst(t) => Ok(Value::Instant(*t)),
        // arc 138: no span — edn_to_value walks an OwnedValue tree (already-parsed EDN); no WatAST available
        // Arc 207 slice 2: `#uuid "..."` EDN reader literal → typed `:wat::core::Uuid`.
        // `uuid::Uuid` is `Copy`; mirrors `Edn::Inst(t) → Value::Instant(*t)` pattern.
        Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u)),
        Edn::Tagged(tag, body) => tagged_to_value(tag, body, types),
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
                    Some(crate::types::TypeDef::Struct(def)) => {
                        coerce_struct_path(p, def, edn, types)
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
                    Some(crate::types::TypeDef::Struct(def)) => {
                        coerce_struct_path(&path, def, edn, types)
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

fn coerce_struct_path(
    type_path: &str,
    def: &crate::types::StructDef,
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
    Ok(Value::Struct(Arc::new(crate::runtime::StructValue {
        type_name: type_path.to_string(),
        fields,
    })))
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
) -> Result<Value, EdnReadError> {
    use wat_edn::Value as Edn;
    let ns = tag.namespace();
    let name = tag.name();

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

    // arc 138: no span — tagged_to_value walks parsed OwnedValue, no WatAST in scope
    let types = types.ok_or(EdnReadError { span: Span::unknown(), kind: EdnReadErrorKind::NoTypeRegistry })?;

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
) -> Result<Value, EdnReadError> {
    let path = ns_to_wat_path(ns, name);
    let def = match types.get(&path) {
        Some(crate::types::TypeDef::Struct(d)) => d,
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

/// Encode a `Value` to a compact EDN `String` — the single codec for
/// process-tier wire serialization (`spawn-program' :process` apply-loop).
///
/// This is the canonical `Value → String` conversion for the process
/// tier; all encode sites in kernel/spawn.rs route through here so there
/// is exactly ONE encode path. Inverse: [`edn_string_to_value`].
pub(crate) fn value_to_edn_string(v: &Value) -> String {
    wat_edn::write(&value_to_edn(v))
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
        Value::wat__std__HashSet(s) => OwnedValue::Set(
            // Stone 216.5b — iterate s.iter() (Values directly, not String keys).
            s.iter().map(|x| value_to_edn_with(x, types)).collect(),
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
        // Arc 207 — typed Uuid → EDN `#uuid "..."` reader literal.
        // Mirrors `Value::Instant → OwnedValue::Inst` pattern.
        // `uuid::Uuid` is `Copy`; `OwnedValue::Uuid` already exists
        // in wat-edn (no crates/wat-edn/ edits needed).
        Value::wat__core__Uuid(u) => OwnedValue::Uuid(*u),
        // Arc 220 — typed Char → EDN character literal.
        // `char` is `Copy`; `OwnedValue::Char` already exists in wat-edn.
        Value::wat__core__Char(c) => OwnedValue::Char(*c),
        // Arc 234 Stone 234.1 — wat__holon__Record: render as a tagged map.
        // Stone S-C.2c — wat__Record (base) uses the same tagged-map rendering.
        // Tag carries the class_fqdn; fields render positionally (field-0, field-1, ...).
        // Named-field EDN rendering lands in Stone 234.2 when defrecord macro ships field names.
        Value::wat__holon__Record { class_fqdn, struct_form, .. }
        | Value::wat__Record { class_fqdn, struct_form } => {
            let tag = tag_from_type_path(class_fqdn);
            let entries: Vec<(OwnedValue, OwnedValue)> = struct_form
                .iter()
                .enumerate()
                .map(|(i, fv)| {
                    (
                        OwnedValue::Keyword(Keyword::new(format!("field-{}", i))),
                        value_to_edn_with(fv, types),
                    )
                })
                .collect();
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(entries)))
        }
        // Stone 237.2 — wat__core__clauses: opaque (multi-arity dispatcher;
        // not directly serializable to EDN).
        Value::wat__core__clauses(cs) => opaque_nil("wat-edn.opaque", {
            let _ = cs;
            "clauses"
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
}
