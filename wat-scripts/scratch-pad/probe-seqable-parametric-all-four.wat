;; ⛔⛔ CORRECTED 2026-08-17 — THIS FILE PROVES DECLARATIONS ONLY. CALLS DO NOT WORK.
;;
;; As first committed (0548f4f9) this header said "THE FULL chain-D Seqable DESIGN, TYPE-CHECKING
;; TODAY". That was WRONG, and the error was mine: this file DECLARES `:sq::count-of` and never
;; CALLS it, so `--check` exit 0 proved only that the declarations are well-formed.
;;
;; Adding four call sites makes it RED — every one:
;;   :sq::count-of: parameter #1 expects :sq::Seqable<?454>; got :wat::core::Vector<wat::core::i64>
;;
;; ★ THE REAL BLOCKER, now precisely located and NOT any of the three on record: a concrete
;; builtin does NOT unify against a PARAMETRIC surface parameter. Its non-parametric sibling
;; probe-seqable-is-spellable-today.wat DOES run end-to-end and prints "3,4" — so the delta is
;; parametricity alone, not surfaces, not extend-type, not builtins.
;;
;; See docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/NOTE-the-real-blocker-is-parametric-satisfaction.md
;;
;; Sibling of probe-seqable-is-spellable-today.wat, which refuted infer.rs:638's three blockers
;; with a NON-parametric surface over two containers. This one closes the remaining three
;; unknowns that probe explicitly did NOT prove:
;;
;;   1. PARAMETRIC surface  -> `(:sq::Seqable :- [T])` declares and checks. (Exemplar: wat/capability.wat:44
;;      `(:wat::capability::Dialable :- [S R])`.)
;;   2. ALL FOUR containers -> Vector, PersistentVector, List, AND Stream all extend-type onto it,
;;      matching `extract_lazyable_elem`'s hardcoded four-head set exactly.
;;   3. A GENERIC fn over the surface -> `:sq::count-of :- [T] [s <- (:sq::Seqable :- [T])]` checks.
;;
;; So the chain doc's D signature is buildable as written:
;;   (defsurface wat.type/Seqable [T] (seq [self] :- (wat.type/Seq [T])))
;;
;; ⚠ ONE REAL FRICTION FOUND, and it is NOT about surfaces: `into` has no ((Vector :- [T]), List) clause.
;; The List arm below routes through foldl+conj instead. That is the SAME missing-clause class
;; already shipped once for PersistentVector (task #45) — a sibling clause, not a blocker.
;;
;; ⚠ STILL UNMEASURED: per-element dispatch COST. join/map/filter walk every element; a surface
;; dispatch per element is a real perf question and nothing here speaks to it.

;; DISCONFIRMING PROBE #2 — PARAMETRIC (Seqable :- [T]) over ALL FOUR containers.
(:wat::core::defsurface :sq::Seqable :- [T] :nature :wat::core::Struct
  :features [(as-vec [self <- (:sq::Seqable :- [T])] -> (:wat::core::Vector :- [T]))])

(:wat::core::extend-type :wat::core::Vector (:sq::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) self))

(:wat::core::extend-type :wat::core::PersistentVector (:sq::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) (:wat::core::into (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::core::List (:sq::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T])
    (:wat::core::foldl (:wat::core::fn [acc <- (:wat::core::Vector :- [T]) x <- :T] -> (:wat::core::Vector :- [T])
                         (:wat::core::conj acc x))
                       (:wat::core::Vector :T) self)))

(:wat::core::extend-type :wat::stream::Stream (:sq::Seqable :- [T])
  (as-vec [self] -> (:wat::core::Vector :- [T]) (:wat::core::into (:wat::core::Vector :T) self)))

;; THE PAYOFF — one generic fn over ANY (Seqable :- [T])
(:wat::core::defn :sq::count-of :- [T] [s <- (:sq::Seqable :- [T])] -> :wat::core::i64
  (:wat::core::length (:sq::Seqable/as-vec s)))

;; STONE 118.3-B — the four call sites. Pre-fix: each is RED, `TypeMismatch`,
;; `:sq::count-of: parameter #1 expects :sq::Seqable<?N>; got :wat::core::<Container><…>`
;; (the exact defect MEASURED-118.3-B names). Post-fix: type-checks AND runs, "3,4,5,2".
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::join ","
      (:wat::core::Vector :wat::core::i64
        (:sq::count-of (:wat::core::Vector :wat::core::i64 1 2 3))
        (:sq::count-of (:wat::core::PersistentVector 1 2 3 4))
        (:sq::count-of (:wat::core::List/of 1 2 3 4 5))
        (:sq::count-of (:wat::stream::cons 1
                          (:wat::stream::lazy
                            (:wat::stream::cons 2
                              (:wat::stream::lazy (:wat::stream::empty))))))))))
