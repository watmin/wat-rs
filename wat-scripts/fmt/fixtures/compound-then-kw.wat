(wat.core/defn fix/ctk [x :- wat.type/i64] :- (wat.core/Option :- [wat.type/i64])
  (wat.core/get (wat.core/assoc (wat.type/HashMap :- [wat.type/keyword wat.type/i64]) :k x) :k))
