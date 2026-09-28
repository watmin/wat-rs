(:wat::core::defn :fix::compound-map
  []
  -> (wat.type/HashMap :- [wat.type/keyword wat.type/i64])
  {:a (:wat::i64::+ 1 2) :b 42})
