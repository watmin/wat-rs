;; RED — a bogus type name in a defstruct FIELD annotation.
(:wat::core::defstruct :user::S [f <- (wat.type/Vector :- [wat.type/Bogus])])
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
