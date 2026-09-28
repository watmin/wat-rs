;; RECORD parametric (:x c) -> i64  CORRECT  accept  (bug ②)
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::a [c <- (:u::Cell :- [wat.type/i64])] -> wat.type/i64 (:x c))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
