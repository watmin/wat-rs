;; probe_arc256_generic_defclause_c05.wat — parametric container clause (Vector T). RED at HEAD.

(:wat::core::defclause :user::len-of ([v <- (:wat::core::Vector :- [T])] -> :wat::core::i64 0))
(:wat::core::defn :user::probe [] -> :wat::core::i64
  (:user::len-of (:wat::core::vector 1 2 3)))
