(:wat::core::defn :fix::sum [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [y (:wat::core::+ x 1) z (:wat::core::+ x 2)] (:wat::core::+ y z)))
