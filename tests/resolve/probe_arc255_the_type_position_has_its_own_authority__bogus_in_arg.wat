;; RED — a bogus type name as a type ARGUMENT under a legitimate head.
(:wat::core::defn :user::g [p <- (wat.type/Tuple :- [wat.type/Bogus])] -> :wat::core::i64 1)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
