;; tests/function/probe_arc241_stone2_c09_bad.wat — NEGATIVE contract 9: missing ret arrow.
;; -> is missing between args-vector and ret-type. startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] :wat::core::i64 x)))
