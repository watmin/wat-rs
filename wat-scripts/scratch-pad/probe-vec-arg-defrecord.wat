;; Probe: does a bare type keyword inside a defrecord form, when that defrecord form sits
;; NESTED INSIDE A VECTOR that is itself a macro argument, trip Doctrine 1's
;; "type keyword used as a value" check — vs. the SAME defrecord form as a DIRECT (non-vector)
;; macro argument (the STOP-1 echo-defsvc exemplar's proven shape)?

(:wat::core::defmacro :probe::direct-arg
  [def-form <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::do ~def-form))

(:wat::core::defmacro :probe::vec-arg
  [defs-vec <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [children (:wat::core::ast->children defs-vec)]
    `(:wat::core::do ~@children)))

(:probe::direct-arg (:wat::core::defrecord :probe::A [x <- :wat::core::i64]))
(:probe::vec-arg [(:wat::core::defrecord :probe::B [y <- :wat::core::i64])])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "loaded ok"))
