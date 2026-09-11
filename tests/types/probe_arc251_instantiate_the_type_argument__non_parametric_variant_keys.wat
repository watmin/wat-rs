;; CONTROL — monomorphic variant {:keys} was never the defect.
(:wat::core::defenum :u::Demo :wat::enum::Pure
  :Has    [has <- :wat::core::i64]
  :HasNot [])
(:wat::core::defn :u::fv [d <- :u::Demo.Has] -> :wat::core::i64
  (:wat::core::let [{:keys [has]} d] has))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
