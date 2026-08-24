;; ord_vec_recursion_deep.wat — (Vec :- [(Vec :- [i64])]): [[1,2],[3,4]] < [[1,2],[3,5]]
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<
    (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::Vector :wat::core::i64 1 2)
      (:wat::core::Vector :wat::core::i64 3 4))
    (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::Vector :wat::core::i64 1 2)
      (:wat::core::Vector :wat::core::i64 3 5))))
