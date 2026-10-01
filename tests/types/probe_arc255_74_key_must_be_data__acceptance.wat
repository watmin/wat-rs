(:wat::core::defrecord :probe::Point [x <- wat.type/i64  y <- wat.type/i64])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [s-i64    (wat.type/HashSet :- [wat.type/i64] 1 2 3)
     s-str    (wat.type/HashSet :- [wat.type/String] "a" "b")
     s-kw     (wat.type/HashSet :- [wat.type/keyword] :a :b)
     s-rec    (wat.type/HashSet :- [:probe::Point] (:probe::Point :x 1 :y 2))
     s-vec    (wat.type/HashSet :- [(wat.type/Vector :- [wat.type/i64])] (wat.type/Vector :- [wat.type/i64] 1 2))
     s-tup    (wat.type/HashSet :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String])] (wat.type/Tuple :- [wat.type/i64 wat.type/String] 1 "x"))
     m-i64    (wat.type/HashMap :- [wat.type/i64 wat.type/String] 1 "one")
     set-lit  #{1 2 3}
     map-lit  {:a 1 :b 2}]
    (:wat::kernel::println "acceptance: ok")))
