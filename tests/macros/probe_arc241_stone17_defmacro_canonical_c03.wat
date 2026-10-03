(wat.core/defmacro test/variadic-wrap
  [& items :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  `(wat.type/Vector :- [wat.type/i64] ~@items))

(wat.core/defn test/three [] :- (wat.type/Vector :- [wat.type/i64])
  (test/variadic-wrap 1 2 3))
