(wat.core/defn fix/get-odd
  [x :- wat.type/i64]
  :- (wat.core/Option :- [wat.type/i64])
  (wat.core/get (wat.type/HashMap :- [wat.type/keyword wat.type/i64]) :k))
