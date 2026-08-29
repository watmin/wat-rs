;; tests/collection/probe_hashmap_ctor_vector_symmetric.wat — co-located fixture.
;; Arc 214 P1 — HashMap constructor: Vector-symmetric shape probes.

;; probe 1: empty HashMap length 0
(:wat::core::defn :t::p1-empty-len [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])))

;; probe 2: single pair get :foo → 42
(:wat::core::defn :t::p2-single-get [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64] :foo 42)]
    (:wat::core::match (:wat::core::get m :foo) 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))

;; probe 3a: multi pair length 3
(:wat::core::defn :t::p3a-multi-len [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]
      :a 1
      :b 2
      :c 3)))

;; probe 3b: multi pair get :b → 20
(:wat::core::defn :t::p3b-multi-get [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]
          :a 10
          :b 20
          :c 30)]
    (:wat::core::match (:wat::core::get m :b) 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))

;; probe 4: String-keyed get "b" → 2
(:wat::core::defn :t::p4-str-keyed-get [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :- [:wat::core::String :wat::core::i64]
          "a" 1
          "b" 2)]
    (:wat::core::match (:wat::core::get m "b") 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))

;; probe 5: HolonAST-keyed length 1
(:wat::core::defn :t::p5-holonast-keyed-len [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::HashMap :- [:wat::holon::HolonAST :wat::holon::HolonAST]
      (:wat::holon::to-holon 42) (:wat::holon::to-holon "answer"))))
