;; tests/function/stone18a_e02.wat — NEGATIVE fixture: fn-form body type mismatch.
;; E02: declared return :i64 but body infers :String.

(:wat::core::defn :test::bad [] -> wat.type/nil
  ((:wat::core::fn [] -> wat.type/i64 "a string")))

