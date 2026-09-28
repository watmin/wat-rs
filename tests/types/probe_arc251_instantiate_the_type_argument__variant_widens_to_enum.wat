;; CONTROL — Demo.Has -> Demo slot ACCEPTED (subtyping unchanged).
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::takes-demo [d <- (:u::Demo :- [wat.type/i64])] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:u::takes-demo (:u::Demo.Has {:has 1})))
