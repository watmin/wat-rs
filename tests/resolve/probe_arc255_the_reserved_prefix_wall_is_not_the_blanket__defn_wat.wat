;; A USER `defn` claiming a name under `:wat::*`. MUST be refused.
(:wat::core::defn :wat::core::sneaky [] -> :wat::core::i64 1)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
