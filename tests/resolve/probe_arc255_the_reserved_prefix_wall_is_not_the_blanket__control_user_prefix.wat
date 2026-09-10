;; CONTROL — the SAME six forms under the user's OWN prefix. All legal. If this ever goes red,
;; the wall has widened past its job and is refusing userland its own namespace.
(:wat::core::defn :user::ok-fn [] -> :wat::core::i64 1)
(:wat::core::defmacro :user::ok-macro [] -> :wat::WatAST (:wat::core::quote 1))
(:wat::core::defstruct :user::OkS [f <- :wat::core::i64])
(:wat::core::typealias :user::OkA :wat::core::i64)
(:wat::core::defenum :user::OkE :wat::enum::Pure :V [])
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
