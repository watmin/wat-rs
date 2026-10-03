(:wat::core::defn :fix::assoc-pair
  [x <- wat.type/i64]
  -> (wat.type/HashMap :- [wat.type/keyword wat.type/i64])
  (:wat::core::assoc (wat.type/HashMap :- [wat.type/keyword wat.type/i64]) :k x))
