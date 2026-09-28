;; NEGATIVE fixture — must fail to type-check (assoc on a Vector is rejected).
(:wat::core::defn :p::f [] -> wat.type/i64
  (:wat::core::assoc (wat.type/Vector :- [wat.type/i64] 1 2 3) 0 99))

