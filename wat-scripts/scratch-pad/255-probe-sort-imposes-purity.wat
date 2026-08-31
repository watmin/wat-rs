;; Scratch probe — arc 255 Stone A-2-ii-b. `sort$native` now REFUSES an impure/nondeterministic
;; comparator AT THE DOOR, before any comparison runs (`eval_vec_sort_by`,
;; `src/collection/transform.rs`) — and that door is why `sort$native` may now declare
;; `@Purity Pure` / `@Determinism Deterministic` for real (`src/intrinsic/collection.rs`).
;;
;; Companion to `255-probe-can-a-user-make-sort-effectful.wat` (pre-gate, arc 255): that probe
;; measured the identical effectful comparator running for real — 4 side effects on a
;; 3-element vector, printed BEFORE the sorted result, exit 0. THIS probe re-runs the same
;; shape POST-gate and expects the opposite: the comparator's own `println` must NEVER fire —
;; the refusal happens before the first comparison, not mid-sort. STOP-1's acceptance row is
;; that zero effects are observable before the error.
;;
;; Expected, in order:
;;   1. `:user::pure-surface` prints three ordinary, UNCHANGED results — `sort/1` (default `<`),
;;      `sort/2` (user comparator), `sort-by` (pure key fn) all still work exactly as before the
;;      gate landed; the door only refuses what was never legitimately pure to begin with.
;;   2. `:user::effectful` then raises a located error naming the offending comparator head —
;;      and "SIDE-EFFECT-FROM-COMPARATOR" is printed ZERO times, never once, anywhere in the
;;      output — the refusal fires before `sorted.sort_by` ever calls the comparator.
;;
;; Not a permanent fixture — delete once this stone's floor/probe evidence is recorded.

(:wat::core::defn :user::pure-surface [] -> :wat::core::nil
  (:wat::core::do
    ;; sort/1 — default `<` comparator (wat/core.wat's own inline fn wrapping `<`).
    (:wat::kernel::println
      (:wat::core::sort (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))
    ;; sort/2 — user-supplied comparator (descending), still pure ∧ deterministic.
    (:wat::kernel::println
      (:wat::core::sort
        (:wat::core::fn [a <- :wat::core::i64
                         b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::> a b))
        (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))
    ;; sort-by — pure key fn (negation), 2-ary — the accessor-keyfn SHAPE
    ;; `wat/query/mem.wat`'s live callers use, minus the record.
    (:wat::kernel::println
      (:wat::core::sort-by
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::- 0 x))
        (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))))

(:wat::core::defn :user::effectful [] -> :wat::core::nil
  (:wat::core::let
    [cmp (:wat::core::fn [a <- :wat::core::i64
                          b <- :wat::core::i64] -> :wat::core::bool
           (:wat::core::do
             (:wat::kernel::println "SIDE-EFFECT-FROM-COMPARATOR")
             (:wat::core::< a b)))]
    (:wat::kernel::println
      (:wat::core::sort$native cmp (:wat::core::Vector :- [:wat::core::i64] 3 1 2)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::pure-surface)
    (:user::effectful)))
