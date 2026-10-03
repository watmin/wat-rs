;; tests/function/probe_arc237_7b_intrinsic_typing.wat
;; Arc 237 Stone 237.7b — ∀T intrinsic typing precision (empty?, contains?, get, conj).
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *.wat.bad files.

;; TIER A — empty? (∀T -> bool)
(wat.core/defn user/empty-q-vector [] :- wat.type/bool
  (wat.core/empty? (wat.type/Vector :- [wat.type/i64])))

(wat.core/defn user/empty-q-hashset-false [] :- wat.type/bool
  (wat.core/empty? (wat.type/HashSet :- [wat.type/i64] 1 2)))

;; TIER A — contains? ((coll, elem) -> bool)
(wat.core/defn user/contains-q-vector-hit [] :- wat.type/bool
  (wat.core/contains? (wat.type/Vector :- [wat.type/i64] 1 2 3) 2))

;; TIER B — get ((coll, key) -> (Option :- [element]))
(wat.core/defn user/get-vector-precise [] :- wat.type/i64
  (wat.core/match (wat.core/get (wat.type/Vector :- [wat.type/i64] 10 20 30) 1)
                     
                     [wat.core/Option.Some {:value x} (wat.i64/+ x 5)]
                     [wat.core/Option.None {} -1]))

;; TIER B — conj ((coll, elem) -> coll)
(wat.core/defn user/conj-vector-preserves [] :- wat.type/i64
  (wat.core/length (wat.core/conj (wat.type/Vector :- [wat.type/i64] 1 2) 3)))
