;; tests/collection/probe_arc216_stone3_hashmap_roundtrip.wat — co-located fixture.
;; Arc 216 Stone 3 — (HashMap :- [K V]) round-trip through HolonAST::Bundle of arbitrary-K Binds.

;; p1: forward round-trip length 2
(:wat::core::defn :t::p1-forward-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  {:foo 42 :bar 99}
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p2a: round-trip length 2
(:wat::core::defn :t::p2a-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  {:foo 42 :bar 99}
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p2b: round-trip contains :foo
(:wat::core::defn :t::p2b-rt-foo [] -> :wat::core::bool
  (:wat::core::let
    [m  {:foo 42 :bar 99}
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::contains-key? rv :foo)))

;; p2c: round-trip contains :bar
(:wat::core::defn :t::p2c-rt-bar [] -> :wat::core::bool
  (:wat::core::let
    [m  {:foo 42 :bar 99}
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::contains-key? rv :bar)))

;; p3a: empty map forward round-trip length 0
(:wat::core::defn :t::p3a-empty-rt-forward [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p3b: empty map with consumer hint still length 0
(:wat::core::defn :t::p3b-empty-rt-reverse [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h -> :wat::core::HashMap)]
    (:wat::hashmap::length rv)))

;; p4a: (HashMap :- [keyword i64]) round-trip length 2
(:wat::core::defn :t::p4a-kw-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1 :b 2)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p4b: (HashMap :- [String i64]) round-trip length 2
(:wat::core::defn :t::p4b-str-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::String :wat::core::i64 "x" 10 "y" 20)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p4c: (HashMap :- [i64 String]) round-trip length 2
(:wat::core::defn :t::p4c-i64k-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p4d: (HashMap :- [bool i64]) round-trip length 2
(:wat::core::defn :t::p4d-bool-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::bool :wat::core::i64 true 1 false 0)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p5a: (HashMap :- [keyword i64]) V=i64 length 1
(:wat::core::defn :t::p5a-v-i64 [] -> :wat::core::i64
  (:wat::core::let
    [m  {:foo 42}
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p5b: (HashMap :- [keyword String]) V=String length 2
(:wat::core::defn :t::p5b-v-str [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::String :name "alice" :city "paris")
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p5c: (HashMap :- [keyword bool]) V=bool length 2
(:wat::core::defn :t::p5c-v-bool [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::bool :active true :disabled false)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p5d: (HashMap :- [keyword keyword]) V=keyword length 2
(:wat::core::defn :t::p5d-v-kw [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::keyword :role :admin :mode :active)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p6a: (HashMap :- [i64 String]) round-trip length 2
(:wat::core::defn :t::p6a-i64k-rt-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p6b: (HashMap :- [i64 String]) round-trip contains-key? 100
(:wat::core::defn :t::p6b-i64k-rt-contains [] -> :wat::core::bool
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::i64 :wat::core::String 100 "hello" 200 "world")
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::contains-key? rv 100)))

;; p7a: nested map outer length 1
(:wat::core::defn :t::p7a-nested-outer-len [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1 :y 2)
     outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
     h     (:wat::holon::to-holon outer)
     rv    (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p7b: nested map arc 228 outer length 1
(:wat::core::defn :t::p7b-nested-arc228 [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1 :y 2)
     outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
     h     (:wat::holon::to-holon outer)
     rv    (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p8a: (HashMap :- [keyword (Vec :- [i64])]) outer length 1
(:wat::core::defn :t::p8a-hashmap-of-vec-len [] -> :wat::core::i64
  (:wat::core::let
    [v  (:wat::core::Vector :wat::core::i64 10 20 30)
     m  (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data v)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p8b: (HashMap :- [keyword (Vec :- [i64])]) arc 228 outer length 1
(:wat::core::defn :t::p8b-hashmap-of-vec-arc228 [] -> :wat::core::i64
  (:wat::core::let
    [v  (:wat::core::Vector :wat::core::i64 10 20 30)
     m  (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data v)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p9a: (HashMap :- [keyword (HashSet :- [i64])]) outer length 1
(:wat::core::defn :t::p9a-hashmap-of-set-len [] -> :wat::core::i64
  (:wat::core::let
    [s  (:wat::core::HashSet :wat::core::i64 1 2 3)
     m  (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data s)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p9b: (HashMap :- [keyword (HashSet :- [i64])]) arc 228 outer length 1
(:wat::core::defn :t::p9b-hashmap-of-set-arc228 [] -> :wat::core::i64
  (:wat::core::let
    [s  (:wat::core::HashSet :wat::core::i64 1 2 3)
     m  (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :data s)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p10a: atomizable passes — returns 1
(:wat::core::defn :t::p10a-atomizable-passes [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1)]
    (:wat::holon::to-holon m)
    1))

;; p10b: nested atomizable passes — returns 1
(:wat::core::defn :t::p10b-nested-atomizable [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 1)
     outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)
     h     (:wat::holon::to-holon outer)]
    1))

;; p13: non-sequential i64 keys → HashMap round-trip length 2
(:wat::core::defn :t::p13-non-seq-i64-keys [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::i64 :wat::core::String 0 "a" 5 "b")
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p14a: empty HashMap classifier unannotated → length 0
(:wat::core::defn :t::p14a-empty-classifier-len [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h)]
    (:wat::hashmap::length rv)))

;; p14b: empty HashMap annotated form → length 0
(:wat::core::defn :t::p14b-empty-classifier-annotated [] -> :wat::core::i64
  (:wat::core::let
    [m  (:wat::core::HashMap :wat::core::keyword :wat::core::i64)
     h  (:wat::holon::to-holon m)
     rv (:wat::holon::from-holon h -> :wat::core::HashMap)]
    (:wat::hashmap::length rv)))
