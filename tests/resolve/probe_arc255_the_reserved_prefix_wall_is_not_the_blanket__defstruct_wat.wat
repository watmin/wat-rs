;; A USER `defstruct` under `:wat::*`. MUST be refused.
(:wat::core::defstruct :wat::core::SneakyS [f <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
