;; wat-tests/core/core-into-persistentvector-from-vector.wat — corpus witness for
;; DESIGN-STONE-into-pv-from-vector.md: `into`'s fourth clause,
;; ((PersistentVector :- [T]), (Vector :- [T])) -> (PersistentVector :- [T]) (wat/seq.wat), backed by the new
;; native `:wat::core::PersistentVector/concat` (src/collection/eval.rs, src/runtime.rs
;; dispatch, src/check.rs `infer_persistentvector_concat`).
;;
;; Before this stone, the RED gate was `NoMatchingClauseAtCallSite` — `into` had only
;; (Vector,Vector), (Vector,Stream), (PersistentVector,Stream). Nine `wat-scripts/perf/grid/*.wat`
;; harnesses hand-rolled a `foldl`+`conj` bridge as a workaround (N interpreted closure
;; invocations); this test is the permanent corpus witness that the native clause is correct so
;; that workaround never needs to come back.
;;
;; Three things a test here must prove (BRIEF-into-pv-from-vector.md):
;;   1. the RED gate flips — `into` accepts a PersistentVector receiver + Vector source.
;;   2. the receiver's KIND is preserved — the result is a PersistentVector, not a Vector.
;;      `assert-eq :- [T]` is monomorphic over ONE T for both sides — `expected` below is a
;;      PersistentVector literal, so a runtime value that came back a Vector instead fails
;;      loudly (`values_equal` has no Vec×PersistentVector arm — falls to `TypeMismatch`,
;;      never a silent `false`), which IS the kind assertion, not just an element compare.
;;   3. the PV×PV clause on `PersistentVector/concat` itself (the op's second scheme; `into`
;;      does not get a (PersistentVector,PersistentVector) clause — that combination was never
;;      in the BRIEF's scope and would need its own DESIGN decision).

;; ─── into: PersistentVector receiver, Vector source ──────────────────────────────
;;
;; #pv[1 2] `into` [3 4] = #pv[1 2 3 4]. The RED gate from the DESIGN doc, exactly.

(:wat::test::deftest :wat-tests::core::core-into-persistentvector-from-vector::into-pv-from-vector

  (:wat::core::let
    [to       (:wat::core::PersistentVector 1 2)
     from     (:wat::core::Vector :wat::core::i64 3 4)
     combined (:wat::core::into to from)
     expected (:wat::core::PersistentVector 1 2 3 4)]
    (:wat::test::assert-eq combined expected)))

;; ─── into: empty PersistentVector receiver (the exact vec->pvec shape) ───────────
;;
;; `(into (PersistentVector) v)` — the one-liner the nine grid axes now call instead of the
;; hand-rolled conj-fold. Proves the empty-receiver case (the actual call shape in production).

(:wat::test::deftest :wat-tests::core::core-into-persistentvector-from-vector::into-pv-from-vector-empty-receiver

  (:wat::core::let
    [empty    (:wat::core::PersistentVector)
     from     (:wat::core::Vector :wat::core::i64 5 6 7)
     combined (:wat::core::into empty from)
     expected (:wat::core::PersistentVector 5 6 7)]
    (:wat::test::assert-eq combined expected)))

;; ─── into: Vector source order is preserved, not just set membership ─────────────

(:wat::test::deftest :wat-tests::core::core-into-persistentvector-from-vector::into-pv-from-vector-order-preserved

  (:wat::core::let
    [to       (:wat::core::PersistentVector)
     from     (:wat::core::Vector :wat::core::i64 3 1 4 1 5)
     combined (:wat::core::into to from)
     expected (:wat::core::PersistentVector 3 1 4 1 5)]
    (:wat::test::assert-eq combined expected)))

;; ─── PersistentVector/concat: the PV×PV scheme (the op's OTHER clause) ───────────
;;
;; Called directly (not through `into`, which has no (PersistentVector,PersistentVector)
;; clause) — proves `PersistentVector/concat`'s second scheme, (PV :- [T]) × (PV :- [T]) -> (PV :- [T]).

(:wat::test::deftest :wat-tests::core::core-into-persistentvector-from-vector::persistentvector-concat-pv-from-pv

  (:wat::core::let
    [to       (:wat::core::PersistentVector 1 2)
     from     (:wat::core::PersistentVector 3 4)
     combined (:wat::core::PersistentVector/concat to from)
     expected (:wat::core::PersistentVector 1 2 3 4)]
    (:wat::test::assert-eq combined expected)))

;; ─── PersistentVector/concat: length of the concatenated result ──────────────────

(:wat::test::deftest :wat-tests::core::core-into-persistentvector-from-vector::persistentvector-concat-length

  (:wat::core::let
    [to       (:wat::core::PersistentVector 1 2 3)
     from     (:wat::core::Vector :wat::core::i64 4 5)
     combined (:wat::core::PersistentVector/concat to from)]
    (:wat::test::assert-eq (:wat::core::length combined) 5)))
