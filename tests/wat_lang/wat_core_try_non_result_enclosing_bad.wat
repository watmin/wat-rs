;; Negative fixture: try inside non-Result-returning function → MalformedForm.
(:wat::core::defn :t::probe [] -> :wat::core::i64
  (:wat::core::Result/try (:wat::core::Ok 42)))
