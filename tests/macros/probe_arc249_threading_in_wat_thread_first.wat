(wat.core/defmacro test/thread-first
  [acc :- wat.type/AST & steps :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  (wat.core/foldl
    (wat.core/fn [a :- wat.holon/HolonAST step :- wat.holon/HolonAST]
       :- wat.holon/HolonAST
       (wat.core/if (wat.core/List? step) 
         `(~(wat.core/first step) ~a ~@(wat.core/rest step))
         `(~step ~a)))
    acc
    steps))
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/= (test/thread-first 5 (wat.i64/- 3)) 2))
