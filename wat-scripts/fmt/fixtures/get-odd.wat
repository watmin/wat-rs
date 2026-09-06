(:wat::core::defn :fix::get-odd
  [x <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::hashmap::get (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) :k))
