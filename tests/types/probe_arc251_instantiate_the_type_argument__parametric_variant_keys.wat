;; SUBJECT — parametric variant, {:keys} must instantiate T to i64.
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure
  :Has    [has <- :T]
  :HasNot [])
(:wat::core::defn :u::fv [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let [{:keys [has]} d] has))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
