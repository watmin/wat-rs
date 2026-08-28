//! `:wat::holon::Hologram/*` intrinsics — arc 255 Stone HOME-8 (the VSA
//! surface gets a home), registry half.
//!
//! `Hologram` is a therm-routed coordinate-cell store: bind an approximate
//! (`HolonAST`) key to a value, and later recover the value for a
//! near-identical probe key via cosine similarity + coincident-floor
//! matching. The seven verbs below are its whole population — construct,
//! mutate (`put`/`remove`), read (`get`/`find`/`len`/`capacity`). Each is a
//! thin binding shim: it evaluates its wat-side args, unwraps the native
//! `ThreadOwnedCell<Hologram>` handle via `require_hologram`, and delegates
//! to the algebra in [`crate::holon::hologram`] (absorbed from the
//! top-level `src/hologram.rs` by the sibling strike, `d43f758870`).
//!
//! **`@Category Resource`, uniformly** — a `Hologram` is a native handle
//! whose lifetime is tracked outside value scope (`ThreadOwnedCell`), the
//! same framing `intrinsic/kernel/resource.rs` uses for `HandlePool`.
//! `make` and the two mutators (`put`, `remove`) are `@Purity Effectful`
//! (they mint or mutate the handle's interior state via `with_mut`); the
//! four readers (`get`, `find`, `len`, `capacity`) are `@Purity Pure` reads
//! via `with_ref` — the same split `resource.rs` draws between
//! `HandlePool::new`/`pop` (Effectful) and `HandlePool::finish`'s `len()`
//! read (Pure).
//!
//! None of these four are among the four rete-classified holon verbs
//! (`src/rete/purity.rs:647`) — STOP-7 forbids adding to that builder-ruled
//! set, and this carve does not.
//!
//! arc 255 Stone H-1a — each handler declares its real fixed arity
//! (`&WatAST` per parameter) instead of `args: &[WatAST]`; the
//! hand-rolled arity checks are gone, replaced by the check
//! `#[wat_intrinsic]` now generates from the declared parameter count. See
//! `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-H-holon-adopts-the-kernels-interface.md`.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::holon::*;
use crate::runtime::{eval_inner, require_encoding_ctx};
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value, ValueSnapshot, RuntimeError, RuntimeErrorKind, AggregateValue};

/// `(:wat::holon::Hologram/make filter)` -> a fresh, empty `Hologram` sized
/// to the program's encoding dimension, routing lookups through `filter`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     filter [:wat::core::f64 :-> :wat::core::bool] a therm-routing filter function
/// @ret     :wat::holon::Hologram a fresh, empty coordinate-cell store
/// @example-norun (:wat::holon::Hologram/make (fn (x) true)) #=> #wat.holon/Hologram{}
#[wat_intrinsic(":wat::holon::Hologram/make")]
pub(crate) fn eval_hologram_make(
    filter: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/make";
    let filter = require_fn(OP, eval_inner(filter, env, sym)?.value_owned())?;
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let h = crate::holon::hologram::Hologram::make(ctx.dim_count, filter);
    Ok(Value::Hologram(Arc::new(
        crate::rust_deps::ThreadOwnedCell::new(h),
    )))
}


/// `(:wat::holon::Hologram/put store key val)` -> `:Unit`. Binds `key` to
/// `val` in `store`, mutating it in place.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store mutated
/// @arg     key :wat::holon::HolonAST the key HolonAST
/// @arg     val :wat::holon::HolonAST the value HolonAST
/// @ret     :wat::core::nil always `Unit`
/// @example-norun (:wat::holon::Hologram/put store key val) #=> nil
#[wat_intrinsic(":wat::holon::Hologram/put")]
pub(crate) fn eval_hologram_put(
    store: &WatAST,
    key: &WatAST,
    val: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/put";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let key = match eval_inner(key, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => (*h).clone(),
        other => {
            return Err(RuntimeError::new(
                key.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let val = match eval_inner(val, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => (*h).clone(),
        other => {
            return Err(RuntimeError::new(
                val.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    store.with_mut(OP, list_span.clone(), |s| s.put(key, val))?;
    Ok(Value::Unit)
}


/// `(:wat::holon::Hologram/get store probe)` -> `(:Option :- [wat::holon::HolonAST])`.
/// Looks up the value bound to a key coincident with `probe`, or `None` if
/// no stored key matches within the coincident floor.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store probed
/// @arg     probe :wat::holon::HolonAST the probe key
/// @ret     (:wat::core::Option :- [:wat::holon::HolonAST]) the matched value, or `None`
/// @example (:wat::holon::Hologram/get (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)) (:wat::holon::leaf "role")) #=> (:wat::holon::Hologram/get (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)) (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::Hologram/get")]
pub(crate) fn eval_hologram_get(
    store: &WatAST,
    probe: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/get";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let probe = match eval_inner(probe, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                probe.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let span = list_span.clone();
    let result = store.with_ref(OP, |s| s.get(&probe, sym, span.clone(), &ctx.encoders))??;
    match result {
        Some(val) => Ok(Value::Option(Arc::new(Some(Value::holon__HolonAST(
            Arc::new(val),
        ))))),
        None => Ok(Value::Option(Arc::new(None))),
    }
}


/// `(:wat::holon::Hologram/find store probe)` -> `(:Option :- [wat::holon::Match])`.
/// Like `get`, but returns both the matched stored key and its value as a
/// `wat::holon::Match` record instead of the value alone.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store probed
/// @arg     probe :wat::holon::HolonAST the probe key
/// @ret     (:wat::core::Option :- [:wat::holon::Match]) the matched (key, value) pair as a `wat::holon::Match`, or `None`
/// @example (:wat::holon::Hologram/find (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)) (:wat::holon::leaf "role")) #=> (:wat::holon::Hologram/find (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)) (:wat::holon::leaf "role"))
#[wat_intrinsic(":wat::holon::Hologram/find")]
pub(crate) fn eval_hologram_find(
    store: &WatAST,
    probe: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/find";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let probe = match eval_inner(probe, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => h,
        other => {
            return Err(RuntimeError::new(
                probe.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let ctx = require_encoding_ctx(OP, sym, list_span)?;
    let span = list_span.clone();
    let result = store.with_ref(OP, |s| s.find(&probe, sym, span.clone(), &ctx.encoders))??;
    match result {
        Some((k, v)) => Ok(Value::Option(Arc::new(Some(Value::Aggregate(Arc::new(
            AggregateValue::record(
                "wat::holon::Match".into(),
                match_names(),
                Arc::new(vec![
                    Value::holon__HolonAST(Arc::new(k)),
                    Value::holon__HolonAST(Arc::new(v)),
                ]),
            ),
        )))))),
        None => Ok(Value::Option(Arc::new(None))),
    }
}


/// `(:wat::holon::Hologram/remove store key)` -> `(:Option :- [wat::holon::HolonAST])`.
/// Removes the entry bound to `key` (exact match), returning its value if
/// present.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store mutated
/// @arg     key :wat::holon::HolonAST the key to remove
/// @ret     (:wat::core::Option :- [:wat::holon::HolonAST]) the removed value, or `None` if absent
/// @example-norun (:wat::holon::Hologram/remove store key) #=> (:wat::core::Option :- [wat::holon::HolonAST])
#[wat_intrinsic(":wat::holon::Hologram/remove")]
pub(crate) fn eval_hologram_remove(
    store: &WatAST,
    key: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/remove";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let key = match eval_inner(key, env, sym)?.value_owned() {
        Value::holon__HolonAST(h) => (*h).clone(),
        other => {
            return Err(RuntimeError::new(
                key.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::holon::HolonAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let removed = store.with_mut(OP, list_span.clone(), |s| s.remove(&key))?;
    match removed {
        Some(val) => Ok(Value::Option(Arc::new(Some(Value::holon__HolonAST(
            Arc::new(val),
        ))))),
        None => Ok(Value::Option(Arc::new(None))),
    }
}


/// `(:wat::holon::Hologram/len store)` -> `:i64`. The number of entries
/// currently stored.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store probed
/// @ret     :wat::core::i64 the number of entries currently stored
/// @example (:wat::holon::Hologram/len (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true))) #=> (:wat::holon::Hologram/len (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)))
#[wat_intrinsic(":wat::holon::Hologram/len")]
pub(crate) fn eval_hologram_len(
    store: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — infallible: `require_hologram`'s TypeMismatch locates via `rust_caller_span!()` inside that helper, and `with_ref`'s own `len()` read has no error path
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/len";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let n = store.with_ref(OP, |s| s.len() as i64)?;
    Ok(Value::i64(n))
}


/// `(:wat::holon::Hologram/capacity store)` -> `:i64`. The maximum number
/// of entries `store` can hold (Kanerva capacity at its dimension).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Resource
/// @arg     store :wat::holon::Hologram the store probed
/// @ret     :wat::core::i64 the store's Kanerva capacity
/// @example (:wat::holon::Hologram/capacity (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true))) #=> (:wat::holon::Hologram/capacity (:wat::holon::Hologram/make (:wat::core::fn [_x <- :wat::core::f64] -> :wat::core::bool true)))
#[wat_intrinsic(":wat::holon::Hologram/capacity")]
pub(crate) fn eval_hologram_capacity(
    store: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — infallible: `require_hologram`'s TypeMismatch locates via `rust_caller_span!()` inside that helper, and `with_ref`'s own `capacity()` read has no error path
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::holon::Hologram/capacity";
    let store = require_hologram(OP, eval_inner(store, env, sym)?.value_owned())?;
    let cap = store.with_ref(OP, |s| s.capacity() as i64)?;
    Ok(Value::i64(cap))
}

