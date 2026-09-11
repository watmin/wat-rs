;; SUBJECT — parametric struct, {:keys} must instantiate X to i64.
(:wat::core::defstruct :u::CellS :- [X] [x <- :X])
(:wat::core::defn :u::fs [c <- (:u::CellS :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let [{:keys [x]} c] x))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
