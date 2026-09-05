(:wat::core::defn :fix::all [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [a (:wat::core::+ x 1) b (:wat::core::+ y 2)] (:wat::core::match a
    (0 b) (_ (:wat::core::+ a b)))))
