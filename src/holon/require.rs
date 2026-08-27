//! `require_*` extraction helpers — coerce a wat-tier `Value` into the
//! holon-domain Rust type a VSA algebra function needs (`Hologram`,
//! `Vector`, `OnlineSubspace`, `Reckoner`, `Engram`, `EngramLibrary`,
//! or a primitive `String`/`f64`/`Function` argument), erroring with a
//! `TypeMismatch` otherwise. Pure functions lifted out of `runtime.rs`
//! per Stone HOME-8 — see `src/holon/mod.rs` for the two-layer doctrine.

use crate::runtime::{EvalBreak, Function, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use crate::span::Span;
use holon;
use std::sync::Arc;

pub(crate) fn require_hologram(
    op: &str,
    v: Value,
) -> Result<Arc<crate::rust_deps::ThreadOwnedCell<crate::holon::hologram::Hologram>>, EvalBreak> {
    match v {
        Value::Hologram(h) => Ok(h),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::Hologram",
                got: Box::new(ValueSnapshot::of(&other)),
                // arc 138: no — takes Value, not WatAST; no source coords available
            },
        )
        .into()),
    }
}


pub(crate) fn require_fn(op: &str, v: Value) -> Result<Arc<Function>, EvalBreak> {
    match v {
        Value::wat__core__fn(f) => Ok(f),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "fn(f64)->bool",
                got: Box::new(ValueSnapshot::of(&other)),
                // arc 138: no — takes Value, not WatAST; no source coords available
            },
        )
        .into()),
    }
}


::wat_source_derive::wat_field_names_from!(MATCH_FIELDS, "wat/holon.wat", ":wat::holon::Match");
pub(crate) fn match_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(MATCH_FIELDS))
        .clone()
}


/// Arc 053 — helper. Extract a `Value::Vector` payload, error on
/// non-Vector input. Cousin of `require_holon`. Used by the
/// Vector-tier algebra primitives.
pub(crate) fn require_vector(op: &str, v: Value) -> Result<Arc<holon::Vector>, EvalBreak> {
    match v {
        Value::Vector(vec) => Ok(vec),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::Vector",
                got: Box::new(ValueSnapshot::of(&other)),
                // arc 138: no — takes Value, not WatAST; no source coords available
            },
        )
        .into()),
    }
}


/// Arc 053 — extract a `Value::OnlineSubspace` payload.
pub(crate) fn require_subspace(
    op: &str,
    v: Value,
    list_span: &Span,
) -> Result<Arc<crate::rust_deps::ThreadOwnedCell<holon::OnlineSubspace>>, EvalBreak> {
    match v {
        Value::OnlineSubspace(s) => Ok(s),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::OnlineSubspace",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// Arc 053 — wrap a `Vec<f64>` into a wat-tier `(:wat::core::Vector :- [f64])` Value.
pub(crate) fn vec_f64_to_value(xs: Vec<f64>) -> Value {
    Value::Vec(Arc::new(xs.into_iter().map(Value::f64).collect()))
}


/// Arc 053 — extract a `Value::Reckoner` payload.
pub(crate) fn require_reckoner(
    op: &str,
    v: Value,
    list_span: &Span,
) -> Result<Arc<crate::rust_deps::ThreadOwnedCell<holon::Reckoner>>, EvalBreak> {
    match v {
        Value::Reckoner(r) => Ok(r),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::Reckoner",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// Arc 053 — extract a `Value::Engram` payload.
pub(crate) fn require_engram(
    op: &str,
    v: Value,
    list_span: &Span,
) -> Result<Arc<crate::rust_deps::ThreadOwnedCell<holon::Engram>>, EvalBreak> {
    match v {
        Value::Engram(e) => Ok(e),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::Engram",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// Arc 053 — extract a `Value::EngramLibrary` payload.
pub(crate) fn require_engram_library(
    op: &str,
    v: Value,
    list_span: &Span,
) -> Result<Arc<crate::rust_deps::ThreadOwnedCell<holon::EngramLibrary>>, EvalBreak> {
    match v {
        Value::EngramLibrary(l) => Ok(l),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::holon::EngramLibrary",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


/// Arc 053 — extract a String from a Value.
pub(crate) fn require_string(op: &str, v: Value, list_span: &Span) -> Result<String, EvalBreak> {
    match v {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


pub(crate) fn require_numeric(op: &str, v: Value, list_span: &Span) -> Result<f64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(n as f64),
        Value::f64(x) => Ok(x),
        other => Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "numeric",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}


