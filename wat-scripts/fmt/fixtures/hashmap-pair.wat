(:wat::core::defn :fix::hashmap-pair
  []
  -> (wat.type/HashMap :- [wat.type/keyword wat.type/String])
  (wat.type/HashMap :- [wat.type/keyword wat.type/String] :some-kw "some-string"))
