;; ord_tuple_lt.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 1 "alpha")
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 2 "alpha")))
