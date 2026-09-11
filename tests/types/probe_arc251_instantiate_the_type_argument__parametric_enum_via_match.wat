;; CONTROL — reaching the field through the enum and a match already instantiated.
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::fm [d <- (:u::Demo :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match d
    [:u::Demo.Has    {:has has} has]
    [:u::Demo.HasNot {}         0]))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
