;; ord_tuple_recursion_deep.wat — Tuple containing Tuple
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<
    (wat.type/Tuple :- [wat.type/i64 (wat.type/Tuple :- [wat.type/i64 wat.type/i64])] 1 (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 2 3))
    (wat.type/Tuple :- [wat.type/i64 (wat.type/Tuple :- [wat.type/i64 wat.type/i64])] 1 (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 2 4))))
