//! `:wat::core::{length,empty?,nth,last,rest,reverse,range}` — arc 255 Stone P6-c-W6, the first
//! wave into `:wat::core::` (deliberately the half that needs no `effectful_by_prefix` widening —
//! see `docs/arc/2026/06/255-builtin-registry/NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md`).
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W6-core-collection-readers.md`.
//!
//! Seven verbs, moved verbatim out of `runtime.rs`'s giant match (`length`/`empty?`/`nth`) and
//! `collection/eval.rs` (`rest`) / `collection/transform.rs` (`last`/`reverse`/`range`) into THIS
//! file, with their real arities declared and every hand-rolled `args.len() != N` guard retired.
//! Moved here — not left in place — so `rete::purity::completeness_gate::dispatch_verbs`'s
//! `#[wat_intrinsic]` file scan (scoped to `src/intrinsic/**`) can still see them: their literal
//! match-arm text is gone from `dispatch_keyword_head_value`, and a handler homed outside
//! `src/intrinsic/` would otherwise vanish from that gate's population entirely — a real, MEASURED
//! hazard this wave hit (see the STONE commit message for the full account), not a style choice.
//!
//! None of the seven runs code it did not write, and none forces a lazy `Stream` cell: `length`/
//! `empty?`/`rest`/`nth`/`reverse` each route through a `StreamContainer`/`MapContainer`
//! capability gate (`measurable()`/`has_tail()`/`nth_indexable()`/`ordered()`) that Stone
//! 118.B4-iii — THE WALL — already set `false` for `Stream`, so a lazy receiver is refused by a
//! `TypeMismatch` before any cell is realized; `last` and `range` never touch `StreamContainer` at
//! all (`last` via `require_vec`'s `Value::Vec`-only gate; `range` takes two `i64`s and never
//! receives a collection argument). All seven: Pure, Deterministic.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{
    eval_inner, require_i64, require_vec, Environment, EvalBreak, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use std::sync::Arc;

// ─── Arc 237 Stone 237.7a — :wat::core::length intrinsic ─────────────────────

/// `(:wat::core::length <collection>) -> :wat::core::i64` — arc 237 Stone 237.7a.
///
/// Polymorphic collection-length primitive: ∀T. T -> i64.
/// Mirrors `eval_type` in shape: arity-1, eval arg, match Value variant.
/// Accepted variants:
/// - `Value::Vec(..)` → vector length
/// - `Value::wat__std__HashMap(..)` → map entry count
/// - `Value::wat__std__HashSet(..)` → set element count
/// - `Value::wat__core__List(..)` → list element count
///
/// All other variants produce a teaching `RuntimeError::TypeMismatch`.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only classifies the already-evaluated receiver via
/// `MapContainer`/`StreamContainer` capability gates (`measurable()`) and reads a length/count
/// off it — no `eval_inner`/`apply_function` on caller-supplied code anywhere. Stone 118.B4-iii
/// — THE WALL: `measurable()` is `false` for `StreamContainer::Stream`, so a lazy `(Stream :- [T])`
/// falls to the `None`/`Some(_)` `TypeMismatch` arms below (teaching `:wat::stream::next`) rather
/// than forcing any cell — no thunk is ever forced. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     xs :T the collection probed — (Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T]); a (Stream :- [T]) is refused (`measurable()` gate excludes it — see :wat::stream::next)
/// @ret     :wat::core::i64 the element/entry count
/// @example (:wat::core::length (:wat::core::Vector 1 2 3)) #=> 3
/// @see     :wat::core::empty?
#[wat_intrinsic(":wat::core::length")]
pub(crate) fn eval_length(
    xs: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::length";
    let arg_val = eval_inner(xs, env, sym)?.value_owned();
    // Arc-278 strike A — map-family arms route through MapContainer (measurable capability).
    // The capability DRIVES the accepted set: the `if m.measurable()` guard is the genuine gate,
    // not a debug_assert. Exhaustive match over the closed MapContainer enum — NO `_`. Adding a
    // new keyed container forces this arm to be updated before the code compiles.
    use crate::collection::map_container::MapContainer;
    match MapContainer::of_value(&arg_val) {
        Some(m) if m.measurable() => return match m {
            MapContainer::HashMap => crate::collection::eval::hashmap_length_inner(&arg_val),
            MapContainer::PersistentMap => crate::collection::eval::persistentmap_length_inner(&arg_val),
            // Arc-278-A2 — Record: length = field count (fields.len()), no registry needed.
            MapContainer::Record => crate::collection::eval::record_length_inner(&arg_val),
        },
        Some(_) => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
        None => {}
    }
    // Arc-278 seq-1a — seq-family arms route through StreamContainer (measurable capability).
    // The capability DRIVES the accepted set: the `if c.measurable()` guard is the genuine gate.
    // Exhaustive match over the closed StreamContainer enum — NO `_`. Adding a new seq container
    // forces this arm to be updated before the code compiles.
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&arg_val) {
        Some(c) if c.measurable() => match c {
            StreamContainer::Vector => crate::collection::eval::vector_length_inner(&arg_val),
            StreamContainer::PersistentVector => crate::collection::eval::persistentvector_length_inner(&arg_val),
            StreamContainer::HashSet => crate::collection::eval::hashset_length_inner(&arg_val),
            StreamContainer::List => crate::collection::eval::list_length_inner(&arg_val),
            // seq-1b — filled
            StreamContainer::Tuple => crate::collection::eval::tuple_length_inner(&arg_val),
            StreamContainer::WatAstList => crate::collection::eval::watastlist_length_inner(&arg_val),
            // Arc 118 — measurable() gate excludes Stream (length on a lazy/infinite seq diverges):
            StreamContainer::Stream => unreachable!("measurable() gate excludes Stream"),
        },
        Some(_) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
        None => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
    }
}

// ─── Arc 237 Stone 237.7b-i — :wat::core::empty? ────────────────────────────

/// `(:wat::core::empty? <collection>) -> :wat::core::bool` — arc 237 Stone 237.7b-i.
///
/// Polymorphic collection-empty predicate: ∀T. T -> bool.
/// Mirrors `eval_length` in shape: arity-1, eval arg, match Value variant.
/// Accepted variants:
/// - `Value::Vec(..)` → true iff vector is empty
/// - `Value::wat__std__HashMap(..)` → true iff map has no entries
/// - `Value::wat__std__HashSet(..)` → true iff set has no elements
/// - `Value::wat__core__List(..)` → true iff list has no elements
///
/// All other variants produce a teaching `RuntimeError::TypeMismatch`.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only classifies the already-evaluated receiver via
/// `MapContainer`/`StreamContainer` capability gates (`measurable()`) and reads a boolean off
/// it — no `eval_inner`/`apply_function` on caller-supplied code anywhere. Stone 118.B4-iii —
/// THE WALL: the hand-written Stream early-realize branch that used to sit here (forcing one
/// step to decide Empty vs Cons) is DELETED — `measurable()` is `false` for
/// `StreamContainer::Stream`, so a lazy `(Stream :- [T])` now falls through to a `TypeMismatch`
/// (teaching `:wat::stream::next`) like any other non-measurable container; no thunk is ever
/// forced. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     xs :T the collection probed — (Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T]); a (Stream :- [T]) is refused (`measurable()` gate excludes it — see :wat::stream::next)
/// @ret     :wat::core::bool whether the collection has zero elements/entries
/// @example (:wat::core::empty? (:wat::core::Vector 1 2 3)) #=> false
/// @see     :wat::core::length
#[wat_intrinsic(":wat::core::empty?")]
pub(crate) fn eval_empty(
    xs: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::empty?";
    let arg_val = eval_inner(xs, env, sym)?.value_owned();
    // Arc-278 strike A — map-family arms route through MapContainer (measurable capability).
    // The capability DRIVES the accepted set: the `if m.measurable()` guard is the genuine gate,
    // not a debug_assert. Exhaustive match over the closed MapContainer enum — NO `_`. Adding a
    // new keyed container forces this arm to be updated before the code compiles.
    use crate::collection::map_container::MapContainer;
    match MapContainer::of_value(&arg_val) {
        Some(m) if m.measurable() => return match m {
            MapContainer::HashMap => crate::collection::eval::hashmap_empty_q_inner(&arg_val),
            MapContainer::PersistentMap => crate::collection::eval::persistentmap_empty_q_inner(&arg_val),
            // Arc-278-A2 — Record: empty? = fields.is_empty(), no registry needed.
            MapContainer::Record => crate::collection::eval::record_empty_q_inner(&arg_val),
        },
        Some(_) => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
        None => {}
    }
    // Arc-278 seq-1a — seq-family arms route through StreamContainer (measurable capability).
    // The capability DRIVES the accepted set: the `if c.measurable()` guard is the genuine gate.
    // Exhaustive match over the closed StreamContainer enum — NO `_`. Adding a new seq container
    // forces this arm to be updated before the code compiles.
    //
    // Stone 118.B4-iii — THE WALL: the hand-written Stream early-realize branch that used to sit
    // here (forcing one step to decide Empty vs Cons) is DELETED. It routed AROUND this very
    // gate — `measurable()` was already `false` for Stream, and the special case let `empty?`
    // ignore that. Deleting it means Stream now falls through to `Some(_)` below like any other
    // non-measurable container, and is refused uniformly.
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&arg_val) {
        Some(c) if c.measurable() => match c {
            StreamContainer::Vector => crate::collection::eval::vector_empty_q_inner(&arg_val),
            StreamContainer::PersistentVector => crate::collection::eval::persistentvector_empty_q_inner(&arg_val),
            StreamContainer::HashSet => crate::collection::eval::hashset_empty_q_inner(&arg_val),
            StreamContainer::List => crate::collection::eval::list_empty_q_inner(&arg_val),
            // seq-1b — filled
            StreamContainer::Tuple => crate::collection::eval::tuple_empty_q_inner(&arg_val),
            StreamContainer::WatAstList => crate::collection::eval::watastlist_empty_q_inner(&arg_val),
            // measurable() gate excludes Stream — named arm, genuinely dead, compiler-forced:
            StreamContainer::Stream => unreachable!(
                "measurable() gate excludes Stream (Stone 118.B4-iii — THE WALL: use :wat::stream::next)"
            ),
        },
        // Stone 118.B4-iii — THE WALL: Stream lands here now (measurable()==false, and no early
        // realize left to intercept it first). A lazy seq's emptiness is decidable in one force,
        // but the wall closes the verb anyway — advance it with `:wat::stream::next`, whose
        // `(NextOutcome :- [T])::Exhausted` already answers exactly what `empty?` was asked.
        Some(StreamContainer::Stream) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), (HashSet :- [T]), Tuple, or WatAST — a lazy (Stream :- [T]) has no empty?; advance it with :wat::stream::next, whose (NextOutcome :- [T])::Exhausted answers what empty? was asked",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
        Some(_) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
        None => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "(Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]), (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])",
            got: Box::new(ValueSnapshot::of(&arg_val))
        }).into()),
    }
}

/// `(:wat::core::nth coll i)` — stone 118.B4-0: the general positional accessor, the
/// RUNTIME-index generalization of `first`/`second`/`third` (`runtime.rs`, same shape, `index`
/// taken from a runtime i64 instead of a Rust constant). Promoted from a wat `defclause` to a
/// Rust intrinsic specifically so a `defmacro` program body — which evaluates only through
/// `dispatch_keyword_head`, this function's caller — can call it (B4-ii's codemod tripped on
/// exactly that). `:wat::core::nth-spec` (`wat/core.wat`) is the retained wat ORACLE; a
/// differential test (`wat-tests/core/core-nth-differential.wat`) proves they agree.
///
/// CONTRACT (unchanged from the retired clause): raise, uniformly, `"nth: index out of range"`
/// on out-of-range — never an `Option`, never a container-specific message. Gated by
/// `nth_indexable()` (`seq_container.rs`), NOT `indexable()` — see that method's doc for why.
///
/// Stream has no O(1) nth (`nth_indexable()`'s doc). It walks: `realize` one cell, and if that
/// is not the target index, recurse on the tail — exactly `i+1` forces for index `i`, one per
/// step. Realizing the whole stream first and then indexing would reintroduce the O(n)
/// retention stone B3 deleted; the walk is the honest cost, not a regression.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (2) arity
/// declared; the hand-rolled `args.len() != 2` guard this wave retires lived right here.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only classifies the already-evaluated receiver via
/// `StreamContainer::nth_indexable()` and reads one element off it — no
/// `eval_inner`/`apply_function` on caller-supplied code anywhere. Stone 118.B4-iii — THE
/// WALL: `nth_indexable()` is `false` for `StreamContainer::Stream`, so a lazy `(Stream :- [T])`
/// falls to a `TypeMismatch` (teaching `(drop s i)` + `:wat::stream::next`) before any walk
/// starts — no thunk is ever forced. Genuinely partial (raises via `panic_any` on out-of-range,
/// same as `first`/`second`/`third`) but Pure ∧ Deterministic on the purity/determinism axes
/// this registry's `@Purity`/`@Determinism` tags measure.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Projection
/// @arg     xs (:wat::core::Vector :- [T]) the receiver — this call also accepts (PersistentVector :- [T]), (List :- [T]), or a WatAST list form (returning :wat::WatAST); a (Stream :- [T]) is refused (`nth_indexable()` gate excludes it — use (drop s i) then :wat::stream::next)
/// @arg     idx :wat::core::i64 the zero-based index; raises "nth: index out of range" if out of bounds
/// @ret     :T the element at `idx`
/// @example (:wat::core::nth (:wat::core::Vector 10 20 30) 1) #=> 20
/// @see     :wat::core::last
#[wat_intrinsic(":wat::core::nth")]
pub(crate) fn eval_nth(
    xs: &WatAST,
    idx: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::nth";
    let v = eval_inner(xs, env, sym)?.value_owned();
    let idx_val = eval_inner(idx, env, sym)?.value_owned();
    let index_i64 = match idx_val {
        Value::i64(n) => n,
        other => {
            return Err(RuntimeError::new(
                idx.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "i64",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // Uniform out-of-range raise — the ONE CONTRACT DECISION (DESIGN-STONE-118.B4-0): the
    // native and `nth-spec` must produce the exact same message on every receiver, never a
    // per-container variant (contrast `eval_positional_accessor`'s arms, which DO vary
    // their message per container — that shape is deliberately NOT reused here).
    //
    // ⚠ MUST panic (`panic_any` + `AssertionPayload`), NOT return a `RuntimeError`: the wat
    // oracle `nth-spec` raises via `Option/expect`/`assertion-failed!`, both of which panic. A
    // `RuntimeError` return surfaces at a process boundary as a DIFFERENT `LociDiedError`
    // variant (not `Panic`) — measured directly: it broke `wat-tests/core/core-nth.wat`'s
    // pre-existing `nth-past-end-*-raises` rows (STOP-4, caught and fixed during this strike).
    fn out_of_range(span: Span) -> EvalBreak {
        let frames = crate::value::snapshot_call_stack();
        let payload = crate::assertion::AssertionPayload {
            message: "nth: index out of range".into(),
            actual: None,
            expected: None,
            location: Some(span),
            frames,
            upstream_chain: None,
            thread_name: std::thread::current().name().map(String::from),
            raised_error: None,
        };
        std::panic::panic_any(payload)
    }

    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.nth_indexable() => {
            if index_i64 < 0 {
                return Err(out_of_range(xs.span().clone()));
            }
            let index = index_i64 as usize;
            match container {
                StreamContainer::Vector => {
                    let Value::Vec(items) = v else {
                        unreachable!("of_value⇒Vector")
                    };
                    items
                        .get(index)
                        .cloned()
                        .ok_or_else(|| out_of_range(xs.span().clone()))
                }
                StreamContainer::PersistentVector => {
                    let Value::wat__core__PersistentVector(pv) = v else {
                        unreachable!("of_value⇒PersistentVector")
                    };
                    pv.get(index)
                        .cloned()
                        .ok_or_else(|| out_of_range(xs.span().clone()))
                }
                StreamContainer::List => {
                    let Value::wat__core__List(items) = v else {
                        unreachable!("of_value⇒List")
                    };
                    items
                        .iter()
                        .nth(index)
                        .cloned()
                        .ok_or_else(|| out_of_range(xs.span().clone()))
                }
                StreamContainer::WatAstList => {
                    let Value::wat__WatAST(ast) = v else {
                        unreachable!("of_value⇒WatAstList")
                    };
                    match &*ast {
                        WatAST::List(children, _) => children
                            .get(index)
                            .map(|c| Value::wat__WatAST(Arc::new(c.clone())))
                            .ok_or_else(|| out_of_range(xs.span().clone())),
                        _ => unreachable!(
                            "StreamContainer::of_value guarantees WatAST::List for WatAstList"
                        ),
                    }
                }
                // Stone 118.B4-iii — THE WALL: `nth_indexable()` is FALSE for Stream now, so
                // this arm is dead — no `container` value can reach it as `Stream`. Named, not
                // folded into `_`, so a future capability change that reopens Stream here is a
                // compile error, not a silent revival. Built one stone ago (B4-0) — the O(i)
                // walk this arm performed is exactly the quadratic-under-a-loop hazard the wall
                // exists to make un-spellable: `(nth s i)` read like O(1) and was O(i).
                StreamContainer::Stream => unreachable!(
                    "nth_indexable() gate excludes Stream (Stone 118.B4-iii — THE WALL: use (drop s i) then :wat::stream::next)"
                ),
                // nth_indexable() gate excludes Tuple/HashSet — named arms, genuinely dead,
                // compiler-forced (exhaustiveness guarantee, seq_container.rs's own doc).
                StreamContainer::Tuple | StreamContainer::HashSet => {
                    unreachable!("nth_indexable() gate excludes Tuple and HashSet")
                }
            }
        }
        // Stone 118.B4-iii — THE WALL: Stream lands here now (nth_indexable()==false). A lazy
        // seq has no O(1) positional access — `(nth s i)` was O(i) via `realize`, walking `i+1`
        // cells with syntax that reads like the O(1) Vector case. Spell it honestly instead:
        // `(drop s i)` then `:wat::stream::next`.
        Some(StreamContainer::Stream) => Err(RuntimeError::new(
            xs.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "(Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), or WatAST — a lazy (Stream :- [T]) has no O(1) nth; use (drop s i) then :wat::stream::next",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
        // ∅ N/A: Tuple (heterogeneous — runtime index can't be typed) / HashSet (unordered).
        Some(_) => Err(RuntimeError::new(
            xs.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vector, PersistentVector, List, or WatAST list",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
        None => Err(RuntimeError::new(
            xs.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vector, PersistentVector, List, or WatAST list",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::last xs) -> (:wat::core::Option :- [T])` — arc 047.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here. No
/// `call_span`/`list_span` context param survives: `require_vec`'s own `TypeMismatch` uses
/// `rust_caller_span!()`, not the call span — nothing else in this body raised on a span.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only accepts a plain `Value::Vec` (`require_vec` — every other
/// receiver, including a `Stream`, is refused before any classification even runs) and reads its
/// final element — no `eval_inner`/`apply_function` on caller-supplied code, and no
/// `StreamContainer` gate to name: `require_vec` never routes a `Stream` value through one, so
/// no thunk is ever forced. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Projection
/// @arg     xs (:wat::core::Vector :- [T]) the vector probed
/// @ret     (:wat::core::Option :- [T]) the last element, or `None` if `xs` is empty
/// @example (:wat::core::last (:wat::core::Vector 1 2 3)) #=> (:wat::core::Some 3)
/// @see     :wat::core::nth
#[wat_intrinsic(":wat::core::last")]
pub(crate) fn eval_vec_last(
    xs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let xs = require_vec(":wat::core::last", eval_inner(xs, env, sym)?.value_owned())?;
    Ok(Value::Option(Arc::new(xs.last().cloned())))
}

/// `(:wat::core::rest xs)` — everything after the first element. Four dispatch arms:
///
/// - `Value::Vec` — returns a new `Vec<T>` of the tail (mirrors `slice[1..]`).
/// - `Value::wat__core__List` — returns a new `List<T>` of the tail; preserves List type identity.
/// - `Value::wat__WatAST(WatAST::List)` — form-value decomposition: returns a new `WatAST::List`
///   of the tail forms, preserving the surrounding span (arc 249 Stone 249.3a-ii).
///   This arm is reachable only in macro-expansion contexts where checker discipline is
///   relaxed; type-checked user code calling `rest` on a form-value is rejected at check time
///   (checker's `rest` arm at `src/check.rs`'s `check_call` match rejects non-Vec/non-List types).
/// - `Value::wat__core__PersistentVector` — rebuild-from-empty via unique `push_back_mut`
///   (stays Array). Preserves PersistentVector type identity.
///
/// Runtime error if the Vec/List/form is empty.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here. No
/// `call_span`/`list_span` context param survives: every error this body raises already carries
/// the receiver's own span (`args[0].span()`, now `xs.span()`); nothing here used the list span.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only classifies the already-evaluated receiver via
/// `StreamContainer::has_tail()` and rebuilds a same-kind tail collection — no
/// `eval_inner`/`apply_function` on caller-supplied code anywhere. Stone 118.B4-iii — THE WALL:
/// `has_tail()` is `false` for `StreamContainer::Stream`, so a lazy `(Stream :- [T])` falls to a
/// `TypeMismatch` (teaching `:wat::stream::next`) before any cell is forced — `rest` used to
/// force one cell to discard it (the same cost as `next`, but the name hid the force); the wall
/// deleted that path. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Projection
/// @arg     xs (:wat::core::Vector :- [T]) the receiver; identity-preserving — this call also accepts (List :- [T]), (PersistentVector :- [T]), or a WatAST list form, each returning the same container kind; a (Stream :- [T]) is refused (`has_tail()` gate excludes it — see :wat::stream::next)
/// @ret     (:wat::core::Vector :- [T]) every element after the first
/// @example (:wat::core::rest (:wat::core::Vector 1 2 3)) #=> (:wat::core::Vector 2 3)
/// @see     :wat::core::last
#[wat_intrinsic(":wat::core::rest")]
pub(crate) fn eval_rest(
    xs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let v = eval_inner(xs, env, sym)?.value_owned();
    // Arc-278 strike 2 — classify via the registry (StreamContainer::of_value + has_tail()).
    // The registry is the single source of truth; dispatch arms below are per-container
    // implementation only — no classification logic lives here.
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.has_tail() => {
            // Dispatch: each has_tail container computes its tail.
            // Identity-preserving: rest(Container<T>) → Container<T>.
            match container {
                StreamContainer::Vector => {
                    let Value::Vec(items) = v else { unreachable!("of_value⇒Vector") };
                    if items.is_empty() {
                        return Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty Vec".into()
                        }).into());
                    }
                    let out: Vec<Value> = items.iter().skip(1).cloned().collect();
                    Ok(Value::Vec(Arc::new(out)))
                }
                // Arc 220 Stone 220.4 — List: rest returns a new List (tail after first element).
                // Maintains type identity: List/rest → List (not Vec).
                StreamContainer::List => {
                    let Value::wat__core__List(items) = v else { unreachable!("of_value⇒List") };
                    if items.is_empty() {
                        return Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty List".into()
                        }).into());
                    }
                    let out: std::collections::LinkedList<Value> = items.iter().skip(1).cloned().collect();
                    Ok(Value::wat__core__List(Arc::new(out)))
                }
                // Arc 249 Stone 249.3a-ii — form-value decomposition: WatAST::List/rest →
                // a new WatAST::List of the tail. Maintains form identity (List/rest → List),
                // mirroring the wat__core__List arm above. Empty form → MalformedForm;
                // of_value guarantees this is a WatAST::List so non-List branch is unreachable.
                StreamContainer::WatAstList => {
                    let Value::wat__WatAST(ast) = v else { unreachable!("of_value⇒WatAstList") };
                    match &*ast {
                        WatAST::List(children, span) => {
                            if children.is_empty() {
                                return Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::MalformedForm {
                                    head: ":wat::core::rest".into(),
                                    reason: "cannot take rest of empty form".into()
                                }).into());
                            }
                            let tail: Vec<WatAST> = children.iter().skip(1).cloned().collect();
                            Ok(Value::wat__WatAST(Arc::new(WatAST::List(tail, span.clone()))))
                        }
                        // Unreachable: of_value only returns WatAstList for List forms.
                        _ => unreachable!("StreamContainer::of_value guarantees WatAST::List for WatAstList"),
                    }
                }
                // Arc-278-0b — PersistentVector: rest returns a new PersistentVector (tail after first element).
                // Maintains type identity: PersistentVector/rest → PersistentVector (not Vec).
                StreamContainer::PersistentVector => {
                    let Value::wat__core__PersistentVector(pv) = v else { unreachable!("of_value⇒PersistentVector") };
                    if pv.is_empty() {
                        return Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::MalformedForm {
                            head: ":wat::core::rest".into(),
                            reason: "cannot take rest of empty PersistentVector".into()
                        }).into());
                    }
                    // Rebuild-from-empty via unique mut — stays Array.
                    let mut out: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
                    for elem in pv.iter().skip(1) {
                        out.push_back_mut(elem.clone());
                    }
                    Ok(Value::wat__core__PersistentVector(out))
                }
                // Stone 118.B4-iii — THE WALL: `has_tail()` is FALSE for Stream now, so this
                // arm is dead — no `container` value can reach it as `Stream`. Named, not `_`,
                // so a future capability change that reopens Stream here is a compile error, not
                // a silent revival. `rest` forced one cell to discard it — the same cost as
                // `next`, but the name hid the force; the wall closes that.
                StreamContainer::Stream => unreachable!(
                    "has_tail() gate excludes Stream (Stone 118.B4-iii — THE WALL: use :wat::stream::next)"
                ),
                // has_tail() gate excludes these — named arms, genuinely dead, compiler-forced:
                StreamContainer::Tuple | StreamContainer::HashSet =>
                    unreachable!("has_tail() gate excludes Tuple/HashSet"),
            }
        }
        // ∅ N/A: container has no tail (Tuple, HashSet — nature forbids it). Stone 118.B4-iii —
        // THE WALL: Stream lands here too now (has_tail()==false) — a lazy seq has no `rest`;
        // advance it with `:wat::stream::next`, whose `NextOutcome<T> = Item(value, rest) |
        // Exhausted` is the only door a Stream yields through.
        Some(StreamContainer::Stream) => Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "(Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), or WatAST — a lazy (Stream :- [T]) has no rest; advance it with :wat::stream::next ((NextOutcome :- [T]) = Item(value, rest) | Exhausted)",
            got: Box::new(ValueSnapshot::of(&v))
        }).into()),
        Some(_) => Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: ":wat::core::rest".into(),
            expected: "Vec, List, PersistentVector, or list form",
            got: Box::new(ValueSnapshot::of(&v))
        }).into()),
        // Not a sequence container (or WatAST non-List form — preserve that specific error).
        None => match v {
            Value::wat__WatAST(ast) => Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::rest".into(),
                expected: "Vec, List, or list form",
                got: Box::new(ValueSnapshot::of(&Value::wat__WatAST(ast)))
            }).into()),
            other => Err(RuntimeError::new(xs.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: ":wat::core::rest".into(),
                expected: "Vec, List, or PersistentVector",
                got: Box::new(ValueSnapshot::of(&other))
            }).into()),
        }
    }
}

/// `(:wat::core::reverse xs) -> xs's own container type` — arc-278 strike 3.
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only classifies the already-evaluated receiver via
/// `StreamContainer::ordered()` and rebuilds a same-kind reversed collection — no
/// `eval_inner`/`apply_function` on caller-supplied code anywhere. `ordered()` is `false` for
/// `StreamContainer::Stream` (and Tuple/WatAstList/HashSet), so a lazy `(Stream :- [T])` falls to
/// the catch-all `TypeMismatch` below — no thunk is ever forced. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     xs (:wat::core::Vector :- [T]) the sequence reversed; this call also accepts (PersistentVector :- [T]) or (List :- [T]), each returning the same container kind — a (Stream :- [T]), Tuple, HashSet, or WatAST form is refused (`ordered()` gate excludes them)
/// @ret     (:wat::core::Vector :- [T]) `xs`'s elements in reverse order
/// @example (:wat::core::reverse (:wat::core::Vector 1 2 3)) #=> (:wat::core::Vector 3 2 1)
/// @see     :wat::core::range
#[wat_intrinsic(":wat::core::reverse")]
pub(crate) fn eval_vec_reverse(
    xs: &WatAST,
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let v = eval_inner(xs, env, sym)?.value_owned();
    // Arc-278 strike 3 — classify via the registry (StreamContainer::of_value + ordered()).
    // Arc-278 strike 4 — inner dispatch is exhaustive over the closed StreamContainer enum (no `_`).
    use crate::collection::seq_container::StreamContainer;
    match StreamContainer::of_value(&v) {
        Some(container) if container.ordered() => match container {
            StreamContainer::Vector => {
                let Value::Vec(items) = v else {
                    unreachable!("of_value⇒Vector")
                };
                let mut out = (*items).clone();
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
                let Value::wat__core__List(items) = v else {
                    unreachable!("of_value⇒List")
                };
                let out: std::collections::LinkedList<Value> = items.iter().rev().cloned().collect();
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
///
/// Arc 255 Stone P6-c-W6 — moved verbatim into `#[wat_intrinsic]` with its real (2) arity
/// declared; the hand-rolled `args.len() != 2` guard this wave retires lived right here. No
/// `call_span`/`list_span` context param survives: `require_i64`'s own `TypeMismatch` uses
/// `rust_caller_span!()`, not the call span, and nothing else in this body raised on a span —
/// there is no collection argument at all to mis-type against.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body performs pure i64 arithmetic (`start..end`) with no receiver
/// to classify at all — `range` never touches `StreamContainer`/a `Stream`, so there is no
/// gate to name and no thunk that could be forced. Pure ∧ Deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     start :wat::core::i64 the inclusive lower bound
/// @arg     end :wat::core::i64 the exclusive upper bound
/// @ret     (:wat::core::Vector :- [:wat::core::i64]) `start, start+1, …, end-1`; empty if `start >= end`
/// @example (:wat::core::range 0 3) #=> (:wat::core::Vector 0 1 2)
/// @see     :wat::core::reverse
#[wat_intrinsic(":wat::core::range")]
pub(crate) fn eval_vec_range(
    start: &WatAST,
    end: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let start = require_i64(":wat::core::range", eval_inner(start, env, sym)?.value_owned())?;
    let end = require_i64(":wat::core::range", eval_inner(end, env, sym)?.value_owned())?;
    let items: Vec<Value> = if start <= end {
        (start..end).map(Value::i64).collect()
    } else {
        Vec::new()
    };
    Ok(Value::Vec(Arc::new(items)))
}
