;; BRIEF-cond-the-first-macro-backed-rete-row.md grounding probe (BASELINE, pre-any-change) —
;; does core's PLAIN `:wat::core::cond` (unmodified, already exists, already has a purity guard
;; in rete/purity.rs's classify_expr) actually FIRE when written literally inside a
;; `(:wat::rete::where ...)` clause? The where-clause AST is never macro-expanded (defrule quotes
;; :when/:then verbatim; eval_test_core calls runtime::eval_inner directly on the raw form) — so
;; this probes whether cond, which has ZERO runtime dispatch arm (only a defmacro), can survive
;; that path at all, independent of any rete-name aliasing.

(:wat::core::defrecord :probe::Req [a <- :wat::core::bool])
(:wat::core::defrecord :probe::Hit [a <- :wat::core::bool])

(:wat::rete::defrule :probe::r1
  :when
  [(:probe::Req (?a <- :a))
   (:wat::rete::where (:wat::core::cond (?a true) (:else false)))]
  :then
  [(:probe::Hit :a ?a)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::core::PersistentVector (:probe::r1))
     staged  (:wat::rete::insert (:wat::rete::compile rules) (:probe::Req :a true))
     fired   (:wat::rete::fire-rules staged)
     hits    (:wat::rete::query fired :probe::Hit)]
    (:wat::kernel::println (:wat::core::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
