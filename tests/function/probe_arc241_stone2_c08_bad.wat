;; tests/function/probe_arc241_stone2_c08_bad.wat — NEGATIVE contract 8: incomplete triple.
;; [x <-] — name then arrow but no type slot. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::i64
  ((:wat::core::fn [x <-] -> :wat::core::i64 42)))
