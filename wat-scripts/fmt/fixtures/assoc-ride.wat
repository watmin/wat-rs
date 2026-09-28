(:wat::core::defn :fix::assoc-ride
  [m <- (wat.type/HashMap :- [wat.type/i64 wat.type/i64])
   b <- wat.type/i64]
  -> (wat.type/HashMap :- [wat.type/i64 wat.type/i64])
  (:wat::hashmap::assoc m (:wat::i64::+ b 1) (:wat::i64::* b 2)))
