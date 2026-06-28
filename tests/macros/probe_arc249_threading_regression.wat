(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1)) [1 2 3]) [2 3 4]))
