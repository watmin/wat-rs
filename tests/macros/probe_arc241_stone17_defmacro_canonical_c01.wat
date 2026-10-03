(wat.core/defmacro test/wrap
  [x :- wat.type/AST]
  :- wat.type/AST
  `(wat.core/Option.Some {:value ~x}))
