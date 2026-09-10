;; A USER `defmacro` under `:wat::*`. MUST be refused.
(:wat::core::defmacro :wat::core::sneakym [] -> :wat::WatAST (:wat::core::quote 1))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
