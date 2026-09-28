(:wat::core::defn :fix::let-complex
  [xs <- (wat.type/Vector :- [wat.type/i64])]
  -> wat.type/i64
  (:wat::core::let [mapped (:wat::core::foldl (:wat::core::fn [acc <- wat.type/i64 x <- wat.type/i64] -> wat.type/i64
                                               (:wat::i64::+ acc x))
                                             0 xs)
                   j (:wat::i64::+ 40 2)]
    j))
