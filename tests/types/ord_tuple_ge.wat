;; ord_tuple_ge.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>=
    (:wat::core::Tuple 10 "x")
    (:wat::core::Tuple 9 "x")))
