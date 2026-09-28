;; Probe: does a bare type keyword inside a defrecord form, when that defrecord form sits
;; NESTED INSIDE A VECTOR that is itself a macro argument, trip Doctrine 1's
;; "type keyword used as a value" check — vs. the SAME defrecord form as a DIRECT (non-vector)
;; macro argument (the STOP-1 echo-defsvc exemplar's proven shape)?

(:wat::core::defmacro :probe::direct-arg
  [def-form <- wat.type/AST] -> wat.type/AST
  `(:wat::core::do ~def-form))

(:wat::core::defmacro :probe::vec-arg
  [defs-vec <- wat.type/AST] -> wat.type/AST
  (:wat::core::let [children (:wat::core::ast->children defs-vec)]
    `(:wat::core::do ~@children)))

(:probe::direct-arg (:wat::core::defrecord :probe::A [x <- wat.type/i64]))
(:probe::vec-arg [(:wat::core::defrecord :probe::B [y <- wat.type/i64])])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println "loaded ok"))
