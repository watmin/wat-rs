;; Demo -> Demo.Has slot REFUSED
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::takes-has [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :u::relay [d <- (:u::Demo :- [:wat::core::i64])] -> :wat::core::nil
  (:u::takes-has d))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
