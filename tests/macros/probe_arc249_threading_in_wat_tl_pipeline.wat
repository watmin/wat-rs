(wat.core/defmacro test/thread-last
  [acc :- wat.type/AST & steps :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  (wat.core/foldl
    (wat.core/fn [a :- wat.holon/HolonAST step :- wat.holon/HolonAST]
       :- wat.holon/HolonAST `(~@step ~a))
    acc
    steps))
;; Arc 118.2a — `map` stays LAZY (feeds `filter` once); `filter` becomes `filterv` to
;; materialize the final pipeline result.
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/= (test/thread-last [1 2 3]
                   (wat.core/map (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ x 1)))
                   (wat.core/filterv (wat.core/fn [x :- wat.type/i64] :- wat.type/bool (wat.i64/> x 2))))
                 [3 4]))
