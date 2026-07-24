;; wat-tests/core/match-no-ascription.wat — -> :T annihilation sub-strike 2 probe (match).
;;
;; match drops its `-> :T` result ascription: the result type is INFERRED by UNIFYING the
;; arm bodies (the mechanism `if` already uses — arc 258.1's bare `(if cond then else)`).
;;
;; GREEN as of sub-strike 2 (arc 258.5): bare match; arm bodies unify; the arrow is
;; annihilated. `infer_match` infers the result by unifying the arm bodies (mirroring
;; `infer_if`); a stray `-> :T` in ascription position is now a located compile error.

;; Both arm bodies are :i64 → unify → :i64. No `-> :T`.
(:wat::test::deftest :wat-tests::core::match-no-ascription-option
  
  (:wat::test::assert-eq
    (:wat::core::match (:wat::core::Some 5)
      ((:wat::core::Some v) v)
      (:wat::core::None 0))
    5))
