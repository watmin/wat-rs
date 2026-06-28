;; tests/function/probe_check_scoped_param_resolution_handwritten.wat
;; NEGATIVE fixture — startup MUST fail (ReturnTypeMismatch).
;; A hand-written defclause returning its :i64 param as :bool — the
;; mismatch must always be caught by the check pass.

(:wat::core::defclause :test::bad-ret-direct
  ([x <- :wat::core::i64] -> :wat::core::bool x))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
