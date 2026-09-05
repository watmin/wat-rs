(:wat::core::defn :fix::generic-fn
  [xs <- (:wat::core::Vector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn :- [:wat::core::i64]
      [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc x))
    0
    xs))
