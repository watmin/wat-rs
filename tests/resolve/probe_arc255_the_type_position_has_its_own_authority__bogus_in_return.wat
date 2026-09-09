;; RED — a bogus type name in RETURN position.
(:wat::core::defn :user::r [] -> (wat.type/Vector :- [wat.type/Bogus]) (:wat::core::Vector))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
