;; probe_arc256_generic_defclause_c05.wat — parametric container clause (Vector T).

(:wat::core::defclause :user::len-of ([v <- (wat.type/Vector :- [T])] -> wat.type/i64 0))
(:wat::core::defn :user::probe [] -> wat.type/i64
  (:user::len-of (wat.type/Vector :- [wat.type/i64] 1 2 3)))
