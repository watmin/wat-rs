(:wat::core::defn :fix::compound-map
  []
  -> (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
  {:a (:wat::i64::+ 1 2) :b 42})
