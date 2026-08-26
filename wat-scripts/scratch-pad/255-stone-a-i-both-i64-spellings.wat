;; wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat — arc 255 Stone A-i / Stone C
;; acceptance row 3, STONE C REWRITE: the surviving spelling works.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-i-the-i64-home.md
;;         docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-C-the-numerics-retirement.md
;;
;; Through Stone B, the 17 i64 ops lived under BOTH `:wat::i64::*` (new) and
;; `:wat::core::i64::*` (old) — this file proved BOTH ran, 34 assertions, one
;; per op per spelling. Stone C retires the old spelling: `:wat::core::i64::*`
;; is now a CHECK-TIME error naming its replacement (see
;; `docs/.../BRIEF-STONE-C-the-numerics-retirement.md`'s acceptance row 1),
;; so this file no longer asserts the old spelling — it would not even
;; TYPE-CHECK. It stays the live proof the surviving spelling works, one
;; assertion per op, 17 total. `to-bigint` / `to-rational` have no simpler
;; literal for their result type, so each round-trips through the existing
;; `bigint::to-f64` / `rational::to-f64` (untouched, out of this stone's
;; scope) back to a comparable f64.
;;
;; Run:  ./target/release/wat --check ./wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat   # EXIT=0
;;       ./target/release/wat        ./wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat   # EXIT=0 (17/17 assertions pass)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::test::assert-eq (:wat::i64::+ 1 2) 3)
    (:wat::test::assert-eq (:wat::i64::- 5 3) 2)
    (:wat::test::assert-eq (:wat::i64::* 3 4) 12)
    (:wat::test::assert-eq (:wat::i64::/ 6 2) 3)
    ;; mod (floored, sign of divisor)
    (:wat::test::assert-eq (:wat::i64::mod -7 3) 2)
    ;; quot (truncate toward zero)
    (:wat::test::assert-eq (:wat::i64::quot -7 3) -2)
    ;; rem (sign of dividend)
    (:wat::test::assert-eq (:wat::i64::rem -7 3) -1)

    (:wat::test::assert-eq (:wat::i64::< 1 2) true)
    (:wat::test::assert-eq (:wat::i64::<= 2 2) true)
    (:wat::test::assert-eq (:wat::i64::> 3 2) true)
    (:wat::test::assert-eq (:wat::i64::>= 2 2) true)
    (:wat::test::assert-eq (:wat::i64::= 2 2) true)
    (:wat::test::assert-eq (:wat::i64::not= 2 3) true)

    ;; to-bigint (no simpler bigint literal — round-trip to f64 instead)
    (:wat::test::assert-eq (:wat::core::bigint::to-f64 (:wat::i64::to-bigint 5)) 5.0)
    (:wat::test::assert-eq (:wat::i64::to-f64 5) 5.0)
    ;; to-rational (no simpler rational literal — round-trip to f64 instead)
    (:wat::test::assert-eq (:wat::core::rational::to-f64 (:wat::i64::to-rational 5)) 5.0)
    (:wat::test::assert-eq (:wat::i64::to-string 42) "42")))
