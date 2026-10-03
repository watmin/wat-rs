(wat.core/defmacro my/probe
  [body :- wat.type/AST]
  :- wat.type/AST
  `(wat.core/do
     (wat.core/defn my/helper [] :- wat.type/i64 42)
     ~body))

(my/probe (wat.core/defn my/main [] :- wat.type/i64 (my/helper)))
