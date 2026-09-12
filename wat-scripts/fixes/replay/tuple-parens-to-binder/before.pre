;; tests/collection/probe_arc216_stone7_tuple_roundtrip.wat — co-located fixture.
;; Arc 216 Stone 7 — Tuple round-trip through HolonAST::Bundle of positional-Binds.

;; p1/p2: 2-tuple (i64, String) round-trip — probes 1 and 2 exercise the same encoding
(:wat::core::defn :t::p1-rt-pair [] -> :(wat::core::i64,wat::core::String)
  (:wat::core::let
    [t  (:wat::core::Tuple 1 "hello")
     h  (:wat::holon::to-holon t)
     rt (:wat::holon::from-holon h)]
    rt))

(:wat::core::defn :t::p2-rt-pair [] -> :(wat::core::i64,wat::core::String)
  (:wat::core::let
    [t  (:wat::core::Tuple 1 "hello")
     h  (:wat::holon::to-holon t)
     rt (:wat::holon::from-holon h)]
    rt))

;; p3: 3-tuple (bool, i64, String) round-trip
(:wat::core::defn :t::p3-rt-triple [] -> :(wat::core::bool,wat::core::i64,wat::core::String)
  (:wat::core::let
    [t  (:wat::core::Tuple true 42 "wat")
     h  (:wat::holon::to-holon t)
     rt (:wat::holon::from-holon h)]
    rt))

;; p4: nested tuple ((i64, i64), String) round-trip
(:wat::core::defn :t::p4-rt-nested [] -> :((wat::core::i64,wat::core::i64),wat::core::String)
  (:wat::core::let
    [inner (:wat::core::Tuple 1 2)
     outer (:wat::core::Tuple inner "outer")
     h     (:wat::holon::to-holon outer)
     rt    (:wat::holon::from-holon h)]
    rt))

;; p5: tuple containing Vec<i64> round-trip
(:wat::core::defn :t::p5-rt-with-vec [] -> (:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::core::i64]) :wat::core::String])
  (:wat::core::let
    [v  [1 2 3]
     t  (:wat::core::Tuple v "tag")
     h  (:wat::holon::to-holon t)
     rt (:wat::holon::from-holon h)]
    rt))

;; p6: tuple containing HashSet<i64> round-trip
(:wat::core::defn :t::p6-rt-with-set [] -> (:wat::core::Tuple :- [(:wat::core::HashSet :- [:wat::core::i64]) :wat::core::String])
  (:wat::core::let
    [s  (:wat::core::HashSet :wat::core::i64 1 2)
     t  (:wat::core::Tuple s "label")
     h  (:wat::holon::to-holon t)
     rt (:wat::holon::from-holon h)]
    rt))

;; p7-admits: Tuple<i64, String> passes is_atomizable check → returns 1
(:wat::core::defn :t::p7-admits [] -> :wat::core::i64
  (:wat::core::let
    [t (:wat::core::Tuple 1 "hello")
     h (:wat::holon::to-holon t)]
    1))
