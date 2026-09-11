;; CONTROL — parametric record's :T/field accessor already instantiated.
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::fr [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::i64
  (:u::Cell/x c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
