;; Stone 255.48 — the rows that must load.
;; An edge admits a record to a featureless surface. A featureful surface
;; still admits a wider record with no edge.

(wat.core/defsurface probe/Mark :nature wat.type/Record :features [])
(wat.core/defrecord probe/Item [n :- wat.type/i64])
(wat.core/extend-type probe/Item probe/Mark)

(wat.core/defn probe/take-mark [x :- probe/Mark] :- wat.type/i64 1)
(wat.core/defn user/admitted [] :- wat.type/i64
  (probe/take-mark (probe/Item :n 1)))

(wat.core/defsurface probe/HasN :nature wat.type/Record
  :features [n :- wat.type/i64])
(wat.core/defrecord probe/Pair [n :- wat.type/i64  extra :- wat.type/i64])
(wat.core/defn probe/take-n [x :- probe/HasN] :- wat.type/i64 1)
(wat.core/defn user/width [] :- wat.type/i64
  (probe/take-n (probe/Pair :n 1 :extra 2)))
