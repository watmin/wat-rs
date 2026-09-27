;; Excursus 003 step 4 (D4) — G2, a user-raised error is untouched.
;;
;; Integer division by zero written DIRECTLY in user code (not reached through a
;; stdlib alias): the raise span is ALREADY the user's own line, so derivation must
;; add nothing — no extra frame, `:location` unchanged.
(:wat::core::defn :my::test::probe-divzero [] -> :wat::core::i64
  (:wat::i64::/ 4 0))
