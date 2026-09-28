(:wat::core::defn :fix::ctk [x <- wat.type/i64] -> (:wat::core::Option :- [wat.type/i64])
  (:wat::hashmap::get (:wat::hashmap::assoc (wat.type/HashMap :- [wat.type/keyword wat.type/i64]) :k x) :k))
