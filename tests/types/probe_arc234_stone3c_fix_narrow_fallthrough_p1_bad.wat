;; Negative fixture: (:bogus x) where x is i64 must fail at check time.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [x 42] (:bogus x)))
