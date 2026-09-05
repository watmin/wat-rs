(:wat::core::defn :fix::assoc-ride
  [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   b <- :wat::core::i64]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::hashmap::assoc m (:wat::i64::+ b 1) (:wat::i64::* b 2)))
