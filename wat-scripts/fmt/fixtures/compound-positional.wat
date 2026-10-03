(wat.core/defn fix/cp [x :- wat.type/i64] :- wat.type/i64
  (wat.i64/+ (wat.i64/* x 2) 1))
