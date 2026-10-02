;; tests/function/fn_rename_legacy_lambda.wat — NEGATIVE: :wat::core::lambda fires BareLegacyLambda.
(:wat::core::defn :user::main [] -> wat.type/i64
  ((:wat::core::lambda ((x wat.type/i64) -> wat.type/i64)
               x)
             5))
