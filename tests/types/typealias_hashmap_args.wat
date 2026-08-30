;; typealias_hashmap_args.wat — per-type-arg aliases work at HashMap ctor.
(:wat::core::typealias :my::Key :wat::core::String)
(:wat::core::typealias :my::Val :wat::core::i64)
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [row (:wat::core::HashMap :- [:my::Key :my::Val] "a" 1 "b" 2)
     got (:wat::core::get row "b")]
    (:wat::core::match got 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))
