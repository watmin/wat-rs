;; BRIEF-cond-the-first-macro-backed-rete-row.md grounding probe (BASELINE, pre-any-change) —
;; does core's PLAIN `:wat::core::cond` (unmodified, already exists, already has a purity guard
;; in rete/purity.rs's classify_expr) actually FIRE when written literally inside a
;; `(:wat::rete::where ...)` clause? The where-clause AST is never macro-expanded (defrule quotes
;; :when/:then verbatim; eval_test_core calls runtime::eval_inner directly on the raw form) — so
;; this probes whether cond, which has ZERO runtime dispatch arm (only a defmacro), can survive
;; that path at all, independent of any rete-name aliasing.
;;
;; POSITIVE CONTROL as of arc 278 task #78 (BRIEF-where-bodies-expand-at-compile-time.md). This
;; was a GAP-WITNESS: before task #78, it raised `#wat.runtime/UnknownFunction {:message "unknown
;; function: :wat::core::cond"}` at fire time — the same "where body never macro-expands" gap the
;; RETE-spelled sibling (`probe-cond-rete-where.wat`) measured, independent of any rete-name
;; aliasing. Task #78 taught `expand_form` to expand a `(:wat::rete::where …)` form's body (via
;; the new `resolve::boundary::Boundary::MakeRule` classification), leaving the surrounding
;; `:probe::Req` fact pattern untouched. Now prints `hits=1` — core-spelled `cond` composes
;; correctly in a real `defrule`'s `where` too.

(:wat::core::defrecord :probe::Req [a <- :wat::core::bool])
(:wat::core::defrecord :probe::Hit [a <- :wat::core::bool])

(:wat::rete::defrule :probe::r1
  :when
  [(:probe::Req (?a <- :a))
   (:wat::rete::where (:wat::rete::core::cond (?a true) (:else false)))]
  :then
  [(:probe::Hit :a ?a)])

(:wat::rete::defquery :probe::q-Hit
  :params []
  :when [(?fact <- :probe::Hit)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::core::PersistentVector (:probe::r1))
     staged  (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) (:probe::Req :a true)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     hits    (:wat::rete::query fired (:probe::q-Hit))]
    (:wat::kernel::println (:wat::core::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
