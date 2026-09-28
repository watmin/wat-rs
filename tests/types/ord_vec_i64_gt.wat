;; ord_vec_i64_gt.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::>
    (wat.type/Vector :- [wat.type/i64] 5)
    (wat.type/Vector :- [wat.type/i64] 1)))
