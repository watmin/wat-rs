;; Demo.Has -> Demo slot ACCEPTED
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::takes-demo [d <- (:u::Demo :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::takes-demo (:u::Demo.Has {:has 1})))
