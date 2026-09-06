(:wat::core::defn :fix::key-order
  []
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::Vector :- [:wat::core::String]
    (:wat::core::format "{a} {b}" :b 5 :a "x")
    (:wat::core::format "{a} {b}" :a "x" :b 5)))
