;; tests/function/probe_arc241_stone2_c06_bad.wat — NEGATIVE contract 6: missing arrow.
;; Slot 1 is = not <-. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::i64
  ((:wat::core::fn [x = :wat::core::i64] -> :wat::core::i64 x)))
