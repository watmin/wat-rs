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
use crate::runtime::{Environment, EvalBreak, SymbolTable, Value};
use crate::span::Span;

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
/// @Total         Unreviewed
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
    crate::collection::eval::length_of(xs, list_span, env, sym)
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
/// @Total         Unreviewed
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
    crate::collection::eval::empty_of(xs, list_span, env, sym)
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
/// @Total         Unreviewed
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
    crate::collection::eval::nth_of(xs, idx, env, sym)
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
/// **Totality ground —** returns `(Option :- [T])` unconditionally (`None` on an empty
/// Vector, never a raise) — no domain hole. Arc 255 Stone P6-c-W6, grouped with `reverse`/
/// `range` in `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that
/// list's, made by reading the implementation. (`rest` is excluded from that ruling: it
/// raises `MalformedForm` on an empty receiver — genuinely partial, per `purity.rs`'s
/// pure∧det entry for this family.)
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
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
    crate::collection::transform::eval_vec_last(xs, env, sym)
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
/// @Total         Unreviewed
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
    crate::collection::eval::eval_rest(xs, env, sym)
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
/// **Totality ground —** returns a same-kind collection for any receiver its `ordered()`
/// gate admits — no domain hole. Arc 255 Stone P6-c-W6, grouped with `last`/`range` in
/// `rete/purity.rs`'s `total` sub-list (arc 255 total-T4a); the verdict is that list's, made
/// by reading the implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
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
    crate::collection::transform::eval_vec_reverse(xs, call_span, env, sym)
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
/// **Totality ground —** returns a `Vector` (empty when `start >= end`) for any two i64s —
/// no domain hole. Arc 255 Stone P6-c-W6, grouped with `last`/`reverse` in `rete/purity.rs`'s
/// `total` sub-list (arc 255 total-T4a); the verdict is that list's, made by reading the
/// implementation.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Total
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
    crate::collection::transform::eval_vec_range(start, end, env, sym)
}
