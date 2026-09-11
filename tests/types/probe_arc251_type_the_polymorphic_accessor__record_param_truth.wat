;; RECORD parametric (:x c) -> i64  CORRECT  accept  (bug ②)
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::a [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::i64 (:x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
