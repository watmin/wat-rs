(wat.core/do
  (wat.core/typealias diag/MyAlias wat.type/i64)
  (wat.core/defn diag/alias-probe [] :- diag/MyAlias 42))
