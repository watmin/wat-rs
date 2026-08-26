;; probe-clause-tco-guard-selects.wat — `:guard` runs BEFORE the body, so it is part
;; of clause SELECTION and the stone must leave it untouched. Guard-dispatched
;; factorial: the first arm's guard is false for n>0, the second selects. 5! = 120.
(:wat::core::defclause :probe::factorial
  ([n <- :wat::core::i64] :guard (:wat::core::= n 0) -> :wat::core::i64 1)
  ([n <- :wat::core::i64] :guard (:wat::i64::> n 0) -> :wat::core::i64
    (:wat::i64::* n (:probe::factorial (:wat::i64::- n 1)))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::factorial 5)))
