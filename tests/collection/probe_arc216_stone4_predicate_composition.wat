;; tests/collection/probe_arc216_stone4_predicate_composition.wat — co-located fixture.
;; Arc 216 Stone 4 — Atomizable predicate composite verification (positive cases).

;; Probe 1: (HashMap :- [keyword (Vector :- [i64])]) round-trip length = 2
(:wat::core::defn :t::probe1-hashmap-of-vector [] -> :wat::core::i64
  (:wat::core::let
    [inner1  (:wat::core::Vector :wat::core::i64 10 20 30)
     inner2  (:wat::core::Vector :wat::core::i64 40 50)
     m       (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :a inner1 :b inner2)
     h       (:wat::holon::to-holon m)
     back    (:wat::holon::from-holon h)]
    (:wat::hashmap::length back)))

;; Probe 2: (Vector :- [(HashSet :- [i64])]) round-trip length = 2
(:wat::core::defn :t::probe2-vector-of-hashset [] -> :wat::core::i64
  (:wat::core::let
    [set1    (:wat::core::HashSet :wat::core::i64 1 2 3)
     set2    (:wat::core::HashSet :wat::core::i64 4 5)
     outer   (:wat::core::Vector :wat::type::Infer set1 set2)
     h       (:wat::holon::to-holon outer)
     back    (:wat::holon::from-holon h)]
    (:wat::core::Vector/length back)))

;; Probe 3: (HashSet :- [(Vector :- [i64])]) round-trip length = 2
(:wat::core::defn :t::probe3-hashset-of-vector [] -> :wat::core::i64
  (:wat::core::let
    [v1     (:wat::core::Vector :wat::core::i64 1 2)
     v2     (:wat::core::Vector :wat::core::i64 3 4)
     outer  (:wat::core::HashSet :wat::type::Infer v1 v2)
     h      (:wat::holon::to-holon outer)
     back   (:wat::holon::from-holon h)]
    (:wat::core::HashSet/length back)))

;; Probe 4: (HashMap :- [keyword (Vector :- [(HashSet :- [i64])])]) round-trip length = 1
(:wat::core::defn :t::probe4-triple-nested [] -> :wat::core::i64
  (:wat::core::let
    [set1    (:wat::core::HashSet :wat::core::i64 1 2)
     set2    (:wat::core::HashSet :wat::core::i64 3)
     vec     (:wat::core::Vector :wat::type::Infer set1 set2)
     m       (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data vec)
     h       (:wat::holon::to-holon m)
     back    (:wat::holon::from-holon h)]
    (:wat::hashmap::length back)))
