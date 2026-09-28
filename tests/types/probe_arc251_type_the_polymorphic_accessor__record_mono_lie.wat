;; RECORD monomorphic (:x c) -> String  a lie  REFUSE
(:wat::core::defrecord :u::Plain [x <- wat.type/i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> wat.type/String (:x c))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
