;; tests/function/probe_arc241_stone3_c04_bad.wat — NEGATIVE contract 4: name not symbol.
;; Slot 0 of defclause arg triple is :kw (keyword). startup MUST fail.

(:wat::core::defclause :user::bad
  ([:kw <- :wat::core::i64] -> :wat::core::i64 42))
