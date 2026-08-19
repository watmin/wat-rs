;; probe-clause-tco-deep-defn.wat — the CONTROL for probe-clause-tco-deep-defclause.
;; Byte-identical body; the ONLY difference is a plain `defn` head instead of a
;; `defclause` head. Completes today and must keep completing. Without this control
;; the RED probe would only show "200k is deep", not "clause heads lack the tail path".
(:wat::core::defn :probe::countdown-defn
  [n <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0)
    acc
    (:probe::countdown-defn (:wat::core::- n 1) (:wat::core::+ acc 1))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::countdown-defn 200000 0)))
