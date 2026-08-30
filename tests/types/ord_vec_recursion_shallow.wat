;; ord_vec_recursion_shallow.wat — [9,1,1] > [1,99,99]: first element wins
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>
    (:wat::core::Vector :- [:wat::core::i64] 9 1 1)
    (:wat::core::Vector :- [:wat::core::i64] 1 99 99)))
