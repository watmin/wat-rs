;; AMEND-255.74 D3 — the key door is `:< :wat::core::Equatable` (`wat/class.wat`), not
;; is_atomizable. u8/bigint/rational/Instant/(Option :- [i64])/(PersistentVector :- [i64])/a
;; Pure enum are ALL `:< Equatable` and were refused pre-amendment (is_atomizable never admitted
;; them) while the runtime hashed every one of them — this fixture is the acceptance proof they
;; check AND run clean now.
(:wat::core::defrecord :probe::Point [x <- wat.type/i64  y <- wat.type/i64])
(:wat::core::defenum :probe::Color :wat::enum::Pure :Red :Green :Blue)

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [s-i64    (wat.type/HashSet :- [wat.type/i64] 1 2 3)
     s-f64    (wat.type/HashSet :- [wat.type/f64] 1.5 2.5)
     s-str    (wat.type/HashSet :- [wat.type/String] "a" "b")
     s-kw     (wat.type/HashSet :- [wat.type/keyword] :a :b)
     s-u8     (wat.type/HashSet :- [wat.type/u8] (:wat::core::u8 1) (:wat::core::u8 2))
     s-bigint (wat.type/HashSet :- [wat.type/bigint] (:wat::i64::to-bigint 100))
     s-rat    (wat.type/HashSet :- [wat.type/rational] (:wat::i64::to-rational 3))
     s-inst   (wat.type/HashSet :- [:wat::time::Instant] (:wat::time::now))
     s-opt    (wat.type/HashSet :- [(:wat::core::Option :- [wat.type/i64])] :wat::core::Option.None)
     s-pv     (wat.type/HashSet :- [(wat.type/PersistentVector :- [wat.type/i64])] (wat.type/PersistentVector :- [wat.type/i64] 1 2))
     s-enum   (wat.type/HashSet :- [:probe::Color] :probe::Color.Red)
     s-rec    (wat.type/HashSet :- [:probe::Point] (:probe::Point :x 1 :y 2))
     s-vec    (wat.type/HashSet :- [(wat.type/Vector :- [wat.type/i64])] (wat.type/Vector :- [wat.type/i64] 1 2))
     s-tup    (wat.type/HashSet :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String])] (wat.type/Tuple :- [wat.type/i64 wat.type/String] 1 "x"))
     m-i64    (wat.type/HashMap :- [wat.type/i64 wat.type/String] 1 "one")
     m-u8     (wat.type/HashMap :- [wat.type/u8 wat.type/String] (:wat::core::u8 1) "one")
     set-lit  #{1 2 3}
     map-lit  {:a 1 :b 2}]
    (:wat::kernel::println "acceptance: ok")))
