;; RED — a bogus type name as the HEAD of a `:-` type reference, param position.
(:wat::core::defn :user::f [p <- (wat.type/Bogus :- [wat.type/i64])] -> wat.type/i64 1)
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "x"))
