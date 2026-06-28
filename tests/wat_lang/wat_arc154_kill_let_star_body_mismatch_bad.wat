;; Negative fixture: let body returns i64 but declared -> nil → ReturnTypeMismatch.
;; Used by test: let_body_type_mismatch_surfaces

(:wat::core::defn :t::main [] -> :wat::core::nil
  (:wat::core::let [a 5] a))
