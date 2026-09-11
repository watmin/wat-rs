;; Demo.Has -> Demo.Has slot ACCEPTED
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::takes-has [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::takes-has (:u::Demo.Has {:has 1})))
