;; wat-tests/core/arith-differential-gen.wat — i64 × bigint, GENERATIVELY.
;;
;; Two independent implementations of integer arithmetic. The law compares them
;; to each other, so nothing here is an oracle anyone invented. The measured
;; sketch is `wat-scripts/scratch-pad/probe-i64-bigint-differential.wat`
;; (`Checked [1681 0]` for `+` and `*` over `ints -20 21` × itself).
;;
;; ⚠ THE CONSTRAINT THAT DEFINES THIS FILE. `i64::+` RAISES `IntegerOverflow` at
;; the i64 boundary; there is no checked variant. A generator that crosses it
;; does not report a violation — it kills the run. Every generator here stays
;; well inside i64 (`ints -20 21` → −20..20, product 41×41 = 1681). The boundary
;; is not this file's business. (Temporary: when totality lands, the boundary
;; becomes generatable.)
;;
;; ⚠ SCOPE, stated rather than implied. Lanes: `+`, `-`, `*`, and injectivity of
;; `i64::to-bigint`. NOT `/`: `bigint::/` returns a `:wat::core::rational` when
;; the quotient is not exact (measured: `5N / 2N` → `5/2`, while `i64::/ 5 2`
;; truncates to `2`). That is a type-level disagreement, not an overflow. NOT
;; `quot` / `rem` / `mod`: bigint has no such verbs (`src/check.rs` registers
;; `bigint::{+,-,*,/}` only). NOT a zero divisor: `i64::/ x 0` raises
;; `DivisionByZero` (measured, one direct call). Absence is a boundary.

;; ── the shared assertion ─────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::core::arith-gen::holds :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::bool]] -> :wat::core::nil
  (:wat::core::match (:wat::gen::check g prop)
    ((:wat::gen::CheckOutcome::Checked pts v _first)
      (:wat::core::let [_ (:wat::test::assert-eq pts (:wat::gen::Gen/card g))]
        (:wat::test::assert-eq v 0)))
    (:wat::gen::CheckOutcome::EmptySpace
      (:wat::test::assert-true false))))

(:wat::core::defrecord :wat-tests::core::arith-gen::Pair
  [a <- :wat::core::i64
   b <- :wat::core::i64])

(:wat::core::defn :wat-tests::core::arith-gen::gen-pair []
  -> (:wat::gen::Gen :- [:wat-tests::core::arith-gen::Pair])
  (:wat::gen::record :wat-tests::core::arith-gen::Pair
    (:wat::gen::ints -20 21)
    (:wat::gen::ints -20 21)))

;; ── the laws ─────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::core::arith-gen::law-add
  [p <- :wat-tests::core::arith-gen::Pair] -> :wat::core::bool
  (:wat::core::let [a (:wat-tests::core::arith-gen::Pair/a p)
                    b (:wat-tests::core::arith-gen::Pair/b p)]
    (:wat::core::= (:wat::core::i64::to-bigint (:wat::core::i64::+ a b))
                   (:wat::core::bigint::+ (:wat::core::i64::to-bigint a)
                                          (:wat::core::i64::to-bigint b)))))

(:wat::core::defn :wat-tests::core::arith-gen::law-sub
  [p <- :wat-tests::core::arith-gen::Pair] -> :wat::core::bool
  (:wat::core::let [a (:wat-tests::core::arith-gen::Pair/a p)
                    b (:wat-tests::core::arith-gen::Pair/b p)]
    (:wat::core::= (:wat::core::i64::to-bigint (:wat::core::i64::- a b))
                   (:wat::core::bigint::- (:wat::core::i64::to-bigint a)
                                          (:wat::core::i64::to-bigint b)))))

(:wat::core::defn :wat-tests::core::arith-gen::law-mul
  [p <- :wat-tests::core::arith-gen::Pair] -> :wat::core::bool
  (:wat::core::let [a (:wat-tests::core::arith-gen::Pair/a p)
                    b (:wat-tests::core::arith-gen::Pair/b p)]
    (:wat::core::= (:wat::core::i64::to-bigint (:wat::core::i64::* a b))
                   (:wat::core::bigint::* (:wat::core::i64::to-bigint a)
                                          (:wat::core::i64::to-bigint b)))))

;; Distinct i64s map to distinct bigints. Equivalent: to-bigint is injective
;; over this range. A round-trip via bigint::to-f64 is NOT this law (unsafe
;; past 2^53) and is not here.
(:wat::core::defn :wat-tests::core::arith-gen::law-inject
  [p <- :wat-tests::core::arith-gen::Pair] -> :wat::core::bool
  (:wat::core::let [a (:wat-tests::core::arith-gen::Pair/a p)
                    b (:wat-tests::core::arith-gen::Pair/b p)]
    (:wat::core::= (:wat::core::= (:wat::core::i64::to-bigint a)
                                  (:wat::core::i64::to-bigint b))
                   (:wat::core::= a b))))

;; ── the properties ───────────────────────────────────────────────────────────────────────
;; card = 41 × 41 = 1681, well inside the 5000 ms deftest wall.
(:wat::test::deftest :wat-tests::core::arith-gen::add-agrees
  (:wat-tests::core::arith-gen::holds
    (:wat-tests::core::arith-gen::gen-pair)
    :wat-tests::core::arith-gen::law-add))

(:wat::test::deftest :wat-tests::core::arith-gen::sub-agrees
  (:wat-tests::core::arith-gen::holds
    (:wat-tests::core::arith-gen::gen-pair)
    :wat-tests::core::arith-gen::law-sub))

(:wat::test::deftest :wat-tests::core::arith-gen::mul-agrees
  (:wat-tests::core::arith-gen::holds
    (:wat-tests::core::arith-gen::gen-pair)
    :wat-tests::core::arith-gen::law-mul))

(:wat::test::deftest :wat-tests::core::arith-gen::to-bigint-injective
  (:wat-tests::core::arith-gen::holds
    (:wat-tests::core::arith-gen::gen-pair)
    :wat-tests::core::arith-gen::law-inject))
