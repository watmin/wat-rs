(wat.core/defn fix/all [x :- wat.type/i64 y :- wat.type/i64] :- wat.type/i64
  (wat.core/let [a (wat.core/+ x 1) b (wat.core/+ y 2)] (wat.core/match a
    [n n] [_ (wat.core/+ a b)])))
