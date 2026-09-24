;; tests/types/probe_stone118_3b_seqable_parametric_satisfaction.wat — co-located fixture.
;;
;; Stone 118.3-B — a concrete container satisfies a PARAMETRIC surface bound. The checker's
;; `(Parametric actual, Parametric expected)` arm string-compared a registered `extend-type` edge
;; against the call site's rendered expected type (a fresh unification var, `(Seqable :- [?454])`)
;; — never equal, so no concrete container could satisfy a parametric surface. See
;; docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/{BRIEF,EXPECTATIONS,MEASURED}-118.3-B*.md.
;;
;; Stone 255.22 — this fixture now tests the LANGUAGE's own `:wat::core::Seqable` (wat/seq.wat),
;; not a private copy. Its old `:t118b::Seqable`/`:t118b::BareSeqable` surfaces and their
;; `as-vec`/`as-vec-bare` methods were probe inventions standing in for `(:wat::core::into [] coll)`;
;; results are read through `into` now. The four `wat/seq.wat` edges declare their parameter
;; (`(extend-type :- [T] (Vector :- [T]) (Seqable :- [T]) …)`), and the checker matches each edge
;; by that binder: the edge's child is pattern-matched against the actual, and the target is
;; instantiated under what it bound.

;; ─── a generic fn over ANY (Seqable :- [T]) — the parametric-satisfaction row ───────────────
(:wat::core::defn :t118b::count-of :- [T] [s <- (:wat::core::Seqable :- [T])] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq s))))

;; ─── a CONCRETE (Seqable :- [i64]) bound — the instantiation row ────────────────────────────
;; The edge's target, instantiated for the actual, must UNIFY with the concrete bound: a
;; `(Vector :- [i64])` offers `(Seqable :- [i64])`. The element type reaches the body: `+` on each
;; element type-checks only if `seq`'s result is `(Stream :- [i64])`.
(:wat::core::defn :t118b::sum-of [s <- (:wat::core::Seqable :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::+ acc x))
                     0
                     (:wat::core::into [] (:wat::core::Seqable/seq s))))

;; ─── entry points, driven via call_beside_value ─────────────────────────────────────────────

;; row 1 — all four containers satisfy the parametric surface.
(:wat::core::defn :t::param-vector [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))

(:wat::core::defn :t::param-persistent-vector [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::PersistentVector 1 2 3 4)))

(:wat::core::defn :t::param-list [] -> :wat::core::i64
  (:t118b::count-of (:wat::core::List 1 2 3 4 5)))

(:wat::core::defn :t::param-stream [] -> :wat::core::i64
  (:t118b::count-of (:wat::stream::cons 1
                       (:wat::stream::lazy
                         (:wat::stream::cons 2
                           (:wat::stream::lazy (:wat::stream::empty)))))))

;; row 2 — all four containers satisfy a CONCRETE instantiation of the surface.
(:wat::core::defn :t::sum-vector [] -> :wat::core::i64
  (:t118b::sum-of (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))

(:wat::core::defn :t::sum-persistent-vector [] -> :wat::core::i64
  (:t118b::sum-of (:wat::core::PersistentVector 1 2 3 4)))

(:wat::core::defn :t::sum-list [] -> :wat::core::i64
  (:t118b::sum-of (:wat::core::List 1 2 3 4 5)))

(:wat::core::defn :t::sum-stream [] -> :wat::core::i64
  (:t118b::sum-of (:wat::stream::cons 10
                     (:wat::stream::lazy
                       (:wat::stream::cons 20
                         (:wat::stream::lazy (:wat::stream::empty)))))))
