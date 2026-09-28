(:wat::core::defn :fix::assoc-pair
  [x <- wat.type/i64]
  -> (wat.type/HashMap :- [wat.type/keyword wat.type/i64])
  (:wat::hashmap::assoc (wat.type/HashMap :- [wat.type/keyword wat.type/i64]) :k x))
