;; a leading comment above the form
(:wat::core::defn :fix::cgo
  [x <- :wat::core::i64]
  -> :wat::core::i64
  ;; about the body
  (:wat::i64::+ x 1))  ;; C trailing on the body

(:wat::core::defn :fix::cgo2
  []
  -> :wat::core::nil
  nil)
