//! Marshaling between wat `Value` and Rust types.
//!
//! The `#[wat_dispatch]` macro (see
//! `docs/arc/2026/04/002-rust-interop-macro/MACRO-DESIGN.md`) emits calls into
//! these traits to convert arguments (wat → Rust) and returns
//! (Rust → wat). The traits are userland-extensible — a shim author
//! can implement `ToWat` / `FromWat` for any type their shim exposes.
//!
//! # What this module provides
//!
//! - [`ToWat`] / [`FromWat`] — the conversion traits.
//! - Blanket impls for primitives (`i64`, `f64`, `bool`, `String`, `()`,
//!   `Option<T>`, `Vec<T>`, `Result<T,E>`, tuples up to arity 6, and
//!   `Value` pass-through).
//! - [`RustOpaqueInner`] + [`Value::RustOpaque`] — the generic
//!   opaque-handle variant. Shim types (e.g. `LruCacheCell`) become
//!   opaque payloads; wat sees them as opaque values identified by
//!   their `:rust::...` type path.
//!
//! # Scope discipline
//!
//! This module is scope-agnostic. The `scope = "thread_owned"` /
//! `"shared"` / `"owned_move"` attributes on `#[wat_dispatch]`
//! control what WRAPPER TYPE the shim uses for its payload
//! (thread-owned cells, plain `Arc`, consumed cells). The marshaling
//! layer only sees `Box<dyn Any + Send + Sync>` — the wrapper's own
//! semantics kick in when the shim's dispatch code handles the
//! downcasted payload. Ownership-scope custody primitives live in
//! the sibling `custodia` module.

use std::any::Any;
use std::sync::Arc;

use crate::runtime::{RuntimeError, Value};
use crate::span::Span;

/// Convert a Rust value into a wat [`Value`]. Used by shim dispatch
/// fns when marshaling a method's return to wat.
pub trait ToWat {
    fn to_wat(self) -> Value;
}

/// Convert a wat [`Value`] into a Rust value. Used by shim dispatch
/// fns when marshaling wat arguments into Rust method params.
///
/// The `op` parameter names the wat-level operation so error messages
/// can point the user at the exact call site. The `span` parameter
/// carries the source location of the original call-site AST node so
/// that type-mismatch errors surface with file:line:col coordinates.
pub trait FromWat: Sized {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError>;
}

// ─── Primitive impls ─────────────────────────────────────────────────

impl ToWat for i64 {
    fn to_wat(self) -> Value {
        Value::i64(self)
    }
}

impl FromWat for i64 {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::i64(n) => Ok(*n),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

impl ToWat for f64 {
    fn to_wat(self) -> Value {
        Value::f64(self)
    }
}

impl FromWat for f64 {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::f64(x) => Ok(*x),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "f64",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

impl ToWat for bool {
    fn to_wat(self) -> Value {
        Value::bool(self)
    }
}

impl FromWat for bool {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::bool(b) => Ok(*b),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "bool",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

impl ToWat for String {
    fn to_wat(self) -> Value {
        Value::String(Arc::new(self))
    }
}

impl FromWat for String {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::String(s) => Ok((**s).clone()),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

// Unit / `:()` — the 0-tuple. Shims that return `()` from a `&mut self`
// method marshal through this.
impl ToWat for () {
    fn to_wat(self) -> Value {
        Value::Unit
    }
}

impl FromWat for () {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::Unit => Ok(()),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "()",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

// ─── Option<T> ───────────────────────────────────────────────────────

impl<T: ToWat> ToWat for Option<T> {
    fn to_wat(self) -> Value {
        Value::Option(Arc::new(self.map(T::to_wat)))
    }
}

impl<T: FromWat> FromWat for Option<T> {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::Option(inner) => match inner.as_ref() {
                Some(x) => Ok(Some(T::from_wat(x, op, span)?)),
                None => Ok(None),
            },
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Option",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

// ─── Tuples (A,), (A,B), (A,B,C), (A,B,C,D), (A,B,C,D,E), (A,B,C,D,E,F) ─
//
// Rust tuples map directly to `Value::Tuple(Arc<Vec<Value>>)`. Each
// element is marshaled through its own `ToWat`/`FromWat` impl. Arity
// is checked at unmarshal time; an arity-mismatch surfaces as a
// `MalformedForm` error naming both expected and actual arities.
//
// The macro expands one (ToWat, FromWat) pair per listed arity.

macro_rules! impl_tuple_marshaling {
    ( $arity:expr, $( $name:ident => $idx:tt ),+ ) => {
        impl<$( $name: ToWat ),+> ToWat for ( $( $name, )+ ) {
            fn to_wat(self) -> Value {
                Value::Tuple(Arc::new(vec![
                    $( self.$idx.to_wat() ),+
                ]))
            }
        }

        impl<$( $name: FromWat ),+> FromWat for ( $( $name, )+ ) {
            fn from_wat(v: &Value, op: &'static str, span: &$crate::span::Span) -> Result<Self, RuntimeError> {
                match v {
                    Value::Tuple(items) => {
                        if items.len() != $arity {
                            return Err(RuntimeError::MalformedForm {
                                head: op.into(),
                                reason: format!(
                                    "expected tuple of arity {}; got arity {}",
                                    $arity,
                                    items.len()
                                ),
                                span: span.clone(), // arc 138 F4b: real span threaded through
                            });
                        }
                        Ok((
                            $( $name::from_wat(&items[$idx], op, span)?, )+
                        ))
                    }
                    other => Err(RuntimeError::TypeMismatch {
                        op: op.into(),
                        expected: "Tuple",
                        got: crate::runtime::ValueSnapshot::of(&other),
                        span: span.clone(), // arc 138 F4b: real span threaded through
                    }),
                }
            }
        }
    };
}

impl_tuple_marshaling!(1, A => 0);
impl_tuple_marshaling!(2, A => 0, B => 1);
impl_tuple_marshaling!(3, A => 0, B => 1, C => 2);
impl_tuple_marshaling!(4, A => 0, B => 1, C => 2, D => 3);
impl_tuple_marshaling!(5, A => 0, B => 1, C => 2, D => 3, E => 4);
impl_tuple_marshaling!(6, A => 0, B => 1, C => 2, D => 3, E => 4, F => 5);

// ─── Result<T, E> ────────────────────────────────────────────────────

impl<T: ToWat, E: ToWat> ToWat for std::result::Result<T, E> {
    fn to_wat(self) -> Value {
        let inner = match self {
            Ok(v) => Ok(v.to_wat()),
            Err(e) => Err(e.to_wat()),
        };
        Value::Result(Arc::new(inner))
    }
}

impl<T: FromWat, E: FromWat> FromWat for std::result::Result<T, E> {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::Result(r) => match r.as_ref() {
                Ok(inner) => Ok(Ok(T::from_wat(inner, op, span)?)),
                Err(inner) => Ok(Err(E::from_wat(inner, op, span)?)),
            },
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Result",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

// ─── Vec<T> ──────────────────────────────────────────────────────────

impl<T: ToWat> ToWat for Vec<T> {
    fn to_wat(self) -> Value {
        Value::Vec(Arc::new(self.into_iter().map(T::to_wat).collect()))
    }
}

impl<T: FromWat> FromWat for Vec<T> {
    fn from_wat(v: &Value, op: &'static str, span: &Span) -> Result<Self, RuntimeError> {
        match v {
            Value::Vec(items) => items
                .iter()
                .map(|x| T::from_wat(x, op, span))
                .collect::<Result<Vec<_>, _>>(),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Vector",
                got: crate::runtime::ValueSnapshot::of(&other),
                span: span.clone(), // arc 138 F4b: real span threaded through
            }),
        }
    }
}

// ─── Pass-through for Value ──────────────────────────────────────────
//
// Shims that want to take a wat Value unchanged (e.g., LruCache's
// keys and values stored as generic Value) use this. The macro emits
// `FromWat::from_wat` on every arg — for a Value-typed param, this
// impl returns a Clone of the Value.

impl ToWat for Value {
    fn to_wat(self) -> Value {
        self
    }
}

impl FromWat for Value {
    /// This impl is infallible — it accepts any `Value` without error
    /// (the only infallible `FromWat` member).
    fn from_wat(v: &Value, _op: &'static str, _span: &Span) -> Result<Self, RuntimeError> {
        Ok(v.clone())
    }
}

// ─── RustOpaque payloads ─────────────────────────────────────────────

/// The generic container for a Rust-shim-owned value in the wat
/// `Value` enum. Identified by its `:rust::...` type path; the actual
/// payload is an erased `Box<dyn Any>`, downcast by the shim's
/// dispatch code.
///
/// This is the wire format for every `:rust::*` type reaching wat via
/// the `#[wat_dispatch]` macro. Dedicated `Value` variants for
/// specific Rust types are discouraged — they duplicate the opaque
/// mechanism without buying anything. The one remaining dedicated
/// variant (`Value::crossbeam_channel__*`) predates the opaque form
/// and is specialized for the performance-sensitive channel path.
pub struct RustOpaqueInner {
    /// Full keyword path of the wat-level type, e.g.
    /// `":rust::lru::LruCache"`. Used by `FromWat` impls to reject
    /// downcasts from the wrong type.
    pub type_path: &'static str,
    /// The actual Rust value, erased. Shim authors choose the
    /// concrete type — plain `T`, `ThreadOwnedCell<T>`, etc.
    pub payload: Box<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for RustOpaqueInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RustOpaqueInner {{ type_path: {:?} }}", self.type_path)
    }
}

/// Construct an opaque wat Value wrapping a Rust payload. Shim
/// authors call this from their dispatch fns when returning a typed
/// Rust value.
pub fn make_rust_opaque<T: Any + Send + Sync>(type_path: &'static str, payload: T) -> Value {
    Value::RustOpaque(Arc::new(RustOpaqueInner {
        type_path,
        payload: Box::new(payload),
    }))
}

/// Validate `v` is a `RustOpaque` with the expected type path and return the
/// inner `Arc`. Callers pass this to `downcast_ref_opaque` for a typed
/// reference to the payload.
pub fn rust_opaque_arc(
    v: &Value,
    expected_path: &'static str,
    op: &'static str,
    span: crate::span::Span,
) -> Result<Arc<RustOpaqueInner>, RuntimeError> {
    match v {
        Value::RustOpaque(inner) => {
            if inner.type_path != expected_path {
                return Err(RuntimeError::TypeMismatch {
                    op: op.into(),
                    expected: expected_path,
                    got: crate::runtime::ValueSnapshot::unavailable(inner.type_path),
                    span,
                });
            }
            Ok(Arc::clone(inner))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: crate::runtime::ValueSnapshot::of(&other),
            span,
        }),
    }
}

/// Downcast an opaque Value's payload to a `&T` reference. The macro's
/// dispatch code calls this for each `:rust::T` argument, bypassing
/// the generic `FromWat` pathway (since opaque handles aren't cloneable
/// and often need shared-ref access, not consumed-value access).
pub fn downcast_ref_opaque<'a, T: Any>(
    inner: &'a RustOpaqueInner,
    expected_path: &'static str,
    op: &'static str,
    span: crate::span::Span,
) -> Result<&'a T, RuntimeError> {
    if inner.type_path != expected_path {
        return Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: crate::runtime::ValueSnapshot::unavailable(inner.type_path),
            span: span.clone(),
        });
    }
    inner.payload.downcast_ref::<T>().ok_or_else(|| {
        RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: crate::runtime::ValueSnapshot::unavailable("payload downcast failed — shim author misalignment"),
            span,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_roundtrip() {
        let v = 42i64.to_wat();
        let s = crate::span::Span::unknown();
        assert_eq!(i64::from_wat(&v, "test", &s).unwrap(), 42);
    }

    #[test]
    fn f64_roundtrip() {
        let v = 2.5f64.to_wat();
        let s = crate::span::Span::unknown();
        assert_eq!(f64::from_wat(&v, "test", &s).unwrap(), 2.5);
    }

    #[test]
    fn bool_roundtrip() {
        let s = crate::span::Span::unknown();
        assert!(bool::from_wat(&true.to_wat(), "t", &s).unwrap());
        assert!(!bool::from_wat(&false.to_wat(), "t", &s).unwrap());
    }

    #[test]
    fn string_roundtrip() {
        let v = "hello".to_string().to_wat();
        let s = crate::span::Span::unknown();
        assert_eq!(String::from_wat(&v, "test", &s).unwrap(), "hello");
    }

    #[test]
    fn unit_roundtrip() {
        let v = ().to_wat();
        let s = crate::span::Span::unknown();
        assert!(matches!(<()>::from_wat(&v, "test", &s), Ok(())));
    }

    #[test]
    fn option_some_roundtrip() {
        let v: Value = Some(7i64).to_wat();
        let s = crate::span::Span::unknown();
        let back: Option<i64> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, Some(7));
    }

    #[test]
    fn option_none_roundtrip() {
        let v: Value = Option::<i64>::None.to_wat();
        let s = crate::span::Span::unknown();
        let back: Option<i64> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, None);
    }

    #[test]
    fn vec_of_i64_roundtrip() {
        let v: Value = vec![1i64, 2, 3].to_wat();
        let s = crate::span::Span::unknown();
        let back: Vec<i64> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, vec![1, 2, 3]);
    }

    #[test]
    fn vec_of_strings_roundtrip() {
        let v: Value = vec!["a".to_string(), "b".to_string()].to_wat();
        let s = crate::span::Span::unknown();
        let back: Vec<String> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn empty_vec_roundtrip() {
        let v: Value = Vec::<i64>::new().to_wat();
        let s = crate::span::Span::unknown();
        let back: Vec<i64> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert!(back.is_empty());
    }

    #[test]
    fn vec_of_options_roundtrip() {
        let v: Value = vec![Some(1i64), None, Some(3)].to_wat();
        let s = crate::span::Span::unknown();
        let back: Vec<Option<i64>> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, vec![Some(1), None, Some(3)]);
    }

    #[test]
    fn vec_from_wrong_value_type_fails() {
        let v = Value::i64(5);
        let s = crate::span::Span::unknown();
        let err = <Vec<i64> as FromWat>::from_wat(&v, "test", &s).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn tuple_2_roundtrip() {
        let v: Value = (42i64, "hello".to_string()).to_wat();
        let s = crate::span::Span::unknown();
        let back: (i64, String) = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, (42, "hello".to_string()));
    }

    #[test]
    fn tuple_3_roundtrip() {
        let v: Value = (1i64, true, 2.5f64).to_wat();
        let s = crate::span::Span::unknown();
        let back: (i64, bool, f64) = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, (1, true, 2.5));
    }

    #[test]
    fn tuple_4_roundtrip() {
        let v: Value = (1i64, 2i64, 3i64, 4i64).to_wat();
        let s = crate::span::Span::unknown();
        let back: (i64, i64, i64, i64) = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, (1, 2, 3, 4));
    }

    #[test]
    fn tuple_nested_with_option_vec() {
        let v: Value = (Some(7i64), vec![1i64, 2, 3]).to_wat();
        let s = crate::span::Span::unknown();
        let back: (Option<i64>, Vec<i64>) = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, (Some(7), vec![1, 2, 3]));
    }

    #[test]
    fn tuple_arity_mismatch_rejected() {
        let v: Value = (1i64, 2i64, 3i64).to_wat();
        let s = crate::span::Span::unknown();
        let err = <(i64, i64) as FromWat>::from_wat(&v, "test", &s).unwrap_err();
        match err {
            RuntimeError::MalformedForm { reason, .. } => {
                assert!(reason.contains("arity 2"));
                assert!(reason.contains("arity 3"));
            }
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn tuple_from_non_tuple_value_fails() {
        let v = Value::i64(1);
        let s = crate::span::Span::unknown();
        let err = <(i64, i64) as FromWat>::from_wat(&v, "test", &s).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn result_ok_roundtrip() {
        let v: Value = std::result::Result::<i64, String>::Ok(7).to_wat();
        let s = crate::span::Span::unknown();
        let back: std::result::Result<i64, String> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, Ok(7));
    }

    #[test]
    fn result_err_roundtrip() {
        let v: Value = std::result::Result::<i64, String>::Err("boom".into()).to_wat();
        let s = crate::span::Span::unknown();
        let back: std::result::Result<i64, String> = FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, Err("boom".to_string()));
    }

    #[test]
    fn result_nested_option_and_vec() {
        let v: Value =
            std::result::Result::<Option<i64>, Vec<String>>::Ok(Some(5)).to_wat();
        let s = crate::span::Span::unknown();
        let back: std::result::Result<Option<i64>, Vec<String>> =
            FromWat::from_wat(&v, "test", &s).unwrap();
        assert_eq!(back, Ok(Some(5)));
    }

    #[test]
    fn result_from_non_result_fails() {
        let v = Value::i64(1);
        let s = crate::span::Span::unknown();
        let err =
            <std::result::Result<i64, String> as FromWat>::from_wat(&v, "test", &s).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn value_passthrough() {
        let v = Value::i64(99);
        let s = crate::span::Span::unknown();
        let back = Value::from_wat(&v, "test", &s).unwrap();
        assert!(matches!(back, Value::i64(99)));
    }

    #[test]
    fn type_mismatch_surfaces_op_name() {
        let v = Value::String(Arc::new("not an i64".into()));
        let s = crate::span::Span::unknown();
        let err = i64::from_wat(&v, ":rust::test::method", &s).unwrap_err();
        match err {
            RuntimeError::TypeMismatch { op, expected, got, .. } => {
                assert_eq!(op, ":rust::test::method");
                assert_eq!(expected, "i64");
                assert_eq!(got.type_name, "wat::core::String");
            }
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn opaque_round_trip_plain_payload() {
        struct Widget {
            tag: i64,
        }
        let s = crate::span::Span::unknown();
        let v = make_rust_opaque(":rust::test::Widget", Widget { tag: 7 });
        let inner = rust_opaque_arc(&v, ":rust::test::Widget", ":test", s.clone()).unwrap();
        let w: &Widget = downcast_ref_opaque(&inner, ":rust::test::Widget", ":test", s).unwrap();
        assert_eq!(w.tag, 7);
    }

    #[test]
    fn opaque_wrong_type_path_rejected() {
        struct A;
        let s = crate::span::Span::unknown();
        let v = make_rust_opaque(":rust::test::A", A);
        let err = rust_opaque_arc(&v, ":rust::test::B", ":test", s).unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn opaque_wrong_payload_type_fails_downcast() {
        struct Actual {
            _t: i64,
        }
        #[derive(Debug)]
        struct ExpectedWrong;
        let s = crate::span::Span::unknown();
        let v = make_rust_opaque(":rust::test::Mixed", Actual { _t: 1 });
        let inner = rust_opaque_arc(&v, ":rust::test::Mixed", ":test", s.clone()).unwrap();
        let result = downcast_ref_opaque::<ExpectedWrong>(&inner, ":rust::test::Mixed", ":test", s);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            RuntimeError::TypeMismatch { .. }
        ));
    }
}
