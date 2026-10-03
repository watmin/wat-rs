(wat.core/defn fix/sum [xs :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/i64
  (wat.core/foldl (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
    (wat.core/+ acc x)) 0 xs))
