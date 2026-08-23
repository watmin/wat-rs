;; typealias_hashmap_std_get.wat — alias over HashMap passes through std get.
(:wat::core::typealias :my::Row (:wat::core::HashMap :- [:wat::core::String :wat::core::i64]))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [row (:wat::core::HashMap :wat::core::String :wat::core::i64 "a" 10 "b" 20)
     got (:wat::core::get row "a")]
    (:wat::core::match got 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))
