//! Vector/List-specific utility ops for the collection dispatch home.
//!
//! Contains the ~16 seq-HOF and helper functions (map, filter, foldl, foldr,
//! sort-by, reverse, range, take, drop, last, rest, find-last-index, zip,
//! window, remove-at, map-with-index). These are NOT container-polymorphic
//! dispatch — they are Vector/List-specific utilities. Their dispatch arms
//! in `dispatch_keyword_head_value` redirect here.
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::reverse".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::core::reverse", eval_inner(&args[0], env, sym)?.value_owned())?;
    let mut out = (*xs).clone();
    out.reverse();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::range start end)` → `Vec<i64>`. Two-arg only; the
/// spec-frozen shape maps to Rust's `start..end` exactly. Callers
/// write `(range 0 n)` explicitly for 0..n.
pub(crate) fn eval_vec_range(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::take".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::core::take", eval_inner(&args[0], env, sym)?.value_owned())?;
    let n = require_i64(":wat::core::take", eval_inner(&args[1], env, sym)?.value_owned())?;
    let cap = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
    let out: Vec<Value> = xs.iter().take(cap).cloned().collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::drop xs n)` → `Vec<T>`. Skip first `n` elements. If
/// `n >= xs.len()`, returns an empty Vec. Negative `n` clamps to 0
/// (returns the full Vec).
pub(crate) fn eval_vec_drop(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::drop".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::core::drop", eval_inner(&args[0], env, sym)?.value_owned())?;
    let n = require_i64(":wat::core::drop", eval_inner(&args[1], env, sym)?.value_owned())?;
    let skip = if n <= 0 { 0 } else { (n as usize).min(xs.len()) };
    let out: Vec<Value> = xs.iter().skip(skip).cloned().collect();
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::sort-by xs less?)` → `Vec<T>`.
///
/// Returns a new Vec sorted by the user-supplied less-than predicate.
/// `less?` is a callable `:fn(T, T) -> :bool`; it returns true iff
/// the first arg is "less than" the second under the desired order.
/// The user picks ascending vs descending by which way they compare:
///
///   asc:  `(fn (a b) -> :bool (:wat::core::< a b))`
///   desc: `(fn (a b) -> :bool (:wat::core::> a b))`
///   key:  `(fn (a b) -> :bool (:wat::core::< (:Foo/age a) (:Foo/age b)))`
///
/// Stable. Wraps Rust's `Vec::sort_by`. Common Lisp / Clojure
/// tradition — predicate-driven ordering with the user owning the
/// asc/desc choice. The two-sided test (calling `less?` for both
/// `(a,b)` and `(b,a)` to distinguish Equal from Less/Greater) keeps
/// stable-sort semantics honest; the doubled call count is amortized
/// against O(n log n) — for the lab's bounded windows it's
/// negligible.
pub(crate) fn eval_vec_sort_by(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::sort-by";
    if args.len() != 2 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (sort-by keyfn xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let xs = require_vec(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
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
                other => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "bool",
                    got: Box::new(ValueSnapshot::of(&other)),
                    // arc 138: no — inside sort_by closure, no AST args in scope
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        // arc 138: no span — leaf helper.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::map".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (map f xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let xs = require_vec(":wat::core::map", eval_inner(&args[1], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            // arc 138: no span — leaf helper.
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::map".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
            } }.into());
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for x in xs.iter() {
        out.push(apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::foldl f init xs)` → acc. `f : (acc, item) → acc`.
/// Left-associative: `f(f(f(init, x0), x1), x2)`. Sequential's driver.
/// Arc 247: fn-first — (foldl f init xs).
/// `:wat::core::foldr` ships alongside — see [`eval_vec_foldr`].
pub(crate) fn eval_vec_foldl(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        // arc 138: no span — leaf helper.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::foldl".into(),
            expected: 3,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (foldl f init xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
    let xs = require_vec(":wat::core::foldl", eval_inner(&args[2], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            // arc 138: no span — leaf helper.
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::foldl".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
            } }.into());
        }
    };
    for x in xs.iter() {
        acc = apply_function(func.clone(), vec![acc, x.clone()], sym, crate::rust_caller_span!())?;
    }
    Ok(acc)
}

/// `(:wat::core::foldr f init xs)` → acc. Right-associative fold.
/// `f(x0, f(x1, f(..., f(xn, init))))`. Iterates the Vec in reverse
/// so the call stack is bounded by iteration, not recursion.
/// Arc 247: fn-first — (foldr f init xs).
pub(crate) fn eval_vec_foldr(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        // arc 138: no span — leaf helper.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::foldr".into(),
            expected: 3,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (foldr f init xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
    let xs = require_vec(":wat::core::foldr", eval_inner(&args[2], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            // arc 138: no span — leaf helper.
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::foldr".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
            } }.into());
        }
    };
    for x in xs.iter().rev() {
        acc = apply_function(func.clone(), vec![x.clone(), acc], sym, crate::rust_caller_span!())?;
    }
    Ok(acc)
}

/// `(:wat::core::filter pred xs)` → `Vec<T>`. Keeps elements for
/// which `pred` returns `:bool true`. `pred` signature: `T -> :bool`.
/// Arc 247: fn-first — (filter pred xs).
pub(crate) fn eval_vec_filter(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        // arc 138: no span — leaf helper.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::filter".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    // Arc 247: fn-first — (filter pred xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let xs = require_vec(":wat::core::filter", eval_inner(&args[1], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            // arc 138: no span — leaf helper.
            return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::filter".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
            } }.into());
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for x in xs.iter() {
        match apply_function(func.clone(), vec![x.clone()], sym, crate::rust_caller_span!())? {
            Value::bool(true) => out.push(x.clone()),
            Value::bool(false) => {}
            other => {
                // arc 138: no span — leaf helper.
                return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::filter".into(),
                    expected: "bool",
                    got: Box::new(ValueSnapshot::of(&other))
                } }.into());
            }
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::std::list::zip xs ys)` → `Vec<(T,U)>`. Short-circuits at
/// the shorter input's length (matches Rust's `xs.iter().zip(ys)`).
pub(crate) fn eval_list_zip(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
pub(crate) fn eval_list_window(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
pub(crate) fn eval_list_remove_at(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::find-last-index";
    if args.len() != 2 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
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
                got: Box::new(ValueSnapshot::of(&other))
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
                return Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "bool (predicate result)",
                    got: Box::new(ValueSnapshot::of(&other)),
                    // arc 138: no — predicate result from apply_function; no AST arg in scope
                } }.into());
            }
        }
    }
    Ok(Value::Option(Arc::new(last_idx.map(Value::i64))))
}

/// `(:wat::core::rest xs)` — everything after the first element of a
/// Vec. Mirrors `slice[1..]`. Runtime error if `xs` is empty (there
/// is no `rest` of an empty sequence). Tuples do NOT support rest —
/// tuple arity is fixed at the type level.
pub(crate) fn eval_vec_rest(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::core::rest".into(),
            expected: 1,
            got: args.len()
        } }.into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    match v {
        Value::Vec(xs) => {
            if xs.is_empty() {
                return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::rest".into(),
                    reason: "cannot take rest of empty Vec".into()
                } }.into());
            }
            let out: Vec<Value> = xs.iter().skip(1).cloned().collect();
            Ok(Value::Vec(Arc::new(out)))
        }
        // Arc 220 Stone 220.4 — List: rest returns a new List (tail after first element).
        // Maintains type identity: List/rest → List (not Vec).
        Value::wat__core__List(xs) => {
            if xs.is_empty() {
                return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::rest".into(),
                    reason: "cannot take rest of empty List".into()
                } }.into());
            }
            let out: std::collections::LinkedList<Value> = xs.iter().skip(1).cloned().collect();
            Ok(Value::wat__core__List(Arc::new(out)))
        }
        other => Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "Vec or List",
            got: Box::new(ValueSnapshot::of(&other))
        } }.into()),
    }
}

/// `(:wat::std::list::map-with-index xs f)` → `Vec<U>`. Per
/// FOUNDATION-CHANGELOG 2026-04-18 stdlib list surface. `f` takes
/// `(item, index)` and returns U. Used by Sequential's indexed fold.
pub(crate) fn eval_list_map_with_index(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: ":wat::std::list::map-with-index".into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let xs = require_vec(":wat::std::list::map-with-index", eval_inner(&args[0], env, sym)?.value_owned())?;
    let f = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: ":wat::std::list::map-with-index".into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(&other))
            } }.into());
        }
    };
    let mut out = Vec::with_capacity(xs.len());
    for (i, x) in xs.iter().enumerate() {
        out.push(apply_function(
            func.clone(),
            vec![x.clone(), Value::i64(i as i64)],
            sym,
            Span::unknown(),
        )?);
    }
    Ok(Value::Vec(Arc::new(out)))
}
