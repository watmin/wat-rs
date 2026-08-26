;; wat-scripts/scratch-pad/255-stone-d-both-bigint-rational-spellings.wat — arc 255 Stone D
;; acceptance row 1, PHASE 3 REWRITE: the surviving spelling works.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-D-bigint-and-rational.md
;;
;; Through Phase 2, the 13 bigint/rational ops lived under BOTH `:wat::{bigint,rational}::*`
;; (new) and `:wat::core::{bigint,rational}::*` / `:wat::core::rational/*` (old) — this file
;; proved BOTH ran, 24 assertions (mirrors `255-stone-a-i-both-i64-spellings.wat`'s own Stone C
;; rewrite, one stone over). Phase 3 retires the old spelling: `:wat::core::{bigint,rational}::*`
;; / `:wat::core::rational/*` are now a CHECK-TIME error naming their replacement, so this file
;; no longer asserts the old spelling — it would not even TYPE-CHECK. It stays the live proof
;; the surviving spelling works, 12 assertions (one per op — `to-f64` doubles as its own
;; standalone-cast case AND the round-trip wrapper every arithmetic assertion uses).
;;
;; `a = 3/8`, `b = 1/4` are chosen so EVERY one of rational `+ - * /` on them is non-integer
;; (0.625 / 0.125 / 0.09375 / 1.5) AND exactly representable in f64 (power-of-2 denominators
;; throughout) — `rational::{+,-,*,/}` COLLAPSES an integer-valued result to `:wat::core::bigint`
;; (Stone C2), which would fail `rational::to-f64`'s strict rational-only match arm; these two
;; operands never trigger that. `numerator`/`denominator` reuse `a` (3/8, also never collapses).
;;
;; Run:  ./target/release/wat --check ./wat-scripts/scratch-pad/255-stone-d-both-bigint-rational-spellings.wat   # EXIT=0
;;       ./target/release/wat        ./wat-scripts/scratch-pad/255-stone-d-both-bigint-rational-spellings.wat   # EXIT=0 (12/12 assertions pass)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── bigint (6 ops: + - * / to-f64 to-rational) ─────────────────────────
    (:wat::test::assert-eq (:wat::bigint::to-f64 (:wat::bigint::+ (:wat::i64::to-bigint 1) (:wat::i64::to-bigint 2))) 3.0)
    (:wat::test::assert-eq (:wat::bigint::to-f64 (:wat::bigint::- (:wat::i64::to-bigint 5) (:wat::i64::to-bigint 3))) 2.0)
    (:wat::test::assert-eq (:wat::bigint::to-f64 (:wat::bigint::* (:wat::i64::to-bigint 3) (:wat::i64::to-bigint 4))) 12.0)
    (:wat::test::assert-eq (:wat::bigint::to-f64 (:wat::bigint::/ (:wat::i64::to-bigint 6) (:wat::i64::to-bigint 2))) 3.0)
    (:wat::test::assert-eq (:wat::bigint::to-f64 (:wat::i64::to-bigint 5)) 5.0)
    (:wat::test::assert-eq (:wat::rational::to-f64 (:wat::bigint::to-rational (:wat::i64::to-bigint 5))) 5.0)

    ;; ── rational (7 ops: + - * / to-f64 numerator denominator) ─────────────
    (:wat::core::let
      [a (:wat::rational::/ (:wat::i64::to-rational 3) (:wat::i64::to-rational 8))
       b (:wat::rational::/ (:wat::i64::to-rational 1) (:wat::i64::to-rational 4))]
      (:wat::core::do
        (:wat::test::assert-eq (:wat::rational::to-f64 (:wat::rational::+ a b)) 0.625)
        (:wat::test::assert-eq (:wat::rational::to-f64 (:wat::rational::- a b)) 0.125)
        (:wat::test::assert-eq (:wat::rational::to-f64 (:wat::rational::* a b)) 0.09375)
        (:wat::test::assert-eq (:wat::rational::to-f64 (:wat::rational::/ a b)) 1.5)
        (:wat::test::assert-eq (:wat::rational::numerator a) 3)
        (:wat::test::assert-eq (:wat::rational::denominator a) 8)))))
