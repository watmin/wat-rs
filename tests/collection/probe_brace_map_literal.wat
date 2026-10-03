;; tests/collection/probe_brace_map_literal.wat — co-located fixture.
;; Arc 214 P2 — {...} map literal in expression position.

;; AMEND-255.74 D3 — an empty {} literal's key type is a fresh inference variable; nothing else
;; in probe 1 below ever fixed it (`length` doesn't propagate a key constraint), so it was
;; refused as unresolved (BoundUnresolved). `ann-form` ascription was tried and ruled out by the
;; builder — types are headers on function definitions; a dynamic value's type is pinned by a
;; TYPED CONSUMER, not an ascription (arc 258 retired ann-form as a crutch for exactly this).
;; This small typed consumer is that pin; probe 1 still calls it with the bare `{}` LITERAL at
;; the call site.
(wat.core/defn t/map-len [m :- (wat.type/HashMap :- [wat.type/keyword wat.type/i64])] :- wat.type/i64
  (wat.core/length m))

;; probe 1: empty {} length 0
(wat.core/defn t/p1-empty-len [] :- wat.type/i64
  (t/map-len {}))

;; probe 2a: single pair {foo 42} length 1
(wat.core/defn t/p2a-single-len [] :- wat.type/i64
  (wat.core/length {:foo 42}))

;; probe 2b: single pair contains :foo
(wat.core/defn t/p2b-single-contains [] :- wat.type/bool
  (wat.core/contains? {:foo 42} :foo))

;; probe 3a: multi pair {a 1 b 2 c 3} length 3
(wat.core/defn t/p3a-multi-len [] :- wat.type/i64
  (wat.core/length {:a 1 :b 2 :c 3}))

;; probe 3b: multi pair contains :b
(wat.core/defn t/p3b-multi-contains [] :- wat.type/bool
  (wat.core/contains? {:a 1 :b 2 :c 3} :b))

;; probe 4: nested in expression (:wat::core::length {:a 1 :b 2}) → 2
(wat.core/defn t/p4-nested-expr-len [] :- wat.type/i64
  (wat.core/length {:a 1 :b 2}))

;; probe 5: map-of-map outer length 1
(wat.core/defn t/p5-map-of-map-len [] :- wat.type/i64
  (wat.core/length {:outer {:inner 42}}))

;; probe 6: non-keyword key {42 :v} length 1
(wat.core/defn t/p6-int-key-len [] :- wat.type/i64
  (wat.core/length {42 :v}))
