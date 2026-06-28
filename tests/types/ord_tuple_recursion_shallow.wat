;; ord_tuple_recursion_shallow.wat — (1,X) < (2,Y): second element never inspected
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<
    (:wat::core::Tuple 1 "anything-here")
    (:wat::core::Tuple 2 "anything-there")))
