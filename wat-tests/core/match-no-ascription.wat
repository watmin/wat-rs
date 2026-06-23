;; wat-tests/core/match-no-ascription.wat — -> :T annihilation sub-strike 2 probe (match).
;;
;; match drops its `-> :T` result ascription: the result type is INFERRED by UNIFYING the
;; arm bodies (the mechanism `if` already uses — arc 258.1's bare `(if cond then else)`).
;;
;; RED at HEAD: `infer_match` MANDATES the arrow — `(match scrut -> :T arm...)`. A bare
;; `(match scrut (pat body) ...)` fails: "`:wat::core::match` now requires `-> :T` …".
;;
;; GREEN after sub-strike 2: bare match; arm bodies unify; the arrow is annihilated.
;;
;; PARKED (ignore) — sub-strike 2 is BLOCKED behind arc 290 (crates healthy first).
;; This is the RED north-star, persisted to DR; remove the ignore when infer_match
;; gains bare-unify (mirroring infer_if) and it flips GREEN.

;; Both arm bodies are :i64 → unify → :i64. No `-> :T`.
(:wat::test::ignore "arc-258 ss2 (match -> :T kill) BLOCKED behind arc-290; bare-match inference not yet built — RED north-star, remove on ss2 land")
(:wat::test::deftest' :wat-tests::core::match-no-ascription-option
  ()
  (:wat::test::assert-eq
    (:wat::core::match (:wat::core::Some 5)
      ((:wat::core::Some v) v)
      (:wat::core::None 0))
    5))
