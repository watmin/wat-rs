;; The measurement rides the head line; each branch takes its own line one level in.
(:wat::core::defn :fix::if-ride
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::> x 0)
    (:wat::i64::+ x 1)
    (:wat::i64::- x 1)))
