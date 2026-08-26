(:wat::core::defn :my::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:wat::core::-> 3 :my::inc) 4))
