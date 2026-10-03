(wat.core/defmacro my/sum [& nums :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  (wat.core/foldl
    (wat.core/fn [acc :- wat.holon/HolonAST n :- wat.holon/HolonAST]
       :- wat.holon/HolonAST `(wat.i64/+ ~acc ~n))
    `0
    nums))
(wat.core/defn user/compute [] :- wat.type/bool (wat.core/= (my/sum 1 2 3) 6))
