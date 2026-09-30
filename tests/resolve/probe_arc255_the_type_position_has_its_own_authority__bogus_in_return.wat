;; RED — a bogus type name in RETURN position.
(:wat::core::defn :user::r [] -> (wat.type/Vector :- [wat.type/Bogus]) (wat.type/Vector :- [wat.type/i64]))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "x"))
