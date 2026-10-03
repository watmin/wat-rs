;; Arc 118.2a — `map` stays LAZY (consumed once by `filter`, no materializer needed); `filter`
;; becomes `filterv` to materialize the final pipeline result for the equality check.
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/= (wat.core/->> [1 2 3]
                   (wat.core/map (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ x 1)))
                   (wat.core/filterv (wat.core/fn [x :- wat.type/i64] :- wat.type/bool (wat.i64/> x 2))))
                 [3 4]))
