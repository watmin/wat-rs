;; NEGATIVE FIXTURE — an UNTYPED inline alpha constraint using a generic ordering comparator.
;;
;; `(:wat::core::> :value 10)` sits inside a fact pattern. Freeze wall
;; (`validate.rs` CoreGeneric → NonReteConstraint) and intern
;; `compile_condition_local` refuse it. Law A sees this spelling.
;; See DESIGN-STONE-inline-constraint-admits-non-rete.md.

(:wat::core::defrecord :probe::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :probe::Hot     [location <- :wat::core::String])

(:wat::rete::defrule :probe::untyped-ordering
  :when
  [(:probe::Reading (?loc :- :location) (:wat::core::> :value 10))]
  :then
  [(:probe::Hot :location ?loc)])

(:wat::rete::defquery :probe::q-Hot
  :params []
  :when [(?fact :- :probe::Hot)])


(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hot))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     session (:wat::core::match (:wat::rete::insert session (:probe::Reading :location "Oslo"   :value 42)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     session (:wat::core::match (:wat::rete::insert session (:probe::Reading :location "Bergen" :value 3)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (:wat::core::length (:wat::rete::query fired (:probe::q-Hot)))))
