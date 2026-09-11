;; VARIANT parametric (:has d) -> i64  CORRECT  accept
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::fb [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::i64 (:has d))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
