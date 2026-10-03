(wat.core/defn fix/postbl
  []
  :- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/String wat.type/String])])
  (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/String wat.type/String])]
    (wat.type/Tuple :- [wat.type/String wat.type/String] "short" "xx")
    (wat.type/Tuple :- [wat.type/String wat.type/String] "much-longer" "y")))
