;; tests/function/probe_arc241_stone3_c05_bad.wat — NEGATIVE contract 5: missing arrow.
;; Slot 1 is = not <- in defclause arg. startup MUST fail.

(:wat::core::defclause :user::bad
  ([x = :wat::core::i64] -> :wat::core::i64 x))
