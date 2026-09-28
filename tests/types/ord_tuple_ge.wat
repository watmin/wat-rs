;; ord_tuple_ge.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::>=
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 10 "x")
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 9 "x")))
