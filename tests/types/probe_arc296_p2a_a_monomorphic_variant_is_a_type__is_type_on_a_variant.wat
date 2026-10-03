(wat.core/defenum usr/Colour wat.enum/Pure
  :Red  [shade :- wat.type/i64]
  :Blue [shade :- wat.type/i64])
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (wat.runtime/is-type? usr/Colour.Red)))
