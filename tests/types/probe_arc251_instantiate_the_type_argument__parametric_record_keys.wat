;; SUBJECT — parametric record, {:keys} must instantiate X to i64.
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::f [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let [{:keys [x]} c] x))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
