(:wat::core::defn :fix::let-complex
  [xs <- (:wat::core::Vector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [mapped (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                                               (:wat::i64::+ acc x))
                                             0 xs)
                   j (:wat::i64::+ 40 2)]
    j))
