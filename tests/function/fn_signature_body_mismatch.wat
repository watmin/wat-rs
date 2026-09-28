;; tests/function/fn_signature_body_mismatch.wat — NEGATIVE: fn body type mismatch.
;; fn declared -> :nil but body is :i64 param x. startup MUST fail.

(:wat::core::defn :user::main [] -> wat.type/nil ((:wat::core::fn [x <- wat.type/i64] -> wat.type/nil x) 7))
