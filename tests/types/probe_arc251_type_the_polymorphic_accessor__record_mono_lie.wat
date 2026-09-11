;; RECORD monomorphic (:x c) -> String  a lie  REFUSE
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> :wat::core::String (:x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
