;; ord_tuple_recursion_deep.wat — Tuple containing Tuple
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<
    (:wat::core::Tuple 1 (:wat::core::Tuple 2 3))
    (:wat::core::Tuple 1 (:wat::core::Tuple 2 4))))
