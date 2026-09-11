;; RECORD monomorphic (:x c) -> i64  CORRECT  accept
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> :wat::core::i64 (:x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
