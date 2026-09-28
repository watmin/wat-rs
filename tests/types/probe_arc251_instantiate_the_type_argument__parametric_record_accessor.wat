;; CONTROL — parametric record's :T/field accessor already instantiated.
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::fr [c <- (:u::Cell :- [wat.type/i64])] -> wat.type/i64
  (:u::Cell/x c))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
