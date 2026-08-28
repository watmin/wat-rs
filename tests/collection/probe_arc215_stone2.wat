;; tests/collection/probe_arc215_stone2.wat — co-located fixture.
;; Arc 215 Stone 2 — [...]  Vector unification + {...} keyword-key lift probes.

;; probe 1a: [1 2 3] length 3
(:wat::core::defn :t::p1a-vec-len [] -> :wat::core::i64
  (:wat::core::length [1 2 3]))

;; probe 1b: [1 2 3] first element 1
(:wat::core::defn :t::p1b-vec-first [] -> :wat::core::i64
  (:wat::core::match
    (:wat::vec::get [1 2 3] 0)
    
    ((:wat::core::Some v) v)
    (:wat::core::None -1)))

;; probe 2: [1.5 2.5] length 2 (f64)
(:wat::core::defn :t::p2-float-vec-len [] -> :wat::core::i64
  (:wat::core::length [1.5 2.5]))

;; probe 3: ["a" "b"] length 2
(:wat::core::defn :t::p3-str-vec-len [] -> :wat::core::i64
  (:wat::core::length ["a" "b"]))

;; probe 4: [] empty Vec length 0
(:wat::core::defn :t::p4-empty-vec-len [] -> :wat::core::i64
  (:wat::core::length []))

;; probe 5: [true false true] length 3
(:wat::core::defn :t::p5-bool-vec-len [] -> :wat::core::i64
  (:wat::core::length [true false true]))

;; probe 6: (:wat::core::Vector :wat::type::Infer 1 2 3) length 3
(:wat::core::defn :t::p6-explicit-infer-vec-len [] -> :wat::core::i64
  (:wat::core::length (:wat::core::Vector :wat::type::Infer 1 2 3)))

;; probe 7: (:wat::core::Vector :wat::type::Infer) empty length 0
(:wat::core::defn :t::p7-empty-infer-vec-len [] -> :wat::core::i64
  (:wat::core::length (:wat::core::Vector :wat::type::Infer)))

;; probe 9: explicit type form unchanged
(:wat::core::defn :t::p9-explicit-type-vec-len [] -> :wat::core::i64
  (:wat::core::length (:wat::core::Vector :wat::core::i64 1 2 3)))

;; probe 10: let binder [x 1 y 2] preserved
(:wat::core::defn :t::p10-let-binder-preserved [] -> :wat::core::i64
  (:wat::core::let
    [x 1
     y 2]
    (:wat::core::+ x y)))

;; probe 11a: int-keyed map {1 "v" 2 "w"} length 2
(:wat::core::defn :t::p11a-int-keyed-len [] -> :wat::core::i64
  (:wat::core::length {1 "v" 2 "w"}))

;; probe 11b: int-keyed map contains key 1
(:wat::core::defn :t::p11b-int-keyed-contains [] -> :wat::core::bool
  (:wat::hashmap::contains-key? {1 "v" 2 "w"} 1))

;; probe 12a: string-keyed map {"a" 1 "b" 2} length 2
(:wat::core::defn :t::p12a-str-keyed-len [] -> :wat::core::i64
  (:wat::core::length {"a" 1 "b" 2}))

;; probe 12b: string-keyed map contains "a"
(:wat::core::defn :t::p12b-str-keyed-contains [] -> :wat::core::bool
  (:wat::hashmap::contains-key? {"a" 1 "b" 2} "a"))
