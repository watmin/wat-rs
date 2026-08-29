;; NEGATIVE fixture — must fail to type-check (assoc on a Vector is rejected).
(:wat::core::defn :p::f [] -> :wat::core::i64
  (:wat::core::assoc (:wat::core::Vector :- [:wat::core::i64] 1 2 3) 0 99))

