;; ord_tuple_gt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>
    (:wat::core::Tuple 5 "z")
    (:wat::core::Tuple 5 "a")))
