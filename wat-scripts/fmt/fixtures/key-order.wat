(:wat::core::defn :fix::key-order
  []
  -> (wat.type/Vector :- [wat.type/String])
  (wat.type/Vector :- [wat.type/String]
    (:wat::core::format "{a} {b}" :b 5 :a "x")
    (:wat::core::format "{a} {b}" :a "x" :b 5)))
