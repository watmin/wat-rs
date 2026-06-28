;; tests/function/probe_arc241_stone2_c07_bad.wat — NEGATIVE contract 7: type slot not keyword.
;; Slot 2 is a string literal, not a Keyword. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::i64
  ((:wat::core::fn [x <- "i64"] -> :wat::core::i64 42)))
