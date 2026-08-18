;; probe-clause-tco-deep-defclause.wat — the RED half of the clause-TCO gate.
;; Tail-recursive counting loop whose head is a `defclause`. 200,000 deep.
;; BEFORE the stone: SIGSEGV — eval_tail has no arm for a ClauseSet head, so it
;; falls to `_ => eval_inner` and recurses on the real stack.
;; AFTER: must print 200000, matching its plain-`defn` twin.
(:wat::core::defclause :probe::countdown
  ([n <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::if (:wat::core::= n 0)
      acc
      (:probe::countdown (:wat::core::- n 1) (:wat::core::+ acc 1)))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::countdown 200000 0)))
