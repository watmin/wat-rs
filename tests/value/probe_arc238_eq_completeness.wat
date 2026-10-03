;; tests/value/probe_arc238_eq_completeness.wat — co-located fixture.
;; Slurped via startup_beside(file!()). Each function covers one equality contract.
;; All functions return :wat::core::bool; Rust side asserts expected truth value.

(wat.core/defrecord my/Pt [x :- wat.type/i64  y :- wat.type/i64])

;; ─── Records ──────────────────────────────────────────────────────────────────

(wat.core/defn t/record-equal [] :- wat.type/bool
  (wat.core/= (my/Pt :x 1 :y 2) (my/Pt :x 1 :y 2)))

(wat.core/defn t/record-unequal-value [] :- wat.type/bool
  (wat.core/= (my/Pt :x 1 :y 2) (my/Pt :x 1 :y 9)))

;; ─── HashMap ──────────────────────────────────────────────────────────────────

(wat.core/defn t/map-equal [] :- wat.type/bool
  (wat.core/= {:a 1 :b 2} {:a 1 :b 2}))

(wat.core/defn t/map-order-independent [] :- wat.type/bool
  (wat.core/= {:a 1 :b 2} {:b 2 :a 1}))

(wat.core/defn t/map-unequal [] :- wat.type/bool
  (wat.core/= {:a 1} {:a 2}))

;; ─── HashSet ──────────────────────────────────────────────────────────────────

(wat.core/defn t/set-equal [] :- wat.type/bool
  (wat.core/= #{1 2 3} #{1 2 3}))

(wat.core/defn t/set-order-independent [] :- wat.type/bool
  (wat.core/= #{1 2 3} #{3 2 1}))

(wat.core/defn t/set-unequal [] :- wat.type/bool
  (wat.core/= #{1 2} #{1 2 3}))

