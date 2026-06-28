;; Negative fixture: odd-count vector bindings [x] → MalformedForm.
;; Used by test: odd_count_vector_errors (first case)

(:wat::core::defn :t::compute [] -> :wat::core::i64
  (:wat::core::let [x] 1))
