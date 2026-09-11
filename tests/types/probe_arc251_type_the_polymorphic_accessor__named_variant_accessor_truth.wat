;; CONTROL — named VARIANT accessor truth still accepted
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::fa [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::i64
  (:u::Demo.Has/has d))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
