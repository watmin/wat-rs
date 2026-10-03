;; tests/function/probe_arc237_7a_length_intrinsic.wat
;; Arc 237 Stone 237.7a — :wat::core::length as a ∀T intrinsic.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Each named fn is exercised by its sibling Rust test.

(wat.core/defn user/length-vector [] :- wat.type/i64
  (wat.core/length [1 2 3]))

(wat.core/defn user/length-vector-empty [] :- wat.type/i64
  (wat.core/length []))

(wat.core/defn user/length-vector-strings [] :- wat.type/i64
  (wat.core/length ["a" "b"]))

(wat.core/defn user/length-hashmap [] :- wat.type/i64
  (wat.core/length {:a 1 :b 2}))

(wat.core/defn user/length-hashset [] :- wat.type/i64
  (wat.core/length (wat.type/HashSet :- [wat.type/i64] 1 2 3)))

;; Runtime-error case: ∀T accepts i64 at check time; fails at eval (not a collection).
(wat.core/defn user/length-noncollection [] :- wat.type/i64
  (wat.core/length 5))
