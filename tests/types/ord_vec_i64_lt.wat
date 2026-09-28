;; ord_vec_i64_lt.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<
    (wat.type/Vector :- [wat.type/i64] 1 2 3)
    (wat.type/Vector :- [wat.type/i64] 1 2 4)))
