;; tests/function/probe_arc241_stone3_c06_bad.wat — NEGATIVE contract 6: incomplete triple.
;; [x <-] in defclause arg — no type slot. startup MUST fail.

(:wat::core::defclause :user::bad
  ([x <-] -> :wat::core::i64 42))
