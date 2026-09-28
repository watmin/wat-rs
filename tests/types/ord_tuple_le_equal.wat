;; ord_tuple_le_equal.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<=
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 1 2 3)
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 1 2 3)))
