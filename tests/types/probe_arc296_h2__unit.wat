(wat.core/defenum usr/Shape wat.enum/Pure
  :Circle [r :- wat.type/i64]
  :Dot [])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (usr/Shape.Dot {})))
