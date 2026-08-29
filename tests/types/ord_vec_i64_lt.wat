;; ord_vec_i64_lt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<
    (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
    (:wat::core::Vector :- [:wat::core::i64] 1 2 4)))
