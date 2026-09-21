;; MEASUREMENT — a computed head whose field type has a domain of TWO.
;; `flag <- bool`, `:then` derives `(not ?flag)`. The fact domain is {F(true), F(false)}.
;; It CANNOT diverge: two facts is the whole universe. Does the verifier refuse it anyway?
(:wat::core::defrecord :fd::F [flag <- :wat::core::bool])

(:wat::rete::defrule :fd::flip
  :when  [(:fd::F (?b :- :flag))]
  :then  [(:fd::F :flag (:wat::rete::core::not ?b))])

(:wat::rete::defquery :fd::q :params [] :when [(?fact :- :fd::F)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :fd)
         session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fd::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
         session (:wat::core::match (:wat::rete::insert session (:fd::F :flag true)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
        (:wat::rete::query fired (:fd::q))))))
