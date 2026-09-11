;; CONTROL — monomorphic record {:keys} was never the defect.
(:wat::core::defrecord :u::Cell [x <- :wat::core::i64])
(:wat::core::defn :u::f [c <- :u::Cell] -> :wat::core::i64
  (:wat::core::let [{:keys [x]} c] x))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
