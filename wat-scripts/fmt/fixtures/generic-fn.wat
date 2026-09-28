(:wat::core::defn :fix::generic-fn
  [xs <- (wat.type/Vector :- [wat.type/i64])]
  -> wat.type/i64
  (:wat::core::foldl
    (wat.core/fn :- [T]
      [acc :- wat.core/i64
       x   :- T]
      :- wat.type/i64
      acc)
    0
    xs))
