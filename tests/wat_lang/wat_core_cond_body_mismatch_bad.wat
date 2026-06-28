;; Negative fixture: cond body arm type mismatches declared return → TypeMismatch.
;; Used by test: cond_refuses_mismatched_body_type

(:wat::core::defn :t::probe [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 1) 42)
    (:else "default")))
