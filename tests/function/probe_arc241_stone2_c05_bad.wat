;; tests/function/probe_arc241_stone2_c05_bad.wat — NEGATIVE contract 5: name not symbol.
;; Slot 0 of triple is :kw (keyword), not a Symbol. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::i64
  ((:wat::core::fn [:kw <- :wat::core::i64] -> :wat::core::i64 42)))
