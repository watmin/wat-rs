//! Stream-HOF and helper functions for the collection dispatch home.
//!
//! Contains the ~15 seq-HOF and helper functions (map, filter, foldl,
//! sort' (primitive comparator-sort), reverse, range, take, drop, last,
//! find-last-index, zip, window, remove-at, map-with-index).
//!
//! Arc-278 strike 3: the HOF family (map/filter/foldl/reverse/take/drop)
//! is now container-polymorphic over `mappable()` containers (currently Vector
//! and PersistentVector). Classification delegates to `StreamContainer::of_value` +
//! `mappable()` — no hand-rolled per-container match in the classifier gate.
//! Per-container element-iteration/rebuild arms remain behind the gate.
//!
//! The four ops in the `:wat::std::list::` namespace (zip, window, remove-at,
//! map-with-index) are named `eval_vec_*` and still enforce `Value::Vec` via
//! `require_vec` — they are not part of the HOF family migration.
//! `rest` lives in `eval.rs` (container-polymorphic; Vec/List/WatAST-form/PersistentVector).
//! Their dispatch arms in `dispatch_keyword_head_value` redirect here.
//!
//! See `src/collection/mod.rs` and `docs/DISPATCH.md` for the full doctrine.

use crate::ast::WatAST;
use crate::runtime::{
    apply_function, eval_inner, require_i64, require_vec, Environment, EvalBreak, RuntimeError,
    RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::reverse".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + ordered()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.ordered() => match container {
            StreamContainer::Vector => {
                let Value::Vec(xs) = v else {
                    unreachable!("of_value⇒Vector")
                };
                let mut out = (*xs).clone();
                out.reverse();
                Ok(Value::Vec(Arc::new(out)))
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = v else {
                    unreachable!("of_value⇒PersistentVector")
                };
                let mut out: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
                for elem in pv.iter().collect::<Vec<_>>().into_iter().rev() {
                    out.push_back_mut(elem.clone());
                }
                Ok(Value::wat__core__PersistentVector(out))
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = v else {
                    unreachable!("of_value⇒List")
                };
                let out: std::collections::LinkedList<Value> = xs.iter().rev().cloned().collect();
                Ok(Value::wat__core__List(Arc::new(out)))
            }
            // ordered() gate excludes these — named arms, genuinely dead, compiler-forced:
            StreamContainer::Tuple
            | StreamContainer::WatAstList
            | StreamContainer::HashSet
            | StreamContainer::Stream => {
                unreachable!("ordered() gate excludes Tuple/WatAstList/HashSet/Stream")
            }
        },
        _ => Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::reverse".into(),
                expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::range".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let start = require_i64(
        ":wat::core::range",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let end = require_i64(
        ":wat::core::range",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        })))?;
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
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |sym, span| {
            let realized = crate::stream::realize(&source, sym, span)?;
            match realized.as_ref() {
                Stream::Empty => Ok(Arc::new(Stream::Empty)),
                Stream::Cons { head, tail } => {
                    let rest = lazy_take_stream(Arc::clone(tail), n - 1);
                    Ok(Arc::new(Stream::Cons {
                        head: head.clone(),
                        tail: rest,
                    }))
                }
                Stream::Thunk(_) | Stream::NativeThunk(_) => {
                    unreachable!("crate::stream::realize always returns Empty|Cons")
                }
            }
        },
    ))))
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    let n = require_i64(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        })))?;
    Ok(Value::wat__stream__Stream(lazy_drop_stream(source, n)))
}

/// Build a deferred `drop` cell: forcing it walks (and, when the upstream is itself lazy,
/// forces) up to `n` cells of `source`, then returns whatever WHNF cell it lands on (`Empty`
/// or a `Cons` whose OWN tail may still be deferred — laziness continues past the drop point).
fn lazy_drop_stream(source: Arc<crate::stream::Stream>, n: i64) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |sym, span| {
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
        },
    ))))
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
    const OP: &str = ":wat::core::sort'"; // rune:lint(retired-name) — live prime (arc 251 comparator-sort primitive); wat-level sort/sort-by wrap it
    if args.len() != 2 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // Arc 247: fn-first — (sort' cmp xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let xs = require_vec(OP, eval_inner(&args[1], env, sym)?.value_owned())?;
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
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
            )
            .map_err(EvalBreak::from)?;
            match v {
                Value::bool(b) => Ok(b),
                other => Err(RuntimeError::new(
                    call_span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into()),
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // Arc 247: fn-first — (map f xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let coll = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let source = crate::stream::value_as_stream(&coll).ok_or_else(|| EvalBreak::from(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        })))?;
    Ok(Value::wat__stream__Stream(lazy_map_stream(func, source)))
}

/// `(:wat::core::mapv f coll)` → `Vector<U>`. Eager. Walks Vector / PersistentVector /
/// List by position (no lazy Stream cells). Stream input still maps lazily then drains.
/// Fanout protocol query spent 288 ms in `(into [] (map f pv))` — 40k NativeThunk
/// cons cells plus apply (`DESIGN-STONE-mapv-eager`).
pub(crate) fn eval_mapv(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::mapv";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let coll = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let apply_one = |x: &Value| -> Result<Value, EvalBreak> {
        apply_function(func.clone(), vec![x.clone()], sym, call_span.clone()).map_err(EvalBreak::from)
    };
    match &coll {
        Value::Vec(v) => {
            let mut out = Vec::with_capacity(v.len());
            for x in v.iter() {
                out.push(apply_one(x)?);
            }
            Ok(Value::Vec(Arc::new(out)))
        }
        Value::wat__core__PersistentVector(v) => {
            let mut out = Vec::with_capacity(v.len());
            for x in v.iter() {
                out.push(apply_one(x)?);
            }
            Ok(Value::Vec(Arc::new(out)))
        }
        Value::wat__core__List(v) => {
            let mut out = Vec::with_capacity(v.len());
            for x in v.iter() {
                out.push(apply_one(x)?);
            }
            Ok(Value::Vec(Arc::new(out)))
        }
        Value::wat__stream__Stream(_) => {
            let source = crate::stream::value_as_stream(&coll).expect("Stream value");
            let mapped = lazy_map_stream(func, source);
            let mut out = Vec::new();
            let mut cur = mapped;
            loop {
                let realized = crate::stream::realize(&cur, sym, call_span)?;
                match realized.as_ref() {
                    crate::stream::Stream::Empty => return Ok(Value::Vec(Arc::new(out))),
                    crate::stream::Stream::Cons { head, tail } => {
                        out.push(head.clone());
                        cur = Arc::clone(tail);
                    }
                    crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
                        unreachable!("realize returns Empty|Cons")
                    }
                }
            }
        }
        other => Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::Vector, PersistentVector, List, or Stream",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Build one deferred `map` cell: forcing it realizes `source` one step, applies `func` to
/// the head (strictly, at force time — this IS the laziness: `func` runs on element N only
/// when the caller pulls that far), and defers the rest via a recursive `NativeThunk`.
fn lazy_map_stream(
    func: Arc<crate::value::Function>,
    source: Arc<crate::stream::Stream>,
) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |sym, span| {
            let realized = crate::stream::realize(&source, sym, span)?;
            match realized.as_ref() {
                Stream::Empty => Ok(Arc::new(Stream::Empty)),
                Stream::Cons { head, tail } => {
                    let mapped_head =
                        apply_function(func.clone(), vec![head.clone()], sym, span.clone())?;
                    let mapped_tail = lazy_map_stream(func.clone(), Arc::clone(tail));
                    Ok(Arc::new(Stream::Cons {
                        head: mapped_head,
                        tail: mapped_tail,
                    }))
                }
                Stream::Thunk(_) | Stream::NativeThunk(_) => {
                    unreachable!("crate::stream::realize always returns Empty|Cons")
                }
            }
        },
    ))))
}

/// `(:wat::core::foldl f init xs)` → acc. `f : (acc, item) → acc`.
/// Left-associative: `f(f(f(init, x0), x1), x2)`. Sequential's driver.
/// Arc 247: fn-first — (foldl f init xs).
pub(crate) fn eval_vec_foldl(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::foldl".into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    // Arc 247: fn-first — (foldl f init xs)
    let f = eval_inner(&args[0], env, sym)?.value_owned();
    let mut acc = eval_inner(&args[1], env, sym)?.value_owned();
    let coll = eval_inner(&args[2], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::foldl".into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + mappable()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    // Stone 118.B6 — `mappable()` OR `Stream`. `mappable()`'s `Stream => false` arm still carries
    // arc 118's own note that streaming HOFs were "a later strike. ○ gap"; this closes foldl's half
    // of it WITHOUT widening that table, because `mappable()` also gates map/filter and
    // moving it would ripple across the whole HOF family in one commit (118.B6 STOP-2).
    match StreamContainer::of_value(&coll) {
        Some(container) if container.mappable() || matches!(container, StreamContainer::Stream) => match container {
            StreamContainer::Vector => {
                let Value::Vec(xs) = coll else {
                    unreachable!("of_value⇒Vector")
                };
                for x in xs.iter() {
                    acc =
                        apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = coll else {
                    unreachable!("of_value⇒PersistentVector")
                };
                for x in pv.iter() {
                    acc =
                        apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = coll else {
                    unreachable!("of_value⇒List")
                };
                for x in xs.iter() {
                    acc =
                        apply_function(func.clone(), vec![acc, x.clone()], sym, call_span.clone())?;
                }
                Ok(acc)
            }
            // Stone 118.B6 — the lazy arm. Eager containers keep their DIRECT iterators above;
            // nothing beats walking memory you already hold, and the native side is under no
            // uniformity requirement because the SPECIFICATION lives in wat
            // (`:wat::core::foldl-spec`, wat/seq.wat), not in a second Rust body.
            //
            // Iterative, not recursive: the accumulator threads through a loop, so fold depth is
            // bounded by iteration and a long stream cannot exhaust the Rust stack (tasks #58/#86 —
            // that death is a silent SIGSEGV).
            StreamContainer::Stream => {
                let Some(mut cur) = crate::stream::value_as_stream(&coll) else {
                    unreachable!("of_value⇒Stream")
                };
                loop {
                    let realized = crate::stream::realize(&cur, sym, call_span)?;
                    match realized.as_ref() {
                        crate::stream::Stream::Empty => return Ok(acc),
                        crate::stream::Stream::Cons { head, tail } => {
                            acc = apply_function(
                                func.clone(),
                                vec![acc, head.clone()],
                                sym,
                                call_span.clone(),
                            )?;
                            cur = Arc::clone(tail);
                        }
                        crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
                            unreachable!("crate::stream::realize always returns Empty|Cons")
                        }
                    }
                }
            }
            // gate excludes these — named arms, genuinely dead, compiler-forced:
            StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet => {
                unreachable!("the gate excludes Tuple/WatAstList/HashSet")
            }
        },
        _ => Err(RuntimeError::new(
            args[2].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::foldl".into(),
                expected: "wat::core::Vector, wat::core::PersistentVector, wat::core::List, or wat::stream::Stream",
                got: Box::new(ValueSnapshot::of(&coll)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::stream->vec acc s)` — Stone 118.B5: the native kernel underneath `into`'s
/// `Vector<T> <- Stream<T>` clause arm (`wat/seq.wat:166`; the clause itself is UNCHANGED — it
/// already named `stream->vec`, and that name simply stopped being interpreted). Promoted from
/// a wat `defn` to a Rust intrinsic, exactly the `nth` (B4-0) / `foldl` (B6) shape:
/// `:wat::core::stream->vec-spec` (`wat/seq.wat`) is the retained wat ORACLE, kept honest by a
/// differential (`wat-tests/core/core-stream-materializers-differential.wat`).
///
/// Realizes the Stream one cell at a time via `crate::stream::realize` — the SAME iterative
/// loop `eval_vec_foldl`'s Stream arm uses (`Stream::Cons { head, tail }` → push `head`,
/// `cur = Arc::clone(tail)`, drop the old `cur`) — so fold depth is bounded by iteration, not
/// recursion, AND no earlier cell is retained past its own step. ⚠ THE TRAP this stone names:
/// a native that first collects into an INTERMEDIATE container (or that keeps `cur`'s previous
/// value alive) reintroduces the O(n) retention B3 deleted
/// (`wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat`) — one pass, one
/// accumulator, nothing else held.
///
/// Seeded by `acc` (so `into` can append onto an existing Vector, not just build from empty) —
/// `Arc::try_unwrap` reclaims `acc`'s backing `Vec` in place when this call holds the only
/// reference (the common case: `into`'s `[]`/fresh-Vector callers), falling back to one clone
/// only when the accumulator is shared.
pub(crate) fn eval_stream_to_vec(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::stream->vec";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let acc = eval_inner(&args[0], env, sym)?.value_owned();
    let Value::Vec(acc) = acc else {
        return Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::Vector",
                got: Box::new(ValueSnapshot::of(&acc)),
            },
        )
        .into());
    };
    let s = eval_inner(&args[1], env, sym)?.value_owned();
    let Some(mut cur) = crate::stream::value_as_stream(&s) else {
        return Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::stream::Stream",
                got: Box::new(ValueSnapshot::of(&s)),
            },
        )
        .into());
    };
    let mut out: Vec<Value> = match Arc::try_unwrap(acc) {
        Ok(v) => v,
        Err(shared) => (*shared).clone(),
    };
    loop {
        let realized = crate::stream::realize(&cur, sym, call_span)?;
        match realized.as_ref() {
            crate::stream::Stream::Empty => return Ok(Value::Vec(Arc::new(out))),
            crate::stream::Stream::Cons { head, tail } => {
                out.push(head.clone());
                cur = Arc::clone(tail);
            }
            crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
                unreachable!("crate::stream::realize always returns Empty|Cons")
            }
        }
    }
}

/// `(:wat::core::stream->pvec acc s)` — Stone 118.B5: the native kernel underneath `into`'s
/// `PersistentVector<T> <- Stream<T>` clause arm (`wat/seq.wat:166`; the clause itself is
/// UNCHANGED). `:wat::core::stream->pvec-spec` (`wat/seq.wat`) is the retained wat ORACLE — see
/// `eval_stream_to_vec`'s doc, immediately above, for the shared shape/trap/differential note;
/// this is its PersistentVector twin. Unique `PVec::push_back_mut` grows the accumulator
/// in place and stays Array (conj is the other verb — persistent `push_back`, which
/// promotes at 8). One pass, one accumulator, exactly like its Vector sibling.
pub(crate) fn eval_stream_to_pvec(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::stream->pvec";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let acc = eval_inner(&args[0], env, sym)?.value_owned();
    let Value::wat__core__PersistentVector(mut pv) = acc else {
        return Err(RuntimeError::new(
            args[0].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::core::PersistentVector",
                got: Box::new(ValueSnapshot::of(&acc)),
            },
        )
        .into());
    };
    let s = eval_inner(&args[1], env, sym)?.value_owned();
    let Some(mut cur) = crate::stream::value_as_stream(&s) else {
        return Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::stream::Stream",
                got: Box::new(ValueSnapshot::of(&s)),
            },
        )
        .into());
    };
    loop {
        let realized = crate::stream::realize(&cur, sym, call_span)?;
        match realized.as_ref() {
            crate::stream::Stream::Empty => {
                return Ok(Value::wat__core__PersistentVector(pv))
            }
            crate::stream::Stream::Cons { head, tail } => {
                pv.push_back_mut(head.clone());
                cur = Arc::clone(tail);
            }
            crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
                unreachable!("crate::stream::realize always returns Empty|Cons")
            }
        }
    }
}

// Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 2a — `:wat::core::filter` is NATIVE
// again (`eval_filter`, above `eval_seqable_to_stream` in this file), superseding the
// Arc-118.2a wat `defclause` (five per-container arms, `wat/seq.wat`) that walked its source
// by repeated `rest` — O(n^2). The "no macro-expansion-time caller, so self-host it" reasoning
// that justified the wat defclause was true but incomplete: it never weighed the O(n^2) cost
// of the only traversal a wat `defclause` COULD express without a native seqable→stream door.
// Now that door exists (Strike 1), `filter` composes through it instead — one body, any
// seqable, dispatched through the `StreamContainer` registry exactly like `map`. check.rs's
// `infer_filter` special-case arm is live again too (`src/collection/infer.rs`).

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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::std::list::zip".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(
        ":wat::std::list::zip",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let ys = require_vec(
        ":wat::std::list::zip",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::std::list::window".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(
        ":wat::std::list::window",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let n = require_i64(
        ":wat::std::list::window",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::std::list::remove-at".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(
        ":wat::std::list::remove-at",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let i = require_i64(
        ":wat::std::list::remove-at",
        eval_inner(&args[1], env, sym)?.value_owned(),
    )?;
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::last".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(
        ":wat::core::last",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let xs = require_vec(OP, eval_inner(&args[0], env, sym)?.value_owned())?;
    let f = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let mut last_idx: Option<i64> = None;
    for (i, x) in xs.iter().enumerate() {
        let result = apply_function(func.clone(), vec![x.clone()], sym, call_span.clone())?;
        match result {
            Value::bool(true) => last_idx = Some(i as i64),
            Value::bool(false) => {}
            other => {
                return Err(RuntimeError::new(
                    call_span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "bool (predicate result)",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
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
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::std::list::map-with-index".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // NB: arg order here is (xs f) — the collection leads. This diverges from the fn-first
    // HOF family (arc 247: map/filter/foldl all take (f xs)). Do NOT copy the extraction
    // order from sibling HOFs — args[0] is the Vec, args[1] is the function.
    let xs = require_vec(
        ":wat::std::list::map-with-index",
        eval_inner(&args[0], env, sym)?.value_owned(),
    )?;
    let f = eval_inner(&args[1], env, sym)?.value_owned();
    let func = match &f {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::std::list::map-with-index".into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
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

/// `(:wat::core::seqable->stream coll)` → `Stream<T>`. Arc-278 DESIGN-STONE
/// seq-traversal-one-door, Strike 1 — the private eager→lazy normalizer, NATIVE now,
/// replacing the wat `defclause` that used to walk its source by repeated `(rest coll)`.
/// `rest` on an eager container REBUILDS the whole remaining container (`eval_rest`,
/// `collection/eval.rs`), so the old walk was O(n^2); this one steps its source BY
/// POSITION and materialises nothing per element, so it is O(n) total.
///
/// Every verb that already delegates through this converter (`keep`, `keep-indexed`,
/// `take-nth`, `dedupe`, `distinct`, `map-indexed` — `wat/seq.wat`) goes linear with ZERO
/// edits of their own: that is the proof the door is shared.
///
/// Dispatch shape copied from `eval_vec_foldl`'s container match (this file), routed
/// through the `StreamContainer` registry (`collection/seq_container.rs`) — no re-derived
/// classification, and (arc-278 strike 4 convention) exhaustive over the closed enum, no `_`.
///
/// - `Stream` — already lazy; returned unchanged (`Arc` bump only).
/// - `Vector` — already indexable behind an `Arc<Vec<Value>>`. Builds a `NativeThunk`
///   holding `(the Arc handle, index)`; forcing it yields `Cons(elem_at(index),
///   thunk(index + 1))`, `Empty` past the end. The handle is `Arc::clone`d once per step —
///   O(1) — never the elements, and no element is touched until its cell is forced.
/// - `PersistentVector` — `PVec`: Array get is a slice; Tree get is O(log n). `.clone()` is
///   O(1) (Arc / RRB handle). Same index-stepping shape as `Vector`.
/// - `List` — `Arc<LinkedList>`, which has NO indexed access. Snapshotted into an indexable
///   `Vec<Value>` ONCE (a single O(n) pass, not per element), then stepped exactly like the
///   `Vector` arm. Indexing the `LinkedList` itself per step would reintroduce the quadratic
///   on this arm — the exact silent divergence the design stone exists to kill.
pub(crate) fn eval_seqable_to_stream(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::seqable->stream";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let coll = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::wat__stream__Stream(seqable_value_to_stream(
        coll,
        OP,
        args[0].span(),
    )?))
}

/// Shared value-level seqable→stream normalizer — the exact per-container dispatch
/// [`eval_seqable_to_stream`] performs, factored out so `filter` (below) can COMPOSE through
/// it on an already-evaluated `Value` instead of re-deriving the same container walk. `op` and
/// `coll_span` are the CALLING verb's op name / arg span, threaded through so a `TypeMismatch`
/// raised here reads as coming from the caller (e.g. `:wat::core::filter`), not this internal
/// plumbing function.
///
/// This is `eval_seqable_to_stream`'s body verbatim (Strike 1) — same containers, same List
/// snapshot rule, same errors — just parameterized over an already-evaluated `Value` rather
/// than a raw AST arg, so both callers get identical per-container correctness for free.
fn seqable_value_to_stream(
    coll: Value,
    op: &str,
    coll_span: &Span,
) -> Result<Arc<crate::stream::Stream>, EvalBreak> {
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&coll) {
        Some(container) => match container {
            // Already lazy — return unchanged (Arc bump only).
            StreamContainer::Stream => {
                let Value::wat__stream__Stream(s) = coll else { unreachable!("of_value⇒Stream") };
                Ok(s)
            }
            StreamContainer::Vector => {
                let Value::Vec(xs) = coll else { unreachable!("of_value⇒Vector") };
                Ok(indexed_vec_stream(xs, 0))
            }
            StreamContainer::PersistentVector => {
                let Value::wat__core__PersistentVector(pv) = coll else { unreachable!("of_value⇒PersistentVector") };
                Ok(indexed_pv_stream(pv, 0))
            }
            StreamContainer::List => {
                let Value::wat__core__List(xs) = coll else { unreachable!("of_value⇒List") };
                // No indexed access on a LinkedList — snapshot ONCE (a single O(n) pass),
                // then step the snapshot by index exactly like the Vector arm.
                let snapshot: Arc<Vec<Value>> = Arc::new(xs.iter().cloned().collect());
                Ok(indexed_vec_stream(snapshot, 0))
            }
            // Not accepted by this door — the same set the wat defclause it replaces
            // accepted (Vector/List/PersistentVector/Stream only).
            StreamContainer::Tuple | StreamContainer::WatAstList | StreamContainer::HashSet => {
                Err(RuntimeError::new(coll_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "wat::core::Vector, wat::core::List, wat::core::PersistentVector, or wat::stream::Stream",
                    got: Box::new(ValueSnapshot::of(&coll)),
                }).into())
            }
        },
        None => Err(RuntimeError::new(coll_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::core::Vector, wat::core::List, wat::core::PersistentVector, or wat::stream::Stream",
            got: Box::new(ValueSnapshot::of(&coll)),
        }).into()),
    }
}

/// `(:wat::core::filter pred coll)` → `Stream<T>`. Arc-278 DESIGN-STONE
/// seq-traversal-one-door, Strike 2a — NATIVE now, replacing the five wat `defclause` arms
/// (`wat/seq.wat`) that each stepped their eager source by repeated `(rest coll)` — O(n^2).
///
/// **Composes, does not re-derive**: normalises `coll` through [`seqable_value_to_stream`] —
/// the exact function `seqable->stream` itself calls — then lazily walks the resulting
/// `Stream`, applying `pred` one element at a time and skipping rejects, only as far as the
/// caller pulls. This reuses Strike 1's per-container correctness (including the `List`
/// snapshot) instead of duplicating it — one door, one walk.
///
/// pred-first (`(filter pred coll)`), mirroring the retired wat clauses' call order and
/// `map`'s fn-first shape (`eval_vec_map`). A raising `pred` PROPAGATES (via `?` inside the
/// lazy cell) rather than being swallowed — a filter that silently dropped an element on a
/// predicate error would be a hidden failure, not an honest one.
pub(crate) fn eval_filter(
    args: &[WatAST],
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::filter";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            call_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // pred-first: arg[0] is the predicate, arg[1] is the collection.
    let p = eval_inner(&args[0], env, sym)?.value_owned();
    let coll = eval_inner(&args[1], env, sym)?.value_owned();
    let pred = match &p {
        Value::wat__core__fn(func) => func.clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let source = seqable_value_to_stream(coll, OP, args[1].span())?;
    Ok(Value::wat__stream__Stream(lazy_filter_stream(
        OP, pred, source,
    )))
}

/// Build one deferred `filter` cell over `source`: forcing it walks `source` — via
/// `crate::stream::realize`'s iterative loop, not Rust recursion, so a long run of rejected
/// elements stays O(1) per element and never grows the Rust stack — applying `pred` to each
/// head strictly, at force time (the laziness: `pred` runs on element N only when the caller
/// pulls that far). The first element `pred` accepts yields `Cons{that element, <a fresh
/// filter cell over the rest>}`; if `source` is exhausted first, `Empty`. `pred`'s errors
/// propagate via `?` — never swallowed, never turned into a silently-dropped element.
fn lazy_filter_stream(
    op: &'static str,
    pred: Arc<crate::value::Function>,
    source: Arc<crate::stream::Stream>,
) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |sym, span| {
            let realized = crate::stream::realize(&source, sym, span)?;
            match realized.as_ref() {
                Stream::Empty => Ok(Arc::new(Stream::Empty)),
                Stream::Cons { head, tail } => {
                    let kept = apply_function(pred.clone(), vec![head.clone()], sym, span.clone())?;
                    match kept {
                        Value::bool(true) => {
                            let filtered_tail =
                                lazy_filter_stream(op, pred.clone(), Arc::clone(tail));
                            Ok(Arc::new(Stream::Cons {
                                head: head.clone(),
                                tail: filtered_tail,
                            }))
                        }
                        Value::bool(false) => {
                            // Skip the rejected element by handing back a fresh filter cell
                            // over the tail — `realize`'s loop keeps forcing it (no Rust
                            // recursion), so a run of N consecutive rejects is O(N), not a
                            // deferred Rust call per reject.
                            Ok(lazy_filter_stream(op, pred.clone(), Arc::clone(tail)))
                        }
                        other => Err(RuntimeError::new(
                            span.clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "wat::core::bool",
                                got: Box::new(ValueSnapshot::of(&other)),
                            },
                        )
                        .into()),
                    }
                }
                Stream::Thunk(_) | Stream::NativeThunk(_) => {
                    unreachable!("crate::stream::realize always returns Empty|Cons")
                }
            }
        },
    ))))
}

/// Build a lazy `Stream` stepping an already-resident `Vec<Value>` (a `Vector`, or a `List`
/// snapshot) by index. Each `NativeThunk` captures the `Arc<Vec<Value>>` handle (an `Arc`
/// clone, O(1)) and the next index; forcing it clones only the ONE element being yielded —
/// nothing is touched until the cell is actually forced (`take`/early-exit still short-circuits).
fn indexed_vec_stream(xs: Arc<Vec<Value>>, index: usize) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    if index >= xs.len() {
        return Arc::new(Stream::Empty);
    }
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |_sym, _span| {
            let head = xs[index].clone();
            let tail = indexed_vec_stream(Arc::clone(&xs), index + 1);
            Ok(Arc::new(Stream::Cons { head, tail }))
        },
    ))))
}

/// Build a lazy `Stream` stepping a `PersistentVector` (`crate::value::pvec::PVec`) by index.
/// Array get is a slice; Tree get is O(log n). `.clone()` is O(1) (Arc / RRB handle) —
/// the container handle is cloned once per step, never rebuilt, never walked
/// element-by-element.
fn indexed_pv_stream(pv: crate::value::pvec::PVec, index: usize) -> Arc<crate::stream::Stream> {
    use crate::stream::{NativeLazyCell, Stream};
    if index >= pv.len() {
        return Arc::new(Stream::Empty);
    }
    Arc::new(Stream::NativeThunk(NativeLazyCell::new(Arc::new(
        move |_sym, _span| {
            let head = pv
                .get(index)
                .expect("index < pv.len() checked at construction")
                .clone();
            let tail = indexed_pv_stream(pv.clone(), index + 1);
            Ok(Arc::new(Stream::Cons { head, tail }))
        },
    ))))
}

// ─── Arc-278 DESIGN-STONE seq-traversal-one-door — regression wall ───────────────────────────
//
// Written before `seqable->stream` went native. `keep` is the clean probe verb
// (EXPECTATIONS' trap-door #3): it already delegates through `:wat::core::seqable->stream`
// and this strike does NOT edit `keep` itself — only the converter it normalizes through.
// A wall, not a stopwatch: quadratic at n=4000 is ~12,000ms, linear is ~10ms — a
// three-order-of-magnitude gap no machine variance crosses.
#[cfg(test)]
mod seqable_to_stream_tests {
    use crate::freeze::{eval_in_frozen, startup_from_source};
    use crate::load::InMemoryLoader;
    use crate::runtime::{Environment, Value};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// `(into (Vector) (keep keep-all pv))` over a 4000-element `PersistentVector` must
    /// complete in under one second. The source is a `PersistentVector` (not a plain
    /// `Vector`) deliberately: PVec rest rebuilds-from-empty via unique mut
    /// (`collection/eval.rs`'s `PersistentVector` arm of `eval_rest`), the same
    /// expensive-per-step shape the DESIGN-STONE's own measurement used
    /// (`probe-pv-lazy-materialize-cost.wat`) — a plain `Vector<i64>` source clones too
    /// cheaply per step to blow the wall at this n. Asserts the absence of the O(n^2)
    /// `seqable->stream` walk (native steps by position).
    #[test]
    fn seqable_to_stream_keep_stays_under_wall_at_n4000() {
        const WORLD: &str = "\
(:wat::core::defn :cx::keep-all [x <- :wat::core::i64] -> :wat::core::Option<wat::core::i64>\n\
  (:wat::core::Some x))\n\
(:wat::core::defn :cx::build-pv [n <- :wat::core::i64] -> :wat::core::PersistentVector<wat::core::i64>\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::i64>  i <- :wat::core::i64]\n\
      -> :wat::core::PersistentVector<wat::core::i64>\n\
      (:wat::core::PersistentVector/conj acc i))\n\
    (:wat::core::PersistentVector)\n\
    (:wat::core::range 0 n)))\n\
";
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(
            "(:wat::core::length (:wat::core::into (:wat::core::Vector :wat::core::i64) \
              (:wat::core::keep :cx::keep-all (:cx::build-pv 4000))))"
        )
        .expect("parse the keep pipeline");

        let start = Instant::now();
        let result = eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
            .value_owned();
        let elapsed = start.elapsed();
        eprintln!("seqable_to_stream_keep_stays_under_wall_at_n4000: elapsed={elapsed:?}");

        assert_eq!(
            result,
            Value::i64(4000),
            "keep over 4000 elements must keep all 4000 — a wrong count means the gate is \
             measuring the wrong thing, not the absence of the quadratic"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "keep over 4000 elements took {elapsed:?} — quadratic is ~12,000ms, linear is \
             ~10ms; this wall (<1s) asserts the ABSENCE of the O(n^2) seqable->stream walk, \
             not a performance number (DESIGN-STONE seq-traversal-one-door)"
        );
    }
}

// ─── Arc-278 DESIGN-STONE seq-traversal-one-door, Strike 2a — regression wall ────────────────
//
// Written before `filter` went native. `filter`'s five wat clauses each stepped their eager
// source by `(rest coll)` — O(n) per step, O(n^2) per walk. The source here MUST be a
// `PersistentVector`, not a plain `Vector` — Strike 1's rider drew this same wall over a
// `Vector` and it passed before the native, because a `Vector`'s `rest` (a flat
// clone-and-collect) is cheap enough per element not to cross a one-second wall at n=4000,
// while PVec rest (rebuild-from-empty via unique mut, `collection/eval.rs`'s
// `PersistentVector` arm of `eval_rest`) missed it by ~35x. A wall, not a stopwatch:
// quadratic at n=4000 is ~12,000ms, linear is ~10ms — a three-order-of-magnitude gap no
// machine variance crosses.
#[cfg(test)]
mod filter_native_tests {
    use crate::freeze::{eval_in_frozen, startup_from_source};
    use crate::load::InMemoryLoader;
    use crate::runtime::{Environment, Value};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// `(into [] (filter pred pv))` over a 4000-element `PersistentVector` must complete in
    /// under one second. Asserts the absence of the O(n^2) wat `filter` walk; native
    /// `filter` composes through `seqable->stream`'s by-position walk.
    #[test]
    fn filter_native_stays_under_wall_at_n4000_persistentvector() {
        const WORLD: &str = "\
(:wat::core::defn :cx::keep-all [x <- :wat::core::i64] -> :wat::core::bool\n\
  true)\n\
(:wat::core::defn :cx::build-pv [n <- :wat::core::i64] -> :wat::core::PersistentVector<wat::core::i64>\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::i64>  i <- :wat::core::i64]\n\
      -> :wat::core::PersistentVector<wat::core::i64>\n\
      (:wat::core::PersistentVector/conj acc i))\n\
    (:wat::core::PersistentVector)\n\
    (:wat::core::range 0 n)))\n\
";
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(
            "(:wat::core::length (:wat::core::into (:wat::core::Vector :wat::core::i64) \
              (:wat::core::filter :cx::keep-all (:cx::build-pv 4000))))"
        )
        .expect("parse the filter pipeline");

        let start = Instant::now();
        let result = eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
            .value_owned();
        let elapsed = start.elapsed();
        eprintln!("filter_native_stays_under_wall_at_n4000_persistentvector: elapsed={elapsed:?}");

        assert_eq!(
            result,
            Value::i64(4000),
            "filter over 4000 elements with an all-true predicate must keep all 4000 — a wrong \
             count means the gate is measuring the wrong thing, not the absence of the quadratic"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "filter over 4000 elements took {elapsed:?} — quadratic is ~12,000ms, linear is \
             ~10ms; this wall (<1s) asserts the ABSENCE of the O(n^2) rest-walk, not a \
             performance number (DESIGN-STONE seq-traversal-one-door, Strike 2a)"
        );
    }
}
