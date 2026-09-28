;; a leading comment above the form
(:wat::core::defn :fix::cgo
  [x <- wat.type/i64]
  -> wat.type/i64
  ;; about the body
  (:wat::i64::+ x 1))  ;; C trailing on the body

(:wat::core::defn :fix::cgo2
  []
  -> wat.type/nil
  nil)
