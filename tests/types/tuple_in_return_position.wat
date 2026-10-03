;; tuple_in_return_position.wat — Tuple in function return position type-checks clean.
(wat.core/defn user/make-pair [] :- (wat.type/Tuple :- [wat.type/i64 wat.type/String]) (wat.type/Tuple :- [wat.type/i64 wat.type/String] 42 "hello"))
(wat.core/defn my/invoke [] :- (wat.type/Tuple :- [wat.type/i64 wat.type/String]) (user/make-pair))
