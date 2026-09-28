;; tests/function/fn_rename_bare_fn_type.wat — NEGATIVE: bare :fn(...) fires BareLegacyLowercaseFn.
(:wat::core::defn :user::main [] -> wat.type/i64
  ((:wat::core::fn
               [g <- :fn(wat::core::i64)->wat::core::i64]
                ->
                wat.type/i64
               (g 5))
             (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 x)))
