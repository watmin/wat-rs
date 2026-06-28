;; tests/function/probe_arc237_stone3_p08_bad.wat — NEGATIVE probe 8: :ensure fn wrong arity.
;; :ensure :fn must be 1-arity; 2-arity must reject at type-check. startup MUST fail.

(:wat::core::defclause :my::bad
  ([x <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
              (:wat::core::i64::> a b))
    x))
