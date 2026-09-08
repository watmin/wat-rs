;; ⛔ THE TRAP — :T is a type PARAMETER, not a registered type. It must stay accepted.
(:wat::core::defn :user::id :- [T] [x <- :T] -> :T x)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:user::id 7)))
