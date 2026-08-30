//! `:wat::linkedlist::*` intrinsics — arc 255 Stone E-iii (set + list get
//! their homes), REGISTRY half of the two-home split `intrinsic/string.rs`
//! established.
//!
//! The 5 Rust-implemented `:wat::linkedlist::*` verbs, off the `:wat::core::`
//! junk-drawer (`:wat::core::List/*`) onto their own top-level namespace.
//!
//! ★ WHY `List` TAKES THE MARKED `:wat::linkedlist::` NAME, NOT the unmarked
//! `:wat::list::` — measured from the backing type, not taste:
//! `Arc<std::collections::LinkedList<Value>>` (`value.rs:340`) is the
//! **copy-on-write** flavor, the same side of the axis as `HashMap` /
//! `HashSet` / `Vector`, not the structurally-shared `rpds`-backed side
//! `PersistentMap`/`PersistentVector` sit on. The builder has ruled that a
//! persistent-backed list is coming; `:wat::list::` must stay FREE for that
//! flavor once it lands, the same reason `:wat::map::`/`:wat::vector::`
//! stayed free for the persistent siblings rather than being claimed by the
//! copy-on-write incumbent. `linkedlist` names what it is — a `LinkedList` —
//! same shape as `hashset`/`hashmap` spelling out the qualifier in full
//! (NOT `llist`, elided-and-ambiguous to a reader who doesn't already know
//! what it stands for — corrected by the orchestrator before this stone
//! shipped). See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-iii-set-and-list-get-their-homes.md`.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY
//! home — dispatch shim + `///` preamble only. The algorithms these handlers
//! call (`list_length_inner`, `list_conj_inner`, …) live in
//! `src/collection/eval.rs`, the NAMESPACE home — already the algorithm's
//! home before this stone, untouched by it (name-only rename; handler bodies
//! do not move).
//!
//! Both the old `:wat::core::List/*` spelling and this new one are LIVE
//! during Phase 1/2 of this stone (register, then move the corpus by
//! codemod); Phase 3 retires the old spelling, leaving this file as the
//! ONLY dispatch path — reached via `crate::intrinsic::registry().lookup`,
//! consulted BEFORE `runtime.rs`'s literal match
//! (`DESIGN-STONE-255.1c-guard-hoist.md`).
//!
//! ⚠ The bare TYPE constructor `:wat::core::List` does NOT move (STOP-3) —
//! it is arc 251's territory, registered separately in `intrinsic/list.rs`;
//! only the 5 slash-verbs below are this stone's territory.
//!
//! `conj` is a **PREPEND**, not an append — Clojure precedent, matching
//! `cons` (distinct from `Vector`'s/`HashSet`'s `conj`, both of which grow at
//! the opposite end/by insertion); see `list_conj_inner`'s own doc.

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, Value};

// ─── the 5 verbs ────────────────────────────────────────────────────────────
//
// arc 255 Stone O-iv-b — migrated to ALGEBRA. Each pair here was, before this stone, a
// hand-written AST shell PLUS a hand-written value twin (Stone N, named via the `value =` attribute)
// that each called the same `*_inner` fn, the value twin guarded only by `.expect
// ("arity-checked")` naming a check that happened on the OTHER door. One declaration now feeds
// both doors; the arity check is generated, and true on the door that raises it. See
// `src/intrinsic/vector.rs` (the worked example, O-iii).

/// `(:wat::linkedlist::length l)` → the number of elements in `l`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     l (:wat::core::List :- [T]) the list probed
/// @ret     :wat::core::i64 the number of elements in `l`
/// @example (:wat::linkedlist::length (:wat::core::List)) #=> 0
/// @example (:wat::linkedlist::length (:wat::core::List 1 2 3)) #=> 3
/// @see     :wat::linkedlist::empty?
#[wat_intrinsic(":wat::linkedlist::length")]
pub(crate) fn list_length(l: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::list_length_inner(l)
}

/// `(:wat::linkedlist::empty? l)` → whether `l` has zero elements.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     l (:wat::core::List :- [T]) the list probed
/// @ret     :wat::core::bool true iff `l` has zero elements
/// @example (:wat::linkedlist::empty? (:wat::core::List)) #=> true
/// @example (:wat::linkedlist::empty? (:wat::core::List 1)) #=> false
/// @see     :wat::linkedlist::length
#[wat_intrinsic(":wat::linkedlist::empty?")]
pub(crate) fn list_empty_q(l: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::list_empty_q_inner(l)
}

/// `(:wat::linkedlist::contains? l item)` → whether `item` occurs as an
/// element of `l`. O(N) linear scan (`LinkedList` has no indexing).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     l (:wat::core::List :- [T]) the list probed
/// @arg     item :T the candidate element
/// @ret     :wat::core::bool true iff `item` occurs in `l`
/// @example (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 2) #=> true
/// @example (:wat::linkedlist::contains? (:wat::core::List 1 2 3) 9) #=> false
/// @see     :wat::linkedlist::get
#[wat_intrinsic(":wat::linkedlist::contains?")]
pub(crate) fn list_contains_q(l: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::list_contains_q_inner(l, item)
}

/// `(:wat::linkedlist::get l i)` → `Some` of the element at index `i` in
/// `l`, or `None` on an out-of-range index. O(N) index walk (`LinkedList`
/// has no random access). Safe: never raises on OOB.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     l (:wat::core::List :- [T]) the list probed
/// @arg     i :wat::core::i64 the index looked up
/// @ret     (:wat::core::Option :- [T]) `Some` the element at `i`, or `None` on OOB
/// @example (:wat::linkedlist::get (:wat::core::List 1 2 3) 0) #=> (:wat::core::Some 1)
/// @example (:wat::linkedlist::get (:wat::core::List 1 2 3) 9) #=> :None
/// @see     :wat::linkedlist::contains?
#[wat_intrinsic(":wat::linkedlist::get")]
pub(crate) fn list_get(l: &Value, i: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::list_get_inner(l, i)
}

/// `(:wat::linkedlist::conj l item)` → a NEW `List` with `item`
/// **prepended** (Clojure `cons` semantics — the opposite end from
/// `Vector`'s/`HashSet`'s `conj`); the original `l` is UNCHANGED.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     l (:wat::core::List :- [T]) the list transformed
/// @arg     item :T the element prepended
/// @ret     (:wat::core::List :- [T]) `l` with `item` prepended
/// @example (:wat::linkedlist::length (:wat::linkedlist::conj (:wat::core::List) 1)) #=> 1
/// @see     :wat::linkedlist::length
#[wat_intrinsic(":wat::linkedlist::conj")]
pub(crate) fn list_conj(l: &Value, item: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::list_conj_inner(l, item)
}
