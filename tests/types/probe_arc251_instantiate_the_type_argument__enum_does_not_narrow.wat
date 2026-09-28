;; CONTROL — Demo -> Demo.Has slot REFUSED (widening is one-way).
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::takes-has [d <- (:u::Demo.Has :- [wat.type/i64])] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :u::relay [d <- (:u::Demo :- [wat.type/i64])] -> wat.type/nil
  (:u::takes-has d))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
