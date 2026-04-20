//! Marshaling between wat `Value` and Rust types.
//!
//! The `#[wat_dispatch]` macro (see
//! `docs/wat-dispatch-macro-design-2026-04-19.md`) emits calls into
//! these traits to convert arguments (wat → Rust) and returns
//! (Rust → wat). The traits are userland-extensible — a shim author
//! can implement `ToWat` / `FromWat` for any type their shim exposes.
//!
//! # What this module provides
//!
//! - [`ToWat`] / [`FromWat`] — the conversion traits.
//! - Blanket impls for primitives (`i64`, `f64`, `bool`, `String`).
//! - An impl pair for `Option<T>` (the only compound type lru needs;
//!   `Vec`/tuple impls land when a caller demands them).
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
//! downcasted payload.

use std::any::Any;
use std::sync::Arc;

use crate::runtime::{RuntimeError, Value};

/// Convert a Rust value into a wat [`Value`]. Used by shim dispatch
/// fns when marshaling a method's return to wat.
pub trait ToWat {
    fn to_wat(self) -> Value;
}

/// Convert a wat [`Value`] into a Rust value. Used by shim dispatch
/// fns when marshaling wat arguments into Rust method params.
///
/// The `op` parameter names the wat-level operation so error messages
/// can point the user at the exact call site.
pub trait FromWat: Sized {
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError>;
}

// ─── Primitive impls ─────────────────────────────────────────────────

impl ToWat for i64 {
    fn to_wat(self) -> Value {
        Value::i64(self)
    }
}

impl FromWat for i64 {
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::i64(n) => Ok(*n),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: other.type_name(),
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
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::f64(x) => Ok(*x),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "f64",
                got: other.type_name(),
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
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::bool(b) => Ok(*b),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "bool",
                got: other.type_name(),
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
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::String(s) => Ok((**s).clone()),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: other.type_name(),
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
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::Unit => Ok(()),
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "()",
                got: other.type_name(),
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
    fn from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError> {
        match v {
            Value::Option(inner) => match inner.as_ref() {
                Some(x) => Ok(Some(T::from_wat(x, op)?)),
                None => Ok(None),
            },
            other => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "Option",
                got: other.type_name(),
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
    fn from_wat(v: &Value, _op: &'static str) -> Result<Self, RuntimeError> {
        Ok(v.clone())
    }
}

// ─── RustOpaque payloads ─────────────────────────────────────────────

/// The generic container for a Rust-shim-owned value in the wat
/// `Value` enum. Identified by its `:rust::...` type path; the actual
/// payload is an erased `Box<dyn Any>`, downcast by the shim's
/// dispatch code.
///
/// This is the wire format for ALL `:rust::*` types except those that
/// have their own dedicated `Value` variant (currently only
/// `Value::rust__lru__LruCache`, which gets replaced by the
/// opaque-payload form when the macro regenerates the lru shim —
/// task #195).
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

/// Extract a reference to the payload inside an opaque Value,
/// checking that the type path matches. Returns the inner `Arc` so
/// callers can downcast via `Arc::<RustOpaqueInner>::downcast` …
/// actually no — callers use `downcast_ref_opaque` below.
pub fn rust_opaque_arc(
    v: &Value,
    expected_path: &'static str,
    op: &'static str,
) -> Result<Arc<RustOpaqueInner>, RuntimeError> {
    match v {
        Value::RustOpaque(inner) => {
            if inner.type_path != expected_path {
                return Err(RuntimeError::TypeMismatch {
                    op: op.into(),
                    expected: expected_path,
                    got: inner.type_path,
                });
            }
            Ok(Arc::clone(inner))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: other.type_name(),
        }),
    }
}

/// Wrapper for single-thread-owned mutable state. Generic version of
/// the hand-written `LruCacheCell` pattern. The `#[wat_dispatch]`
/// macro uses this to wrap `Self` returns when the annotated impl
/// block declares `scope = "thread_owned"`.
///
/// Ownership invariant: every `.with_mut` / `.with_ref` call asserts
/// `thread::current().id() == self.owner` before dereferencing the
/// `UnsafeCell`. Cross-thread access errors with a clear
/// `MalformedForm`. Zero Mutex.
///
/// # Safety
///
/// The `unsafe impl Send + Sync` is upheld by the thread-id guard.
/// Only one thread can reach the `UnsafeCell`; the interpreter is
/// single-threaded within that thread and the `with_*` closures do
/// not re-enter Value evaluation against the same cell.
pub struct ThreadOwnedCell<T: Send> {
    owner: std::thread::ThreadId,
    cell: std::cell::UnsafeCell<T>,
}

impl<T: Send> std::fmt::Debug for ThreadOwnedCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreadOwnedCell {{ owner: {:?} }}", self.owner)
    }
}

// Safety: see type-level docs.
unsafe impl<T: Send> Send for ThreadOwnedCell<T> {}
unsafe impl<T: Send> Sync for ThreadOwnedCell<T> {}

impl<T: Send> ThreadOwnedCell<T> {
    /// Create a new cell bound to the current thread.
    pub fn new(inner: T) -> Self {
        Self {
            owner: std::thread::current().id(),
            cell: std::cell::UnsafeCell::new(inner),
        }
    }

    fn ensure_owner(&self, op: &'static str) -> Result<(), RuntimeError> {
        if std::thread::current().id() != self.owner {
            return Err(RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "thread-owned value crossed thread boundary \
                     (owner: {:?}, current: {:?})",
                    self.owner,
                    std::thread::current().id()
                ),
            });
        }
        Ok(())
    }

    /// Borrow the inner value mutably after asserting ownership.
    pub fn with_mut<R>(
        &self,
        op: &'static str,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op)?;
        // Safety: thread-owner invariant checked above.
        Ok(unsafe { f(&mut *self.cell.get()) })
    }

    /// Borrow the inner value immutably after asserting ownership.
    /// (Kept for `&self` methods under `scope = "thread_owned"`.)
    pub fn with_ref<R>(
        &self,
        op: &'static str,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op)?;
        // Safety: thread-owner invariant checked above.
        Ok(unsafe { f(&*self.cell.get()) })
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
) -> Result<&'a T, RuntimeError> {
    if inner.type_path != expected_path {
        return Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: inner.type_path,
        });
    }
    inner.payload.downcast_ref::<T>().ok_or_else(|| {
        RuntimeError::TypeMismatch {
            op: op.into(),
            expected: expected_path,
            got: "(payload downcast failed — shim author misalignment)",
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_roundtrip() {
        let v = 42i64.to_wat();
        assert_eq!(i64::from_wat(&v, "test").unwrap(), 42);
    }

    #[test]
    fn f64_roundtrip() {
        let v = 2.5f64.to_wat();
        assert_eq!(f64::from_wat(&v, "test").unwrap(), 2.5);
    }

    #[test]
    fn bool_roundtrip() {
        assert!(bool::from_wat(&true.to_wat(), "t").unwrap());
        assert!(!bool::from_wat(&false.to_wat(), "t").unwrap());
    }

    #[test]
    fn string_roundtrip() {
        let v = "hello".to_string().to_wat();
        assert_eq!(String::from_wat(&v, "test").unwrap(), "hello");
    }

    #[test]
    fn unit_roundtrip() {
        let v = ().to_wat();
        assert!(matches!(<()>::from_wat(&v, "test"), Ok(())));
    }

    #[test]
    fn option_some_roundtrip() {
        let v: Value = Some(7i64).to_wat();
        let back: Option<i64> = FromWat::from_wat(&v, "test").unwrap();
        assert_eq!(back, Some(7));
    }

    #[test]
    fn option_none_roundtrip() {
        let v: Value = Option::<i64>::None.to_wat();
        let back: Option<i64> = FromWat::from_wat(&v, "test").unwrap();
        assert_eq!(back, None);
    }

    #[test]
    fn value_passthrough() {
        let v = Value::i64(99);
        let back = Value::from_wat(&v, "test").unwrap();
        assert!(matches!(back, Value::i64(99)));
    }

    #[test]
    fn type_mismatch_surfaces_op_name() {
        let v = Value::String(Arc::new("not an i64".into()));
        let err = i64::from_wat(&v, ":rust::test::method").unwrap_err();
        match err {
            RuntimeError::TypeMismatch { op, expected, got } => {
                assert_eq!(op, ":rust::test::method");
                assert_eq!(expected, "i64");
                assert_eq!(got, "String");
            }
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn opaque_round_trip_plain_payload() {
        struct Widget {
            tag: i64,
        }
        let v = make_rust_opaque(":rust::test::Widget", Widget { tag: 7 });
        let inner = rust_opaque_arc(&v, ":rust::test::Widget", ":test").unwrap();
        let w: &Widget = downcast_ref_opaque(&inner, ":rust::test::Widget", ":test").unwrap();
        assert_eq!(w.tag, 7);
    }

    #[test]
    fn opaque_wrong_type_path_rejected() {
        struct A;
        let v = make_rust_opaque(":rust::test::A", A);
        let err = rust_opaque_arc(&v, ":rust::test::B", ":test").unwrap_err();
        assert!(matches!(err, RuntimeError::TypeMismatch { .. }));
    }

    #[test]
    fn opaque_wrong_payload_type_fails_downcast() {
        struct Actual {
            _t: i64,
        }
        #[derive(Debug)]
        struct ExpectedWrong;
        let v = make_rust_opaque(":rust::test::Mixed", Actual { _t: 1 });
        let inner = rust_opaque_arc(&v, ":rust::test::Mixed", ":test").unwrap();
        let result = downcast_ref_opaque::<ExpectedWrong>(&inner, ":rust::test::Mixed", ":test");
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            RuntimeError::TypeMismatch { .. }
        ));
    }
}
