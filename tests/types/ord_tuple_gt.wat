;; ord_tuple_gt.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::>
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 5 "z")
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 5 "a")))
