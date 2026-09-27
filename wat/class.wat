;; Stone 255.54 — Orderable and Equatable.
;;
;; Featureless surfaces: membership is a declared edge (B1). Nature is Struct so
;; the surface does not become a :wat::core::Record (that chain would make every
;; Orderable a record). The gates `<` and `=` still use the Rust predicates.
;; 255.55 switches them.
;;
;; :nature is mandatory on defsurface. Struct is the nature that does not
;; register this surface under :wat::core::Record.

(:wat::core::defsurface :wat::core::Orderable :nature :wat::core::Struct :features [])
(:wat::core::defsurface :wat::core::Equatable :nature :wat::core::Struct :features [])

;; ── Orderable leaves. The predicate's leaf arms, plus bigint and rational
;; (the runtime orders them; the predicate does not). ────────────────────────
(:wat::core::extend-type :wat::core::i64 :wat::core::Orderable)
(:wat::core::extend-type :wat::core::u8 :wat::core::Orderable)
(:wat::core::extend-type :wat::core::f64 :wat::core::Orderable)
(:wat::core::extend-type :wat::core::bigint :wat::core::Orderable)
(:wat::core::extend-type :wat::core::rational :wat::core::Orderable)
(:wat::core::extend-type :wat::core::String :wat::core::Orderable)
(:wat::core::extend-type :wat::core::bool :wat::core::Orderable)
(:wat::core::extend-type :wat::core::keyword :wat::core::Orderable)
(:wat::core::extend-type :wat::time::Instant :wat::core::Orderable)
(:wat::core::extend-type :wat::time::Duration :wat::core::Orderable)
(:wat::core::extend-type :wat::holon::Vector :wat::core::Orderable)

;; ── Orderable containers. A tuple of any arity, each slot Orderable. ────────
(:wat::core::extend-type :- [[T :< :wat::core::Orderable]]
  (:wat::core::Vector :- [T]) :wat::core::Orderable)
(:wat::core::extend-type :- [[T :< :wat::core::Orderable]]
  (:wat::core::Option :- [T]) :wat::core::Orderable)
(:wat::core::extend-type :- [[T :< :wat::core::Orderable] [E :< :wat::core::Orderable]]
  (:wat::core::Result :- [T E]) :wat::core::Orderable)
;; A variant registers <: its enum, but a conditional edge is not a string
;; subtype, so that edge is not transitive. The ordering predicate widens a
;; variant to the enum (`widen_to_enclosing_enum`). These edges are that
;; widening, stated as membership. Option.None carries no parameter.
(:wat::core::extend-type :wat::core::Option.None :wat::core::Orderable)
(:wat::core::extend-type :- [[T :< :wat::core::Orderable]]
  (:wat::core::Option.Some :- [T]) :wat::core::Orderable)
(:wat::core::extend-type :- [[T :< :wat::core::Orderable] [E :< :wat::core::Orderable]]
  (:wat::core::Result.Ok :- [T E]) :wat::core::Orderable)
(:wat::core::extend-type :- [[T :< :wat::core::Orderable] [E :< :wat::core::Orderable]]
  (:wat::core::Result.Err :- [T E]) :wat::core::Orderable)
(:wat::core::extend-type :- [[Ts :< :wat::core::Orderable] :..]
  (:wat::core::Tuple :- [Ts :..]) :wat::core::Orderable)

;; ── Equatable leaves. The predicate's leaf arms. ────────────────────────────
(:wat::core::extend-type :wat::core::i64 :wat::core::Equatable)
(:wat::core::extend-type :wat::core::u8 :wat::core::Equatable)
(:wat::core::extend-type :wat::core::f64 :wat::core::Equatable)
(:wat::core::extend-type :wat::core::bigint :wat::core::Equatable)
(:wat::core::extend-type :wat::core::rational :wat::core::Equatable)
(:wat::core::extend-type :wat::core::String :wat::core::Equatable)
(:wat::core::extend-type :wat::core::bool :wat::core::Equatable)
(:wat::core::extend-type :wat::core::keyword :wat::core::Equatable)
(:wat::core::extend-type :wat::core::Uuid :wat::core::Equatable)
(:wat::core::extend-type :wat::core::char :wat::core::Equatable)
(:wat::core::extend-type :wat::time::Instant :wat::core::Equatable)
(:wat::core::extend-type :wat::time::Duration :wat::core::Equatable)
(:wat::core::extend-type :wat::holon::Vector :wat::core::Equatable)
(:wat::core::extend-type :wat::holon::HolonAST :wat::core::Equatable)
(:wat::core::extend-type :wat::WatAST :wat::core::Equatable)

;; ── Equatable containers. HashMap, HashSet, PersistentVector are unconditional
;; (their runtime equality is Value's total PartialEq). PersistentMap is not a
;; member. ────────────────────────────────────────────────────────────────────
(:wat::core::extend-type :- [[T :< :wat::core::Equatable]]
  (:wat::core::Vector :- [T]) :wat::core::Equatable)
(:wat::core::extend-type :- [[T :< :wat::core::Equatable]]
  (:wat::core::List :- [T]) :wat::core::Equatable)
(:wat::core::extend-type :- [[T :< :wat::core::Equatable]]
  (:wat::core::Option :- [T]) :wat::core::Equatable)
(:wat::core::extend-type :- [[T :< :wat::core::Equatable] [E :< :wat::core::Equatable]]
  (:wat::core::Result :- [T E]) :wat::core::Equatable)
(:wat::core::extend-type :- [[Ts :< :wat::core::Equatable] :..]
  (:wat::core::Tuple :- [Ts :..]) :wat::core::Equatable)
(:wat::core::extend-type :- [K V]
  (:wat::core::HashMap :- [K V]) :wat::core::Equatable)
(:wat::core::extend-type :- [T]
  (:wat::core::HashSet :- [T]) :wat::core::Equatable)
(:wat::core::extend-type :- [T]
  (:wat::core::PersistentVector :- [T]) :wat::core::Equatable)

;; Q1 — records and holon records. :wat::holon::Record is already a subtype of
;; :wat::core::Record, so one edge covers both. Structs are not members.
(:wat::core::extend-type :wat::core::Record :wat::core::Equatable)
;; EN-P — a Pure enum registers <: :wat::enum::Pure. Impure enums do not.
(:wat::core::extend-type :wat::enum::Pure :wat::core::Equatable)
