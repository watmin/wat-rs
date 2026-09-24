;; arc 255 Stone 255.18 — MEASUREMENT (2026-09-24). A generic fn holding `(Loc :- [T])` (T
;; declared in its binder) calls a defclause keyed on the CONCRETE implementors — the shape of
;; `:wat::spawn::runner-count` / `:wat::spawn::with-label` called from `map-worker` or from a
;; defservice `start$impl`. Does the defclause's narrowing arm reach a clause?
;;
;; LOADED (this file): the BARE surface `:probe::Loc` narrows to both clauses — rc=0, prints 9.
;;
;; REFUSED (the generic row, kept OUT of the loaded forms so this file stays loadable; the
;; committed refusal WAS tests/types/probe_arc255_18_locus_names_its_transport_generic_narrowing.wat.bad;
;; 255.19 removed the real defclauses — `runner-count`/`with-label` are `Locus` surface methods —
;; so that row is now the positive `…_generic_reads_its_count.wat`. The checker behaviour this
;; file measures is unchanged; only the stdlib stopped depending on it):
;;
;;   (:wat::core::defn :probe::generic :- [T] [l <- (:probe::Loc :- [T])] -> :wat::core::i64
;;     (:probe::count l))
;;
;;   #wat.check/NoMatchingClauseAtCallSite {:message "no clause of `:probe::count` matches arity 1
;;   with types [(:probe::Loc :- [:T])]; clauses attempted: (1: [:probe::Th]); (1: [:probe::Pr])" …}
;;
;; WHY (check.rs, the defclause arm): narrowing tries `assignable(clause-param, arg)` in reverse —
;; `Th <: (Loc :- [:T])` — and 255.15's edge inference must unify `Sh` with the RIGID `:T` of the
;; enclosing binder, which it cannot. The bare surface has no argument to unify, so it passes.
;; So a consumer that forwards its locus to a concrete-keyed defclause cannot take
;; `(Locus :- [T])` today: that is a checker/API decision, not a declaration.
(:wat::core::defstruct :probe::Sh [])
(:wat::core::defstruct :probe::Wi [])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(tag [self <- (:probe::Loc :- [T])] -> :wat::core::i64)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Sh]) (tag [self] 1))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wi]) (tag [self] 2))
(:wat::core::defclause :probe::count
  ([l <- :probe::Th] -> :wat::core::i64 (:probe::Th/x l))
  ([l <- :probe::Pr] -> :wat::core::i64 (:probe::Pr/y l)))
;; bare surface (the family top), narrows through the defclause
(:wat::core::defn :probe::bare [l <- :probe::Loc] -> :wat::core::i64
  (:probe::count l))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::str (:probe::bare (:probe::Pr :y 9)))))
