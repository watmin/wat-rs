;; BASELINE: a NON-parametric surface + extend-type + method-call. If this passes, the mechanics
;; work and the <T> param is the gap. If the receiver still fails, my extend-type syntax is wrong.
;; CLAIM: (Holds2/get b) == 42.
(wat.core/defsurface probe/Holds2 :nature wat.type/Struct
  :features [(get [self :- probe/Holds2] :- wat.type/i64)])
(wat.core/defrecord probe/IntBox2 [n :- wat.type/i64])
(wat.core/extend-type probe/IntBox2 probe/Holds2
  (get [self] (probe.IntBox2/n self)))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [b  (probe/IntBox2 42)
     ok (wat.core/ann-form (probe.Holds2/get b) wat.type/i64)]
    (wat.core/do
      (wat.test/assert-eq ok 42)
      (wat.kernel/println "measured"))))
