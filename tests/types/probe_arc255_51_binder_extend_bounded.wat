(wat.core/defsurface u/Mark :nature wat.type/Record :features [])
(wat.core/extend-type :- [[T :< u/Mark]]
  (wat.type/Vector :- [T])
  u/Mark)
