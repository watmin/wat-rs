;; tests/collection/probe_brace_map_literal.wat — co-located fixture.
;; Arc 214 P2 — {...} map literal in expression position.

;; probe 1: empty {} length 0
(:wat::core::defn :t::p1-empty-len [] -> :wat::core::i64
  (:wat::core::length {}))

;; probe 2a: single pair {foo 42} length 1
(:wat::core::defn :t::p2a-single-len [] -> :wat::core::i64
  (:wat::core::length {:foo 42}))

;; probe 2b: single pair contains :foo
(:wat::core::defn :t::p2b-single-contains [] -> :wat::core::bool
  (:wat::hashmap::contains-key? {:foo 42} :foo))

;; probe 3a: multi pair {a 1 b 2 c 3} length 3
(:wat::core::defn :t::p3a-multi-len [] -> :wat::core::i64
  (:wat::core::length {:a 1 :b 2 :c 3}))

;; probe 3b: multi pair contains :b
(:wat::core::defn :t::p3b-multi-contains [] -> :wat::core::bool
  (:wat::hashmap::contains-key? {:a 1 :b 2 :c 3} :b))

;; probe 4: nested in expression (:wat::core::length {:a 1 :b 2}) → 2
(:wat::core::defn :t::p4-nested-expr-len [] -> :wat::core::i64
  (:wat::core::length {:a 1 :b 2}))

;; probe 5: map-of-map outer length 1
(:wat::core::defn :t::p5-map-of-map-len [] -> :wat::core::i64
  (:wat::core::length {:outer {:inner 42}}))

;; probe 6: non-keyword key {42 :v} length 1
(:wat::core::defn :t::p6-int-key-len [] -> :wat::core::i64
  (:wat::core::length {42 :v}))
