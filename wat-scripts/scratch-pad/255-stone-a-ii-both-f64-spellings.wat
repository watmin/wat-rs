;; wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat — arc 255 Stone A-ii / Stone C
;; acceptance row 3, STONE C REWRITE: the surviving spelling works.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-ii-the-f64-home.md
;;         docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-C-the-numerics-retirement.md
;;
;; Through Stone B, the 19 f64 ops lived under BOTH `:wat::f64::*` (new) and
;; `:wat::core::f64::*` (old) — this file proved BOTH ran, 38 assertions, one
;; per op per spelling. Stone C retires the old spelling: `:wat::core::f64::*`
;; is now a CHECK-TIME error naming its replacement (see
;; `docs/.../BRIEF-STONE-C-the-numerics-retirement.md`'s acceptance row 1),
;; so this file no longer asserts the old spelling — it would not even
;; TYPE-CHECK. It stays the live proof the surviving spelling works, one
;; assertion per op, 17 total.
;;
;; `max-of` / `min-of` are VARIADIC (bare args) — the retired single-Vector
;; calling convention is gone along with the old spelling. Both are
;; exercised here at FOUR arguments (more than two), so the row proves
;; something beyond the binary case.
;;
;; Run:  ./target/release/wat --check ./wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat   # EXIT=0
;;       ./target/release/wat        ./wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat   # EXIT=0 (17/17 assertions pass)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq (:wat::f64::+ 1.0 2.0) 3.0)
    (:wat::test::assert-eq (:wat::f64::- 5.0 3.0) 2.0)
    (:wat::test::assert-eq (:wat::f64::* 3.0 4.0) 12.0)
    (:wat::test::assert-eq (:wat::f64::/ 6.0 2.0) 3.0)
    (:wat::test::assert-eq (:wat::f64::max 1.0 2.0) 2.0)
    (:wat::test::assert-eq (:wat::f64::min 1.0 2.0) 1.0)

    (:wat::test::assert-eq (:wat::f64::< 1.0 2.0) true)
    (:wat::test::assert-eq (:wat::f64::<= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::f64::> 3.0 2.0) true)
    (:wat::test::assert-eq (:wat::f64::>= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::f64::= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::f64::not= 2.0 3.0) true)

    (:wat::test::assert-eq (:wat::f64::abs -3.5) 3.5)
    (:wat::test::assert-eq (:wat::f64::round 1.5 0) 2.0)
    (:wat::test::assert-eq (:wat::f64::to-i64 3.75) (:wat::core::Some 3))
    (:wat::test::assert-eq (:wat::f64::to-string 2.5) "2.5")
    (:wat::test::assert-eq (:wat::f64::clamp 5.0 -1.0 1.0) 1.0)

    ;; max-of / min-of — variadic, 4 args (> 2) so the row proves something
    ;; beyond the binary case.
    (:wat::test::assert-eq (:wat::f64::max-of 1.0 -5.0 4.2 3.0) (:wat::core::Some 4.2))
    (:wat::test::assert-eq (:wat::f64::min-of 1.0 -5.0 4.2 3.0) (:wat::core::Some -5.0))))
