(:wat::core::defn :fix::assoc-pair
  [x <- :wat::core::i64]
  -> (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
  (:wat::hashmap::assoc (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) :k x))
