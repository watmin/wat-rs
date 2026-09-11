;; RECORD parametric (:x c) -> String  a lie  REFUSE
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::a [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::String (:x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
