(:wat::core::defn :fix::postbl
  []
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/String wat.type/String])])
  (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/String wat.type/String])]
    (:wat::core::Tuple "short" "xx")
    (:wat::core::Tuple "much-longer" "y")))
