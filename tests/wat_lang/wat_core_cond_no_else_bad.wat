;; Negative fixture: cond without :else → check error.
;; Used by test: cond_refuses_missing_else

(:wat::core::defn :t::probe [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 1) "first")
    ((:wat::core::= 2 2) "second")))
