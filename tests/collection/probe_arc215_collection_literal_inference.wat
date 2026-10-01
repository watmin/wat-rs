;; tests/collection/probe_arc215_collection_literal_inference.wat — co-located fixture.
;; Arc 215 Stone 1 — _infer placeholder + literal completion probes.

;; probe 1a: single pair {foo 42} length 1
(:wat::core::defn :t::p1a-map-len [] -> wat.type/i64
  (:wat::core::length {:foo 42}))

;; probe 1b: single pair contains :foo
(:wat::core::defn :t::p1b-map-contains [] -> wat.type/bool
  (:wat::hashmap::contains-key? {:foo 42} :foo))

;; probe 2a: multi pair length 3
(:wat::core::defn :t::p2a-map-len [] -> wat.type/i64
  (:wat::core::length {:a 1 :b 2 :c 3}))

;; probe 2b: get :b from multi-pair map → 2
(:wat::core::defn :t::p2b-map-get-b [] -> wat.type/i64
  (:wat::core::let
    [m {:a 1 :b 2 :c 3}]
    (:wat::core::match (:wat::core::get m :b) 
      [:wat::core::Option.Some {:value v} v]
      [:wat::core::Option.None {} -1])))

;; probe 3: string-valued map length 2
(:wat::core::defn :t::p3-string-map-len [] -> wat.type/i64
  (:wat::core::length {:a "hello" :b "world"}))

;; probe 4a: nested map outer length 1
(:wat::core::defn :t::p4a-nested-map-outer-len [] -> wat.type/i64
  (:wat::core::length {:outer {:inner 42}}))

;; probe 4b: get :outer → inner map; length of inner = 1
(:wat::core::defn :t::p4b-nested-map-inner-len [] -> wat.type/i64
  (:wat::core::let
    [outer {:outer {:inner 42}}]
    (:wat::core::match (:wat::core::get outer :outer) 
      [:wat::core::Option.Some {:value inner-map} (:wat::core::length inner-map)]
      [:wat::core::Option.None {} -1])))

;; AMEND-255.74 D3 — an empty {}/#{} literal's key/element type is a fresh inference variable;
;; nothing else in probes 6/7 below ever fixed it (`length` doesn't propagate a key/element
;; constraint), so it was refused as unresolved (BoundUnresolved). `ann-form` ascription was
;; tried and ruled out by the builder — types are headers on function definitions; a dynamic
;; value's type is pinned by a TYPED CONSUMER, not an ascription (arc 258 retired ann-form as a
;; crutch for exactly this). These two small typed consumers are that pin; each probe below
;; still calls them with the bare `{}`/`#{}` LITERAL at the call site.
(:wat::core::defn :t::map-len [m <- (wat.type/HashMap :- [wat.type/keyword wat.type/i64])] -> wat.type/i64
  (:wat::core::length m))
(:wat::core::defn :t::set-len [s <- (wat.type/HashSet :- [wat.type/i64])] -> wat.type/i64
  (:wat::core::length s))

;; probe 6: empty {} length 0
(:wat::core::defn :t::p6-empty-map-len [] -> wat.type/i64
  (:t::map-len {}))

;; probe 7: empty #{} length 0
(:wat::core::defn :t::p7-empty-set-len [] -> wat.type/i64
  (:t::set-len #{}))

;; probe 8a: single element #{42} length 1
(:wat::core::defn :t::p8a-single-set-len [] -> wat.type/i64
  (:wat::core::length #{42}))

;; probe 8b: single element #{42} contains 42
(:wat::core::defn :t::p8b-single-set-contains [] -> wat.type/bool
  (:wat::core::contains? #{42} 42))

;; probe 9a: multi element #{1 2 3} length 3
(:wat::core::defn :t::p9a-multi-set-len [] -> wat.type/i64
  (:wat::core::length #{1 2 3}))

;; probe 9b: multi element #{1 2 3} contains 2
(:wat::core::defn :t::p9b-multi-set-contains [] -> wat.type/bool
  (:wat::core::contains? #{1 2 3} 2))

;; probe 10: dedup #{1 1 2 2 3} length 3
(:wat::core::defn :t::p10-set-dedup-len [] -> wat.type/i64
  (:wat::core::length #{1 1 2 2 3}))

;; probe 12a: map of sets outer length 2
(:wat::core::defn :t::p12a-map-of-sets-outer-len [] -> wat.type/i64
  (:wat::core::length {:a #{1 2} :b #{3 4}}))

;; probe 12b: map of sets inner #{1 2} length 2
(:wat::core::defn :t::p12b-map-of-sets-inner-len [] -> wat.type/i64
  (:wat::core::let
    [m {:a #{1 2} :b #{3 4}}]
    (:wat::core::match (:wat::core::get m :a) 
      [:wat::core::Option.Some {:value s} (:wat::core::length s)]
      [:wat::core::Option.None {} -1])))
