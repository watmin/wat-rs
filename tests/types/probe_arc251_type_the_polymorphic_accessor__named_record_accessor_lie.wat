;; CONTROL — named RECORD accessor lie still refused
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> :wat::core::String (:u::Plain/x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
