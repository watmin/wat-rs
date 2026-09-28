;; tests/function/fn_rename_mixed_legacy.wat — NEGATIVE: mixed legacy (lambda + bare :fn(...)) fires BOTH walkers.
(:wat::core::defn :user::main [] -> wat.type/i64
  ((:wat::core::lambda
               ((g :fn(wat::core::i64)->wat::core::i64)
                ->
                wat.type/i64)
               (g 5))
             (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 x)))
