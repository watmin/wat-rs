(wat.core/defn my/inc [x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ x 1))
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/= (wat.core/-> 3 my/inc) 4))
