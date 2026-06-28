;; Negative fixture: matches? with zero arguments → check error.
;; Used by test: rejects_arity_zero

(:wat::core::defn :t::probe [] -> :wat::core::bool
  (:wat::form::matches?))
