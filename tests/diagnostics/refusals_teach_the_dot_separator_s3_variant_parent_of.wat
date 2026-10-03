(wat.core/defenum u/E wat.enum/Pure :A [x :- wat.type/i64] :B)
(wat.core/defn u/f [] :- (wat.core/Option :- [wat.type/keyword])
  (wat.runtime/variant-parent-of u/E.A))
