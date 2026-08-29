;; tests/collection/probe_arc216_stone1_hashset_roundtrip.wat — co-located fixture.
;; Arc 216 Stone 1 — (HashSet :- [T]) round-trip through HolonAST::Bundle.

;; p1: forward round-trip via to-holon + from-holon — length 3
(:wat::core::defn :t::p1-forward-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{1 2 3})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p2a: round-trip length 3
(:wat::core::defn :t::p2a-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{1 2 3})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p2b: round-trip contains 2
(:wat::core::defn :t::p2b-rt-contains [] -> :wat::core::bool
  (:wat::core::let
    [h (:wat::holon::to-holon #{1 2 3})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::contains? s 2)))

;; p3: empty set round-trip length 0
(:wat::core::defn :t::p3-empty-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p4a: single element round-trip length 1
(:wat::core::defn :t::p4a-single-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{42})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p4b: single element round-trip contains 42
(:wat::core::defn :t::p4b-single-rt-contains [] -> :wat::core::bool
  (:wat::core::let
    [h (:wat::holon::to-holon #{42})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::contains? s 42)))

;; p5a: (HashSet :- [i64]) round-trip contains 20
(:wat::core::defn :t::p5a-i64-rt-contains [] -> :wat::core::bool
  (:wat::core::let
    [h (:wat::holon::to-holon #{10 20 30})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::contains? s 20)))

;; p5b: (HashSet :- [String]) round-trip length 3
(:wat::core::defn :t::p5b-str-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon (:wat::core::HashSet :- [:wat::core::String] "a" "b" "c"))
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p5c: (HashSet :- [bool]) round-trip length 2
(:wat::core::defn :t::p5c-bool-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon (:wat::core::HashSet :- [:wat::core::bool] true false))
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p6: deduplicated set round-trip length 3
(:wat::core::defn :t::p6-dedupe-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{1 1 2 2 3})
     s (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p7a: nested set round-trip outer length 2
(:wat::core::defn :t::p7a-nested-rt-outer-len [] -> :wat::core::i64
  (:wat::core::let
    [inner1 (:wat::core::HashSet :- [:wat::core::i64] 1 2)
     inner2 (:wat::core::HashSet :- [:wat::core::i64] 3)
     outer  (:wat::core::HashSet :- [:wat::type::Infer] inner1 inner2)
     h      (:wat::holon::to-holon outer)
     s      (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p7b: nested set arc 228 re-verify outer length 2
(:wat::core::defn :t::p7b-nested-rt-arc228 [] -> :wat::core::i64
  (:wat::core::let
    [inner1 (:wat::core::HashSet :- [:wat::core::i64] 1 2)
     inner2 (:wat::core::HashSet :- [:wat::core::i64] 3)
     outer  (:wat::core::HashSet :- [:wat::type::Infer] inner1 inner2)
     h      (:wat::holon::to-holon outer)
     s      (:wat::holon::from-holon h)]
    (:wat::hashset::length s)))

;; p8a: atomizable passes — returns 1
(:wat::core::defn :t::p8a-atomizable-passes [] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::holon::to-holon #{1 2 3})]
    1))

;; p8b: nested atomizable passes — returns 1
(:wat::core::defn :t::p8b-nested-atomizable [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashSet :- [:wat::core::i64] 1 2)
     outer (:wat::core::HashSet :- [:wat::type::Infer] inner)
     h     (:wat::holon::to-holon outer)]
    1))
