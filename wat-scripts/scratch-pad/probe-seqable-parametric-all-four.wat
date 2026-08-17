;; ★ THE FULL chain-D `Seqable` DESIGN, TYPE-CHECKING TODAY. Run 2026-08-17, --check exit 0.
;;
;; Sibling of probe-seqable-is-spellable-today.wat, which refuted infer.rs:638's three blockers
;; with a NON-parametric surface over two containers. This one closes the remaining three
;; unknowns that probe explicitly did NOT prove:
;;
;;   1. PARAMETRIC surface  -> `:sq::Seqable<T>` declares and checks. (Exemplar: wat/capability.wat:44
;;      `:wat::capability::Dialable<S,R>`.)
;;   2. ALL FOUR containers -> Vector, PersistentVector, List, AND Stream all extend-type onto it,
;;      matching `extract_lazyable_elem`'s hardcoded four-head set exactly.
;;   3. A GENERIC fn over the surface -> `:sq::count-of<T> [s <- :sq::Seqable<T>]` checks.
;;
;; So the chain doc's D signature is buildable as written:
;;   (defsurface wat.type/Seqable [T] (seq [self] :- (wat.type/Seq [T])))
;;
;; ⚠ ONE REAL FRICTION FOUND, and it is NOT about surfaces: `into` has no (Vector<T>, List) clause.
;; The List arm below routes through foldl+conj instead. That is the SAME missing-clause class
;; already shipped once for PersistentVector (task #45) — a sibling clause, not a blocker.
;;
;; ⚠ STILL UNMEASURED: per-element dispatch COST. join/map/filter walk every element; a surface
;; dispatch per element is a real perf question and nothing here speaks to it.

;; DISCONFIRMING PROBE #2 — PARAMETRIC Seqable<T> over ALL FOUR containers.
(:wat::core::defsurface :sq::Seqable<T> :nature :wat::core::Struct
  :features [(as-vec [self <- :sq::Seqable<T>] -> :wat::core::Vector<T>)])

(:wat::core::extend-type :wat::core::Vector :sq::Seqable<T>
  (as-vec [self] -> :wat::core::Vector<T> self))

(:wat::core::extend-type :wat::core::PersistentVector :sq::Seqable<T>
  (as-vec [self] -> :wat::core::Vector<T> (:wat::core::into (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::core::List :sq::Seqable<T>
  (as-vec [self] -> :wat::core::Vector<T>
    (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::Vector<T> x <- :T] -> :wat::core::Vector<T>
                         (:wat::core::conj acc x))
                       (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::stream::Stream :sq::Seqable<T>
  (as-vec [self] -> :wat::core::Vector<T> (:wat::core::into (:wat::core::Vector :T) self)))

;; THE PAYOFF — one generic fn over ANY Seqable<T>
(:wat::core::defn :sq::count-of<T> [s <- :sq::Seqable<T>] -> :wat::core::i64
  (:wat::core::length (:sq::Seqable/as-vec s)))
