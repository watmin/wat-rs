(wat.core/defmacro user/m
  [a :- wat.type/AST]
  :- wat.type/AST
  `(wat.i64/+ ~a 1))
(wat.core/defn user/p [] :- wat.type/i64
  (user/m 1))
