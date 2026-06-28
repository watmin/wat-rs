;; tests/function/probe_arc237_stone2_p12_bad.wat — NEGATIVE probe 12: literal-pattern arg.
;; Per arc 159/169/234 binding contract: clause args MUST be [name <- :Type].
;; Literal patterns ([0 <- :i64]) are not valid. startup MUST fail.

(:wat::core::defclause :my::bad-pattern
  ([0 <- :wat::core::i64] -> :wat::core::i64 1))
