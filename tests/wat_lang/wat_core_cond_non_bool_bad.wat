;; Negative fixture: cond test arm is non-bool (i64) → check error.
;; Used by test: cond_refuses_non_bool_test

(:wat::core::defn :t::probe [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    (42 "first")
    (:else "none")))
