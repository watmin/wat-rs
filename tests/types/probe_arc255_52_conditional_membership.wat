(wat.core/defsurface u/Mark :nature wat.type/Record :features [])
(wat.core/defsurface u/Any :nature wat.type/Record :features [])
(wat.core/defrecord u/In  [n :- wat.type/i64])
(wat.core/defrecord u/Out [n :- wat.type/i64])
(wat.core/extend-type u/In u/Mark)
(wat.core/extend-type :- [[T :< u/Mark]] (wat.type/Vector :- [T]) u/Mark)
(wat.core/extend-type :- [T] (wat.type/Vector :- [T]) u/Any)
(wat.core/defn u/take [m :- u/Mark] :- wat.type/i64 1)
(wat.core/defn u/take-any [m :- u/Any] :- wat.type/i64 1)
(wat.core/defn u/f :- [[T :< u/Mark]] [x :- T] :- wat.type/i64 1)
(wat.core/defn user/vec-in [] :- wat.type/i64
  (u/take (wat.type/Vector :- [u/In] (u/In :n 1))))
(wat.core/defn user/vec-nested [] :- wat.type/i64
  (u/take (wat.type/Vector :- [(wat.type/Vector :- [u/In])]
              (wat.type/Vector :- [u/In] (u/In :n 1)))))
(wat.core/defn user/f-in [] :- wat.type/i64
  (u/f (wat.type/Vector :- [u/In] (u/In :n 1))))
(wat.core/defn user/any-out [] :- wat.type/i64
  (u/take-any (wat.type/Vector :- [u/Out] (u/Out :n 1))))
