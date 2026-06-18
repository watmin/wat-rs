//! Vector/List-specific utility ops for the collection dispatch home.
//!
//! Contains the ~15 seq-HOF and helper functions (map, filter, foldl, foldr,
//! sort' (primitive comparator-sort), reverse, range, take, drop, last,
//! find-last-index, zip, window, remove-at, map-with-index). These are NOT
//! container-polymorphic dispatch — they are Vector-specific utilities (all
//! enforce `Value::Vec` via `require_vec`).
//! The four ops in the `:wat::std::list::` namespace (zip, window, remove-at,
//! map-with-index) are named `eval_vec_*` here to mirror the ENFORCED value type.
//! `rest` was moved to `eval.rs` (container-polymorphic; Vec/List/WatAST-form arms).
//! Their dispatch arms in `dispatch_keyword_head_value` redirect here.
//!
//! See `src/collection/mod.rs` and `docs/DISPATCH.md` for the full doctrine.

use crate::ast::WatAST;
use crate::runtime::{
    apply_function, eval_inner, require_i64, require_vec, EvalBreak, Environment,
    RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use std::sync::Arc;

pub(crate) fn eval_vec_reverse(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::reverse".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::Vec(xs) => {
            let mut out = (*xs).clone();
            out.reverse();
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc-278-0c — PersistentVector: reverse returns a new PersistentVector (type-preserving).
        Value::wat__core__PersistentVector(pv) => {
            let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for elem in pv.iter().collect::<Vec<_>>().into_iter().rev() {
                out = out.push_back(elem.clone());
            }
            Ok(Value::wat__core__PersistentVector(out))
        }
        other => Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::reverse".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::range start end)` → `Vec<i64>`. Two-arg only; the
/// spec-frozen shape maps to Rust's `start..end` exactly. Callers
/// write `(range 0 n)` explicitly for 0..n.
pub(crate) fn eval_vec_range(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::range".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let start = require_i64(":wat::core::range", eval_inner(&args[0], env, sym)?.value_owned())?;
    let end = require_i64(":wat::core::range", eval_inner(&args[1], env, sym)?.value_owned())?;
    let items: Vec<Value> = if start <= end {
        (start..end).map(Value::i64).collect()
    } else {
        Vec::new()
    };
    Ok(Value::Vec(Arc::new(items)))
}

/// `(:wat::core::take xs n)` → `Vec<T>`. First `n` elements; if
/// `n >= xs.len()`, returns the full Vec. Negative `n` clamps to 0
/// (empty Vec).
pub(crate) fn eval_vec_take(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::take".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(":wat::core::take", eval_inner(&args[1], env, sym)?.value_owned())?;
    match coll {
        Value::Vec(xs) => {
            let cap = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
            let out: Vec<Value> = xs.iter().take(cap).cloned().collect();
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc-278-0c — PersistentVector: take n returns a new PersistentVector (type-preserving).
        Value::wat__core__PersistentVector(pv) => {
            let cap = if n <= 0 { 0 } else { (n as usize).min(pv.len()) };
            let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for elem in pv.iter().take(cap) {
                out = out.push_back(elem.clone());
            }
            Ok(Value::wat__core__PersistentVector(out))
        }
        other => Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::take".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::drop xs n)` → `Vec<T>`. Skip first `n` elements. If
/// `n >= xs.len()`, returns an empty Vec. Negative `n` clamps to 0
/// (returns the full Vec).
pub(crate) fn eval_vec_drop(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::drop".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(":wat::core::drop", eval_inner(&args[1], env, sym)?.value_owned())?;
    match coll {
        Value::Vec(xs) => {
            let skip = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
            let out: Vec<Value> = xs.iter().skip(skip).cloned().collect();
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc-278-0c — PersistentVector: drop n returns a new PersistentVector (type-preserving).
        Value::wat__core__PersistentVector(pv) => {
            let skip = if n <= 0 { 0 } else { (n as usize).min(pv.len()) };
            let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for elem in pv.iter().skip(skip) {
                out = out.push_back(elem.clone());
            }
            Ok(Value::wat__core__PersistentVector(out))
        }
        other => Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::drop".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::sort' less? xs)` → `Vec<T>` — the primitive comparator-sort engine.
///
/// Arc 251 Stone: renamed from `sort-by` to `sort'` (primitive convention, like
/// `spawn-program'`). The wat-level `sort` and `sort-by` defclauses in `core.wat`
/// build on this primitive.
///
/// `less?` is a callable `:fn(T, T) -> :bool`; it returns true iff
/// the first arg is "less than" the second under the desired order.
///
///   asc:  `(fn [a b] -> :bool (:wat::core::< a b))`
///   desc: `(fn [a b] -> :bool (:wat::core::> a b))`
///
/// Stable. Wraps Rust's `Vec::sort_by`. Common Lisp / Clojure
/// tradition — predicate-driven ordering with the user owning the
/// asc/desc choice. The two-sided test (calling `less?` for both
/// `(a,b)` and `(b,a)` to distinguish Equal from Less/Greater) keeps
/// stable-sort semantics honest; the doubled call count is amortized
/// against O(n log n) — for the lab's bounded windows it's
/// negligible.
// rune:temperare(simplicity-win) — two-sided less? calls (up to 2× apply_function per comparison)
// preserve the (T,T)->bool predicate interface the rest of the stdlib uses; a three-way comparator
// would halve the call count but requires a new predicate protocol. Cost ceiling: sort runs on
// config-bounded windows (N ≤ a few hundred), so O(2N log N) interpreter re-entries stay negligible.
pub(crate) fn eval_vec_sort_by(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::sort'";
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (sort' cmp xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let xs = require_vec(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let mut sorted: Vec<Value> = (*xs).clone();
    let mut sort_err: Option<EvalBreak> = None;
    sorted.sort_by(|a, b| {
        use std::cmp::Ordering;
        if sort_err.is_some() {
            return Ordering::Equal;
        }
        let call = |x: &Value, y: &Value| -> Result<bool, EvalBreak> {
            let v = apply_function(
                func.clone(),
                vec![x.clone(), y.clone()],
                sym,
                crate::rust_caller_span!(),
            ).map_err(EvalBreak::from)?;
            match v {
                Value::bool(b) => Ok(b),
                other => Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "bool",
                    got: Box::new(ValueSnapshot::of(&other)),
                } }.into()),
            }
        };
        let ab = match call(a, b) {
            Ok(v) => v,
            Err(e) => {
                sort_err = Some(e);
                return Ordering::Equal;
            }
        };
        if ab {
            return Ordering::Less;
        }
        let ba = match call(b, a) {
            Ok(v) => v,
            Err(e) => {
                sort_err = Some(e);
                return Ordering::Equal;
            }
        };
        if ba {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    if let Some(e) = sort_err {
        return Err(e);
    }
    Ok(Value::Vec(Arc::new(sorted)))
}

/// `(:wat::core::map f xs)` → `Vec<U>`. Calls `f` on each element.
/// `f` must be a callable Value (fn or define-registered).
/// Arc 247: fn-first — (map f xs).
pub(crate) fn eval_vec_map(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::map".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (map f xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let coll = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::map".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    match coll {
        Value::Vec(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for x in xs.iter() {
                out.push(apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())?);
            }
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc-278-0c — PersistentVector: map returns a new PersistentVector (type-preserving).
        Value::wat__core__PersistentVector(pv) => {
            let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for x in pv.iter() {
                out = out.push_back(apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())?);
            }
            Ok(Value::wat__core__PersistentVector(out))
        }
        other => Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::map".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::foldl f init xs)` → acc. `f : (acc, item) → acc`.
/// Left-associative: `f(f(f(init, x0), x1), x2)`. Sequential's driver.
/// Arc 247: fn-first — (foldl f init xs).
/// `:wat::core::foldr` ships alongside — see [`eval_vec_foldr`].
pub(crate) fn eval_vec_foldl(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::foldl".into(),
            expected: 3,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (foldl f init xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
    let coll = eval_inner(&args[2], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::foldl".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    match coll {
        Value::Vec(xs) => {
            for x in xs.iter() {
                acc = apply_function(func.clone(), vec![acc, x.clone()], sym, crate::rust_caller_span!())?;
            }
            Ok(acc)
        }
        // Arc-278-0c — PersistentVector: foldl iterates pv left-to-right; returns the accumulator.
        Value::wat__core__PersistentVector(pv) => {
            for x in pv.iter() {
                acc = apply_function(func.clone(), vec![acc, x.clone()], sym, crate::rust_caller_span!())?;
            }
            Ok(acc)
        }
        other => Err(RuntimeError { span: args[2].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::foldl".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::foldr f init xs)` → acc. Right-associative fold.
/// `f(x0, f(x1, f(..., f(xn, init))))`. Iterates the Vec in reverse
/// so the call stack is bounded by iteration, not recursion.
/// Arc 247: fn-first — (foldr f init xs).
pub(crate) fn eval_vec_foldr(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::foldr".into(),
            expected: 3,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (foldr f init xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
    let coll = eval_inner(&args[2], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::foldr".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    match coll {
        Value::Vec(xs) => {
            for x in xs.iter().rev() {
                acc = apply_function(func.clone(), vec![x.clone(), acc], sym, crate::rust_caller_span!())?;
            }
            Ok(acc)
        }
        // Arc-278-0c — PersistentVector: foldr iterates pv in reverse; returns the accumulator.
        Value::wat__core__PersistentVector(pv) => {
            let elems: Vec<&Value> = pv.iter().collect();
            for x in elems.into_iter().rev() {
                acc = apply_function(func.clone(), vec![x.clone(), acc], sym, crate::rust_caller_span!())?;
            }
            Ok(acc)
        }
        other => Err(RuntimeError { span: args[2].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::foldr".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::core::filter pred xs)` → `Vec<T>`. Keeps elements for
/// which `pred` returns `:bool true`. `pred` signature: `T -> :bool`.
/// Arc 247: fn-first — (filter pred xs).
pub(crate) fn eval_vec_filter(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::filter".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (filter pred xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let coll = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::filter".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    match coll {
        Value::Vec(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            for x in xs.iter() {
                match apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())? {
                    Value::bool(true) => out.push(x.clone()),
                    Value::bool(false) => {}
                    other => {
                        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::filter".into(),
                            expected: "bool",
                            got: Box::new(ValueSnapshot::of(&other))
                        } }.into());
                    }
                }
            }
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc-278-0c — PersistentVector: filter keeps elements where pred holds; returns PersistentVector.
        Value::wat__core__PersistentVector(pv) => {
            let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for x in pv.iter() {
                match apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())? {
                    Value::bool(true) => { out = out.push_back(x.clone()); }
                    Value::bool(false) => {}
                    other => {
                        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::filter".into(),
                            expected: "bool",
                            got: Box::new(ValueSnapshot::of(&other))
                        } }.into());
                    }
                }
            }
            Ok(Value::wat__core__PersistentVector(out))
        }
        other => Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::filter".into(),
            expected: "wat::core::Vector or wat::core::PersistentVector",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::std::list::zip xs ys)` → `Vec<(T,U)>`. Short-circuits at
/// the shorter input's length (matches Rust's `xs.iter().zip(ys)`).
///
/// The wat-level op lives in the `:wat::std::list::` namespace (surface contract, unchanged).
/// This Rust function is named `eval_vec_zip` to mirror the ENFORCED value type: both inputs
/// must be `Value::Vec` (enforced by `require_vec`); actual `Value::wat__core__List` values
/// are rejected at runtime.
pub(crate) fn eval_vec_zip(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::std::list::zip".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::std::list::zip", eval_inner(&args[0], env, sym)?.value_owned())?;
    let ys = require_vec(":wat::std::list::zip", eval_inner(&args[1], env, sym)?.value_owned())?;
    let n = xs.len().min(ys.len());
    let mut out = Vec::with_capacity(n);
    for (x, y) in xs.iter().zip(ys.iter()).take(n) {
        out.push(Value::Tuple(Arc::new(vec![x.clone(), y.clone()])));
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::std::list::window xs n)` → `Vec<Vec<T>>`. Sliding window
/// of size `n`; maps to Rust's `slice.windows(n)`. `n <= 0` returns
/// an empty Vec. `n > xs.len()` returns an empty Vec (no full
/// window fits) — matches Rust's behavior.
///
/// The wat-level op lives in the `:wat::std::list::` namespace (surface contract, unchanged).
/// This Rust function is named `eval_vec_window` to mirror the ENFORCED value type: input
/// must be `Value::Vec` (enforced by `require_vec`); actual `Value::wat__core__List` values
/// are rejected at runtime.
pub(crate) fn eval_vec_window(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::std::list::window".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::std::list::window", eval_inner(&args[0], env, sym)?.value_owned())?;
    let n = require_i64(":wat::std::list::window", eval_inner(&args[1], env, sym)?.value_owned())?;
    if n <= 0 {
        return Ok(Value::Vec(Arc::new(Vec::new())));
    }
    let n = n as usize;
    let out: Vec<Value> = xs
        .windows(n)
        .map(|w| Value::Vec(Arc::new(w.to_vec())))
        .collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::std::list::remove-at xs i)` → `Vec<T>`. New Vec with
/// the element at `i` removed. Out-of-range index returns the Vec
/// unchanged (rather than erroring) — matches the inline select
/// loop's "drop the disconnected receiver if it happens to be at
/// index i" idiom without requiring a pre-check. Negative i also
/// no-ops.
///
/// The wat-level op lives in the `:wat::std::list::` namespace (surface contract, unchanged).
/// This Rust function is named `eval_vec_remove_at` to mirror the ENFORCED value type: input
/// must be `Value::Vec` (enforced by `require_vec`); actual `Value::wat__core__List` values
/// are rejected at runtime.
pub(crate) fn eval_vec_remove_at(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::std::list::remove-at".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::std::list::remove-at", eval_inner(&args[0], env, sym)?.value_owned())?;
    let i = require_i64(":wat::std::list::remove-at", eval_inner(&args[1], env, sym)?.value_owned())?;
    if i < 0 || (i as usize) >= xs.len() {
        return Ok(Value::Vec(xs));
    }
    let target = i as usize;
    let mut out = Vec::with_capacity(xs.len() - 1);
    for (idx, v) in xs.iter().enumerate() {
        if idx != target {
            out.push(v.clone());
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

pub(crate) fn eval_vec_last(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::last".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::core::last", eval_inner(&args[0], env, sym)?.value_owned())?;
    Ok(Value::Option(Arc::new(xs.last().cloned())))
}

/// Arc 047 — `(:wat::core::find-last-index xs pred)` returns
/// `Option<i64>`. Iterates `xs`, applies `pred` to each element,
/// returns `Some(i)` for the rightmost `i` where `pred` returned
/// `true`. Returns `None` if no element matched (or `xs` is empty).
/// Mirrors Rust's `iter().rposition(pred)`.
pub(crate) fn eval_vec_find_last_index(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::find-last-index";
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(OP, eval_inner(&args[0], env, sym)?.value_owned())?;
    let f = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let mut last_idx: Option<i64> = None;
    for (i, x) in xs.iter().enumerate() {
        let result = apply_function(
            func.clone(),
            vec![x.clone()],
            sym,
            crate::rust_caller_span!(),
        )?;
        match result {
            Value::bool(true) => last_idx = Some(i as i64),
            Value::bool(false) => {}
            other => {
                return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "bool (predicate result)",
                    got: Box::new(ValueSnapshot::of(&other)),
                } }.into());
            }
        }
    }
    Ok(Value::Option(Arc::new(last_idx.map(Value::i64))))
}

/// `(:wat::std::list::map-with-index xs f)` → `Vec<U>`. Per
/// FOUNDATION-CHANGELOG 2026-04-18 stdlib list surface. `f` takes
/// `(item, index)` and returns U. Used by Sequential's indexed fold.
///
/// The wat-level op lives in the `:wat::std::list::` namespace (surface contract, unchanged).
/// This Rust function is named `eval_vec_map_with_index` to mirror the ENFORCED value type:
/// input must be `Value::Vec` (enforced by `require_vec`); actual `Value::wat__core__List`
/// values are rejected at runtime.
pub(crate) fn eval_vec_map_with_index(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::std::list::map-with-index".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // NB: arg order here is (xs f) — the collection leads. This diverges from the fn-first
    // HOF family (arc 247: map/filter/foldl/foldr all take (f xs)). Do NOT copy the extraction
    // order from sibling HOFs — args[0] is the Vec, args[1] is the function.
    let xs = require_vec(":wat::std::list::map-with-index", eval_inner(&args[0], env, sym)?.value_owned())?;
    let f = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::std::list::map-with-index".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for (i, x) in xs.iter().enumerate() {
        out.push(apply_function(
            func.clone(),
            vec![x.clone(), Value::i64(i as i64)],
            sym,
            crate::rust_caller_span!(),
        )?);
    }
    Ok(Value::Vec(Arc::new(out)))
}
