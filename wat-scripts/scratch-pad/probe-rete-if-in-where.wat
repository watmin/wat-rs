;; Does a RETE-spelled `if` — a Form row that re-dispatches to core `if`'s genuine runtime arm —
;; actually FIRE inside a `(:wat::rete::where ...)`?
;;
;; This is the load-bearing control for the `cond` question. A `where` body is never
;; macro-expanded (defrule quotes :when verbatim; eval_test_core calls runtime::eval_inner on the
;; raw AST), so a MACRO cannot survive there. A runtime special form can. If this prints hits=1,
;; then "expand rete cond into rete if" is a correct TARGET and macro-expansion of the where body
;; is the ONLY remaining blocker — rather than there being a second problem underneath.

(:wat::core::defrecord :probe::Req [a <- :wat::core::bool])
(:wat::core::defrecord :probe::Hit [a <- :wat::core::bool])

(:wat::rete::defrule :probe::r1
  :when
  [(:probe::Req (?a <- :a))
   (:wat::rete::where (:wat::rete::core::if ?a true false))]
  :then
  [(:probe::Hit :a ?a)])

(:wat::rete::defquery :probe::q-Hit
  :params []
  :when [(?fact <- :probe::Hit)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::core::PersistentVector (:probe::r1))
     staged  (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")]) (:probe::Req :a true)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired   (:wat::core::match (:wat::rete::fire-rules staged) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits    (:wat::rete::query fired (:probe::q-Hit))]
    (:wat::kernel::println
      (:wat::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
