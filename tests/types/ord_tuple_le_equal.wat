;; ord_tuple_le_equal.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<=
    (:wat::core::Tuple 1 2 3)
    (:wat::core::Tuple 1 2 3)))
