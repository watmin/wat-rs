;; tests/function/probe_arc237_stone3_p13_bad.wat — NEGATIVE probe 13: keyword order violation.
;; Order fixed: args → :guard? → :ensure? → body; :ensure BEFORE :guard is illegal. startup MUST fail.

(:wat::core::defclause :my::bad
  ([x <- :wat::core::i64]
    :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
              (:wat::core::i64::> result 0))
    :guard (:wat::core::i64::> x 0)
    -> :wat::core::i64 x))
