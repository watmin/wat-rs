;; Stone 255.51 — a bounded type parameter is a member of its bound.
(wat.core/defsurface u/Mark :nature wat.type/Record :features [])
(wat.core/defrecord u/In  [n :- wat.type/i64])
(wat.core/defrecord u/Out [n :- wat.type/i64])
(wat.core/extend-type u/In u/Mark)
(wat.core/defn u/takes-mark [m :- u/Mark] :- wat.type/i64 1)
(wat.core/defn u/pass :- [[T :< u/Mark]] [x :- T] :- wat.type/i64
  (u/takes-mark x))
(wat.core/defn user/admitted [] :- wat.type/i64
  (u/pass (u/In :n 1)))
(wat.core/defn user/anon [] :- wat.type/i64
  (wat.core/let
    [f (wat.core/fn :- [[T :< u/Mark]] [x :- T] :- wat.type/i64
         (u/takes-mark x))]
    (f (u/In :n 1))))
