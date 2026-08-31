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
//!
//! Arc 255 Stone A-2-ii-b added an eighth verb, `sort$native` — outside the P6-c-W6 wave above
//! (a different arity-2 comparator-sort primitive, homed for the purity GATE it ships alongside
//! rather than this wave's read-only motive), but needing the identical `#[wat_intrinsic]`
//! file-scan visibility the header above explains, so it lives here rather than opening a file
//! for one verb.

// ─── THE DELEGATE TEMPLATE — one gotcha, twice paid for ────────────────────────
//
// A 1-ARITY delegate must forward its single `&WatAST` with `std::slice::from_ref(x)`,
// NOT `&[x.clone()]`. Both compile; only the first passes `clippy -D warnings`
// (`clippy::cloned_ref_to_slice_refs`). This fired on the FIRST 1-arity delegate of arc
// 255 Stone A-2-ii-b-0 (`:wat::core::type`) and again on all three of A-2-ii-b-1
// (`Some`/`Ok`/`Err`) — a rider copying an existing 2-arity delegate, where
// `&[a.clone(), b.clone()]` is correct and unavoidable, lands on the wrong idiom every
// time. Recorded here rather than in a brief because the template is what gets copied.
//
//     1 arg :  crate::runtime::eval_x(std::slice::from_ref(v), span, env, sym)
//     N args:  crate::runtime::eval_x(&[a.clone(), b.clone()], span, env, sym)

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
/// **Expand-time ground —** polymorphic collection op: reads no state, performs no effect.
/// Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Collections —
/// polymorphic intrinsics" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
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
/// **Expand-time ground —** polymorphic collection op: reads no state, performs no effect.
/// Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Collections —
/// polymorphic intrinsics" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
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
/// **Expand-time ground —** now a Rust intrinsic (was a wat `defclause`); reads no state,
/// performs no effect, and its out-of-range raise is a deterministic located abort — not
/// disqualifying (same class as `i64::/`'s division-by-zero, admitted above). Ruling relocated
/// from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a; Stone 118.B4-0); the
/// verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
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
/// **Expand-time ground —** polymorphic collection op: reads no state, performs no effect.
/// Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Collections —
/// polymorphic intrinsics" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
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
/// **Expand-time ground —** polymorphic collection op: reads no state, performs no effect.
/// Safe to evaluate while a `defmacro` body is being expanded. Ruling relocated from
/// `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a), from its "Collections —
/// polymorphic intrinsics" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
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
/// **Expand-time ground —** bounded iteration over a finite list: pure. Safe to evaluate
/// while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Collections — HOFs (bounded
/// iteration over finite lists)" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
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
/// **Expand-time ground —** bounded iteration over a finite list: pure. Safe to evaluate
/// while a `defmacro` body is being expanded. Ruling relocated from `macros/eval.rs`'s
/// expand-time allow-list (arc 255 expand-T4a), from its "Collections — HOFs (bounded
/// iteration over finite lists)" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
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

/// `(:wat::core::sort$native cmp xs) -> (:wat::core::Vector :- [T])` — arc 056/247/251: the
/// Rust comparator-sort primitive `sort`/`sort-by` (`wat/core.wat`) wrap; `cmp` is
/// `[T T :-> :wat::core::bool]`, `less?`-shaped (Clojure convention — the caller owns
/// ascending vs. descending via the predicate).
///
/// Arc 255 Stone A-2-ii-b — the gate and this homing ship in ONE stone, because a declaration
/// the door does not enforce is a lie. `eval_vec_sort_by` (`src/collection/transform.rs`) now
/// refuses a comparator that is not proven Pure ∧ Deterministic BEFORE any comparison runs —
/// a refusal fired mid-sort would already have run the caller's comparator on some pairs, the
/// exact effects the gate exists to prevent. Homing here retires three hand-lists that now
/// derive from this registration: the literal `:wat::core::sort$native` arm in `runtime.rs`'s
/// dispatch match, the `KNOWN_UNREVIEWED` row in `rete/purity.rs`, and the expand-time-residue
/// entry in `macros/eval.rs`'s `is_expand_time_legal` (its `registry().lookup_entry` door now
/// answers from `@ExpandTime` below instead of falling through to that hand-list).
///
/// **Purity ground — `@Purity Pure` is true BECAUSE OF THE GATE above it, not by assertion.**
/// Before this stone, an effectful comparator — e.g.
/// `(fn [a b] (do (println …) (< a b)))` — ran for real: measured firing 4 side effects
/// sorting a 3-element vector (`255-probe-can-a-user-make-sort-effectful.wat`), so `Pure`
/// would have been a claim a user could falsify in one line. `eval_vec_sort_by` classifies the
/// comparator against its OWN `closed_env` (`ClassifyCtx::Runtime`/`Static`, never the
/// caller's) and refuses anything not Pure ∧ Deterministic before the first comparison, so by
/// the time this fn's body is reached at all the comparator already IS pure and deterministic
/// — this declaration inherits that from the door standing in front of it, rather than
/// re-deriving it. Same warrant, same door, for `@Determinism Deterministic`.
///
/// **Totality ground — on `sort$native`'s OWN merits, not the comparator's.** `Total` is
/// deliberately NOT imposed on the comparator
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`), and the reason is NOT
/// the one first written here. It is not that accessors are `Partial` via `Option/expect`
/// (true, but not binding). **It is that the totality census has not run:** `@Totality` reads
/// `Total 29 · Partial 3 · Preserving 2 · Unreviewed 403`, and `Unreviewed` is default-deny by
/// design. Measured — even `sort/1`'s trivial default comparator fails `total?`, because
/// `:wat::core::<` (the polymorphic generic) is `Unreviewed` while its own per-type sibling
/// `:wat::i64::<` is ruled `Total`. So a `Total` demand today refuses EVERY caller, not for
/// being partial but for never having been looked at — and the only cheap way to silence that
/// gate is to GUESS `Total`, which is precisely the laundering `Unreviewed` exists to prevent.
/// Impose it after the census, when it costs nothing where true. Meanwhile sort stays
/// total regardless, because a comparator that raises just makes the sort raise (ordinary
/// propagation, not a domain hole this fn owns). Measured directly: a pathological but Pure ∧
/// Deterministic comparator returning an inconsistent ordering still yields a scrambled but
/// well-formed vector for any input — exit 0, no panic — so `sort$native` itself is `Total` on
/// its own merits.
///
/// **Expand-time ground —** pure ∧ deterministic (via the gate above) with no state read;
/// safe to evaluate while a `defmacro` body is being expanded.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     cmp [:T :T :-> :wat::core::bool] the `less?` comparator; refused before any comparison runs unless proven Pure ∧ Deterministic against its own closed environment
/// @arg     xs (:wat::core::Vector :- [T]) the vector sorted
/// @ret     (:wat::core::Vector :- [T]) a new vector holding `xs`'s elements ordered by `cmp`
/// @yields  cmp two elements of `xs` at a time — the pair being ordered; `cmp` returns whether the first sorts before the second. Called up to twice per comparison (the two-sided test that distinguishes Equal from Less/Greater — see `eval_vec_sort_by`), which is why an EFFECTFUL comparator leaks an implementation detail into observable output and is refused at the door
/// @example (:wat::core::sort$native (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool (:wat::core::< a b)) (:wat::core::Vector 3 1 2)) #=> (:wat::core::Vector 1 2 3)
/// ⚠ NO `@see :wat::core::sort` — and the reason is a real limit worth knowing.
/// `all_see_fqdns_resolve_to_registered_intrinsics` requires every `@see` to name a REGISTERED
/// INTRINSIC, and `sort`/`sort-by` are wat `defclause`s in `wat/core.wat`, not intrinsics. So a
/// Rust primitive cannot cite its own wat-level public wrapper through `@see`, even though that is
/// the single most useful cross-reference it has. The relationship is carried in the prose above
/// instead. (Measured: the gate went red on exactly this, `@see` is optional, and pointing it at
/// some other registered verb to satisfy the gate would be a worse lie than omitting it.)
#[wat_intrinsic(":wat::core::sort$native")]
pub(crate) fn eval_sort_native(
    cmp: &WatAST,
    xs: &WatAST,
    call_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::collection::transform::eval_vec_sort_by(&[cmp.clone(), xs.clone()], call_span, env, sym)
}
