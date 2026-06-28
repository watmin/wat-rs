;; tests/function/defn_bad_type.wat — NEGATIVE: body type mismatch (T8).
;; Defn declares -> :nil but body returns i64. startup MUST fail with ReturnTypeMismatch.

(:wat::core::defn :user::bad
  [] -> :wat::core::nil
  42)
