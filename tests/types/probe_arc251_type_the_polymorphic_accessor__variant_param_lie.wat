;; VARIANT parametric (:has d) -> String  a lie  REFUSE  (bug ①)
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::fb [d <- (:u::Demo.Has :- [wat.type/i64])] -> wat.type/String (:has d))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
