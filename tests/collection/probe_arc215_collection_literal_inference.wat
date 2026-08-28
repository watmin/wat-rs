;; tests/collection/probe_arc215_collection_literal_inference.wat — co-located fixture.
;; Arc 215 Stone 1 — _infer placeholder + literal completion probes.

;; probe 1a: single pair {foo 42} length 1
(:wat::core::defn :t::p1a-map-len [] -> :wat::core::i64
  (:wat::core::length {:foo 42}))

;; probe 1b: single pair contains :foo
(:wat::core::defn :t::p1b-map-contains [] -> :wat::core::bool
  (:wat::hashmap::contains-key? {:foo 42} :foo))

;; probe 2a: multi pair length 3
(:wat::core::defn :t::p2a-map-len [] -> :wat::core::i64
  (:wat::core::length {:a 1 :b 2 :c 3}))

;; probe 2b: get :b from multi-pair map → 2
(:wat::core::defn :t::p2b-map-get-b [] -> :wat::core::i64
  (:wat::core::let
    [m {:a 1 :b 2 :c 3}]
    (:wat::core::match (:wat::core::get m :b) 
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))

;; probe 3: string-valued map length 2
(:wat::core::defn :t::p3-string-map-len [] -> :wat::core::i64
  (:wat::core::length {:a "hello" :b "world"}))

;; probe 4a: nested map outer length 1
(:wat::core::defn :t::p4a-nested-map-outer-len [] -> :wat::core::i64
  (:wat::core::length {:outer {:inner 42}}))

;; probe 4b: get :outer → inner map; length of inner = 1
(:wat::core::defn :t::p4b-nested-map-inner-len [] -> :wat::core::i64
  (:wat::core::let
    [outer {:outer {:inner 42}}]
    (:wat::core::match (:wat::core::get outer :outer) 
      ((:wat::core::Some inner-map) (:wat::core::length inner-map))
      (:wat::core::None -1))))

;; probe 6: empty {} length 0
(:wat::core::defn :t::p6-empty-map-len [] -> :wat::core::i64
  (:wat::core::length {}))

;; probe 7: empty #{} length 0
(:wat::core::defn :t::p7-empty-set-len [] -> :wat::core::i64
  (:wat::core::length #{}))

;; probe 8a: single element #{42} length 1
(:wat::core::defn :t::p8a-single-set-len [] -> :wat::core::i64
  (:wat::core::length #{42}))

;; probe 8b: single element #{42} contains 42
(:wat::core::defn :t::p8b-single-set-contains [] -> :wat::core::bool
  (:wat::core::contains? #{42} 42))

;; probe 9a: multi element #{1 2 3} length 3
(:wat::core::defn :t::p9a-multi-set-len [] -> :wat::core::i64
  (:wat::core::length #{1 2 3}))

;; probe 9b: multi element #{1 2 3} contains 2
(:wat::core::defn :t::p9b-multi-set-contains [] -> :wat::core::bool
  (:wat::core::contains? #{1 2 3} 2))

;; probe 10: dedup #{1 1 2 2 3} length 3
(:wat::core::defn :t::p10-set-dedup-len [] -> :wat::core::i64
  (:wat::core::length #{1 1 2 2 3}))

;; probe 12a: map of sets outer length 2
(:wat::core::defn :t::p12a-map-of-sets-outer-len [] -> :wat::core::i64
  (:wat::core::length {:a #{1 2} :b #{3 4}}))

;; probe 12b: map of sets inner #{1 2} length 2
(:wat::core::defn :t::p12b-map-of-sets-inner-len [] -> :wat::core::i64
  (:wat::core::let
    [m {:a #{1 2} :b #{3 4}}]
    (:wat::core::match (:wat::core::get m :a) 
      ((:wat::core::Some s) (:wat::core::length s))
      (:wat::core::None -1))))
