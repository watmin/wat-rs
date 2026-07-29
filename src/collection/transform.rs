//! Stream-HOF and helper functions for the collection dispatch home.
//!
//! Contains the ~15 seq-HOF and helper functions (map, filter, foldl, foldr,
//! sort' (primitive comparator-sort), reverse, range, take, drop, last,
//! find-last-index, zip, window, remove-at, map-with-index).
//!
//! Arc-278 strike 3: the HOF family (map/filter/foldl/foldr/reverse/take/drop)
//! is now container-polymorphic over `mappable()` containers (currently Vector
//! and PersistentVector). Classification delegates to `StreamContainer::of_value` +
//! `mappable()` — no hand-rolled per-container match in the classifier gate.
//! Per-container element-iteration/rebuild arms remain behind the gate.
//!
//! The four ops in the `:wat::std::list::` namespace (zip, window, remove-at,
//! map-with-index) are named `eval_vec_*` and still enforce `Value::Vec` via
//! `require_vec` — they are not part of the HOF family migration.
//! `rest` lives in `eval.rs` (container-polymorphic; Vec/List/WatAST-form arms).
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
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + ordered()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.ordered() => match container {
            StreamContainer::Vector => {
                let Value::Vec(xs) = v else { unreachable!("of_value⇒Vector") };
                let mut out = (*xs).clone();
                out.reverse();
                Ok(Value::Vec(Arc::new(out)))
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = v else { unreachable!("of_value⇒PersistentVector") };
                let mut out: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
                for elem in pv.iter().collect::<Vec<_>>().into_iter().rev() {
                    out = out.push_back(elem.clone());
                }
                Ok(Value::wat__core__PersistentVector(out))
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = v else { unreachable!("of_value⇒List") };
                let out: std::collections::LinkedList<Value> = xs.iter().rev().cloned().collect();
                Ok(Value::wat__core__List(Arc::new(out)))
            }
            // ordered() gate excludes these — named arms, genuinely dead, compiler-forced:
            StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet | StreamContainer::Stream =>
                unreachable!("ordered() gate excludes Tuple/WatAstList/HashSet/Stream"),
        },
        _ => Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::reverse".into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
            got: Box::new(ValueSnapshot::of(&v))
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

/// `(:wat::core::take xs n)` → `Stream<T>`. Lazily yields at most the first `n` elements of
/// `xs` (any seqable: `Vector<T>` | `List<T>` | `PersistentVector<T>` | `Stream<T>`) — pulling
/// element `n+1` never happens, so `take` composed with an upstream lazy stage (e.g. `map`)
/// never forces past what it needs. Negative `n` clamps to 0 (empty).
///
/// Arc 118.2a — the FLIP: return is always `Stream<T>` now (was container-preserving eager).
/// **Stays a Rust intrinsic** — see [`eval_vec_map`]'s doc for the bootstrap-circularity
/// reason (`:wat::core::defn`'s own macro body calls `take` at macro-expansion time; a
/// wat-defined `take` would make `defn` itself unbootstrappable). See
/// [`crate::stream::NativeLazyCell`] for the full writeup.
pub(crate) fn eval_vec_take(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::take";
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError {
        span: args[0].span().clone(),
        kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        },
    }))?;
    Ok(Value::wat__stream__Stream(lazy_take_stream(source, n)))
}

/// Build a deferred `take` cell: forcing it realizes `source` one step; if `n` has already
/// been exhausted or `source` is empty, yields `Empty`; otherwise yields the head and defers
/// `n - 1` more from the tail.
fn lazy_take_stream(source: Arc<crate::stream::Stream>, n: i64) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    if n <= 0 {
        return Arc::new(Stream::Empty);
    }
    Arc::new(Stream::NativeThunk(NativeLazyCell {
        thunk: Arc::new(move |sym, span| {
            let realized = crate::stream::realize(&source, sym, span)?;
            match realized.as_ref() {
                Stream::Empty => Ok(Arc::new(Stream::Empty)),
                Stream::Cons { head, tail } => {
                    let rest = lazy_take_stream(Arc::clone(tail), n - 1);
                    Ok(Arc::new(Stream::Cons { head: head.clone(), tail: rest }))
                }
                Stream::Thunk(_) | Stream::NativeThunk(_) => {
                    unreachable!("crate::stream::realize always returns Empty|Cons")
                }
            }
        }),
    }))
}

/// `(:wat::core::drop xs n)` → `Stream<T>`. Lazily skips the first `n` elements of `xs` (any
/// seqable), returning the remainder — still lazy beyond the drop point (a further `Stream`
/// tail stays deferred). Negative `n` clamps to 0 (returns everything).
///
/// Arc 118.2a — the FLIP: return is always `Stream<T>` now (was container-preserving eager).
/// **Stays a Rust intrinsic** — see [`eval_vec_map`]'s doc for the bootstrap-circularity
/// reason (`:wat::core::defn`'s own macro body calls `drop` at macro-expansion time). See
/// [`crate::stream::NativeLazyCell`] for the full writeup.
pub(crate) fn eval_vec_drop(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::drop";
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } }.into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError {
        span: args[0].span().clone(),
        kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        },
    }))?;
    Ok(Value::wat__stream__Stream(lazy_drop_stream(source, n)))
}

/// Build a deferred `drop` cell: forcing it walks (and, when the upstream is itself lazy,
/// forces) up to `n` cells of `source`, then returns whatever WHNF cell it lands on (`Empty`
/// or a `Cons` whose OWN tail may still be deferred — laziness continues past the drop point).
fn lazy_drop_stream(source: Arc<crate::stream::Stream>, n: i64) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    Arc::new(Stream::NativeThunk(NativeLazyCell {
        thunk: Arc::new(move |sym, span| {
            let mut cur = Arc::clone(&source);
            let mut remaining = n;
            loop {
                if remaining <= 0 {
                    return crate::stream::realize(&cur, sym, span);
                }
                let realized = crate::stream::realize(&cur, sym, span)?;
                match realized.as_ref() {
                    Stream::Empty => return Ok(Arc::new(Stream::Empty)),
                    Stream::Cons { tail, .. } => {
                        cur = Arc::clone(tail);
                        remaining -= 1;
                    }
                    Stream::Thunk(_) | Stream::NativeThunk(_) => {
                        unreachable!("crate::stream::realize always returns Empty|Cons")
                    }
                }
            }
        }),
    }))
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
    const OP: &str = ":wat::core::sort'";  // rune:lint(retired-name) — live prime (arc 251 comparator-sort primitive); wat-level sort/sort-by wrap it
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
                call_span.clone(),
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

/// `(:wat::core::map f xs)` → `Stream<U>`. Lazily calls `f` on each element as the result is
/// pulled; `xs` may be any seqable (`Vector<T>` | `List<T>` | `PersistentVector<T>` |
/// `Stream<T>`). `f` must be a callable Value (fn or define-registered).
///
/// Arc 118.2a — the FLIP: `map` used to be an eager Rust intrinsic (Vec→Vec, container-
/// preserving). It is now LAZY — the return is always a `Stream<U>`, and `f` is applied one
/// element at a time, only as far as the caller pulls (`first`/`rest`/`realize`).
///
/// **Stays a Rust intrinsic, not wat-over-primitives** (Decision B's original preference):
/// `:wat::core::defrecord` / `:wat::holon::defrecord` / `:wat::service::defservice` /
/// `:wat::rete::defrule` all call `:wat::core::map` INSIDE their own macro bodies (macro-
/// expansion time, step 4 of the freeze pipeline — before a wat-defined `defclause`'s real
/// clauses exist, step 6). A wat-defined `map` would be an inert nil-returning checker stub at
/// the exact moment those ~30+ stdlib `defrecord`/`defservice` invocations need real behavior.
/// See [`crate::stream::NativeLazyCell`] for the full bootstrap-circularity writeup. `filter`
/// has no such caller and ships as a genuine wat `defclause` instead (`wat/seq.wat`).
///
/// Arc 247: fn-first — (map f xs).
pub(crate) fn eval_vec_map(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::map";
    if args.len() != 2 {
        return Err(RuntimeError { span: call_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
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
                op: OP.into(),
                expected: "wat::core::fn",
                got: Box::new(ValueSnapshot::of(other))
            } }.into());
        }
    };
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError {
        span: args[1].span().clone(),
        kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        },
    }))?;
    Ok(Value::wat__stream__Stream(lazy_map_stream(func, source)))
}

/// Build one deferred `map` cell: forcing it realizes `source` one step, applies `func` to
/// the head (strictly, at force time — this IS the laziness: `func` runs on element N only
/// when the caller pulls that far), and defers the rest via a recursive `NativeThunk`.
fn lazy_map_stream(
    func: Arc<crate::value::Function>,
    source: Arc<crate::stream::Stream>,
) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    Arc::new(Stream::NativeThunk(NativeLazyCell {
        thunk: Arc::new(move |sym, span| {
            let realized = crate::stream::realize(&source, sym, span)?;
            match realized.as_ref() {
                Stream::Empty => Ok(Arc::new(Stream::Empty)),
                Stream::Cons { head, tail } => {
                    let mapped_head = apply_function(func.clone(), vec![head.clone()], sym, span.clone())?;
                    let mapped_tail = lazy_map_stream(func.clone(), Arc::clone(tail));
                    Ok(Arc::new(Stream::Cons { head: mapped_head, tail: mapped_tail }))
                }
                Stream::Thunk(_) | Stream::NativeThunk(_) => {
                    unreachable!("crate::stream::realize always returns Empty|Cons")
                }
            }
        }),
    }))
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
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + mappable()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&coll) {
        Some(container) if container.mappable() => match container {
            StreamContainer::Vector => {
                let Value::Vec(xs) = coll else { unreachable!("of_value⇒Vector") };
                for x in xs.iter() {
                    acc = apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = coll else { unreachable!("of_value⇒PersistentVector") };
                for x in pv.iter() {
                    acc = apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = coll else { unreachable!("of_value⇒List") };
                for x in xs.iter() {
                    acc = apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            // mappable() gate excludes these — named arms, genuinely dead, compiler-forced:
            StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet | StreamContainer::Stream =>
                unreachable!("mappable() gate excludes Tuple/WatAstList/HashSet/Stream"),
        },
        _ => Err(RuntimeError { span: args[2].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::foldl".into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
            got: Box::new(ValueSnapshot::of(&coll))
        } }.into()),
    }
}

/// `(:wat::core::foldr f init xs)` → acc. Right-associative fold.
/// `f(x0, f(x1, f(..., f(xn, init))))`. Iterates the container in reverse
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
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + mappable()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&coll) {
        Some(container) if container.mappable() => match container {
            StreamContainer::Vector => {
                let Value::Vec(xs) = coll else { unreachable!("of_value⇒Vector") };
                for x in xs.iter().rev() {
                    acc = apply_function(func.clone(), vec![x.clone(), acc], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = coll else { unreachable!("of_value⇒PersistentVector") };
                let elems: Vec<&Value> = pv.iter().collect();
                for x in elems.into_iter().rev() {
                    acc = apply_function(func.clone(), vec![x.clone(), acc], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = coll else { unreachable!("of_value⇒List") };
                let elems: Vec<&Value> = xs.iter().collect();
                for x in elems.into_iter().rev() {
                    acc = apply_function(func.clone(), vec![x.clone(), acc], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            // mappable() gate excludes these — named arms, genuinely dead, compiler-forced:
            StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet | StreamContainer::Stream =>
                unreachable!("mappable() gate excludes Tuple/WatAstList/HashSet/Stream"),
        },
        _ => Err(RuntimeError { span: args[2].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::foldr".into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
            got: Box::new(ValueSnapshot::of(&coll))
        } }.into()),
    }
}

// Arc 118.2a — `eval_vec_filter` RETIRED. `:wat::core::filter` has no macro-expansion-time
// caller anywhere in the stdlib (unlike map/take/drop), so it ships as a genuine wat
// `defclause` instead (Vector<T>/List<T>/PersistentVector<T>/Stream<T> clauses, `wat/seq.wat`)
// — honoring Decision B's self-hosting preference wherever the bootstrap allows it. Its
// check.rs `infer_filter` special-case arm is retired too (see `src/collection/infer.rs`);
// `:wat::core::filter` now falls through to ordinary defclause dispatch.

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
            call_span.clone(),
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
            call_span.clone(),
        )?);
    }
    Ok(Value::Vec(Arc::new(out)))
}
