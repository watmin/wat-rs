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
use std::sync::Arc;
use wat_edn::{Keyword, OwnedValue, Tag};

// ─── Public eval entry points ────────────────────────────────────

/// `(:wat::edn::write v)` → `:String`. Compact single-line EDN.
pub fn eval_edn_write(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::edn::write";
    let v = require_one_arg(OP, args, env, sym)?;
    let edn = value_to_edn(&v);
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
    let edn = value_to_edn(&v);
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
    let edn = value_to_edn(&v);
    Ok(Value::String(Arc::new(wat_edn::to_json_string(&edn))))
}

fn require_one_arg(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    eval(&args[0], env, sym)
}

// ─── The walker ──────────────────────────────────────────────────

/// Convert a wat `Value` to a `wat_edn::OwnedValue`. Per-variant
/// mapping table at the top of the module.
pub fn value_to_edn(v: &Value) -> OwnedValue {
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
            Some(inner) => value_to_edn(inner),
        },
        Value::Result(r) => match &**r {
            Ok(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "ok"),
                Box::new(value_to_edn(inner)),
            ),
            Err(inner) => OwnedValue::Tagged(
                Tag::ns("wat-edn.result", "err"),
                Box::new(value_to_edn(inner)),
            ),
        },

        // ── Compound containers ──────────────────────────────────
        Value::Vec(xs) => OwnedValue::Vector(xs.iter().map(value_to_edn).collect()),
        Value::Tuple(xs) => OwnedValue::Vector(xs.iter().map(value_to_edn).collect()),
        Value::wat__std__HashMap(m) => OwnedValue::Map(
            m.values()
                .map(|(k, v)| (value_to_edn(k), value_to_edn(v)))
                .collect(),
        ),
        Value::wat__std__HashSet(s) => OwnedValue::Set(s.values().map(value_to_edn).collect()),

        // ── User-declared struct / enum ──────────────────────────
        Value::Struct(sv) => {
            let tag = tag_from_type_path(&sv.type_name);
            // Fields rendered as a Map with :field-N keys (positional —
            // struct field names aren't carried at runtime in slice 1).
            let entries: Vec<(OwnedValue, OwnedValue)> = sv
                .fields
                .iter()
                .enumerate()
                .map(|(i, fv)| {
                    (
                        OwnedValue::Keyword(Keyword::new(&format!("field-{}", i))),
                        value_to_edn(fv),
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
                let payload: Vec<OwnedValue> =
                    ev.fields.iter().map(value_to_edn).collect();
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
