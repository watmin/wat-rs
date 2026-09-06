(:wat::core::defn :fix::ctk [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::hashmap::get (:wat::hashmap::assoc (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) :k x) :k))
