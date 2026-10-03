(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/= (wat.core/-> 5 (wat.i64/- 3)) 2))
