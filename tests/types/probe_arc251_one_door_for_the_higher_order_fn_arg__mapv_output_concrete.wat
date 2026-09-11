;; STOP-1 — mapv's U must still bind. A Vector of i64 is accepted, not a leftover fresh var.
(:wat::core::defn :u::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ x 1))
(:wat::core::defn :u::takes-vec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::takes-vec (:wat::core::mapv :u::inc [1 2 3])))
