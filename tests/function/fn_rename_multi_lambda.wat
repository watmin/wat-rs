;; tests/function/fn_rename_multi_lambda.wat — NEGATIVE: multiple :wat::core::lambda sites fire BareLegacyLambda.
(wat.core/defn user/main [] :- wat.type/i64
  ((wat.core/lambda (() :- wat.type/i64)
               (wat.i64/+ 1 2))
             ))
