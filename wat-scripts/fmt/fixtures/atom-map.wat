(wat.core/defn fix/atom-map
  []
  :- (wat.type/HashMap :- [wat.type/keyword wat.type/i64])
  {:a 3 :b 42})
