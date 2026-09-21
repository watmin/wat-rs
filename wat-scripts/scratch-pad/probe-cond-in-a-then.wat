;; NOTE-rete-cond-lowers-on-the-lhs-but-not-the-rhs.md (2026-08-24, inbound).
;; `cond` works in a `where` and was claimed to fail at compile-all in a `:then`.
(:wat::core::defrecord :cr::In  [n <- :wat::core::i64])
(:wat::core::defrecord :cr::Out [label <- :wat::core::String])

(:wat::rete::defrule :cr::rule
  :when
  [(:cr::In (?n :- :n))
   (:wat::rete::where (:wat::rete::core::cond ((:wat::rete::i64::> ?n 5) true) (:else false)))]
  :then
  [(:cr::Out :label (:wat::rete::core::cond ((:wat::rete::i64::> ?n 100) "big") (:else "small")))])

(:wat::rete::defquery :cr::q :params [] :when [(?fact :- :cr::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :cr)
         session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cr::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
         session (:wat::core::match (:wat::rete::insert session (:cr::In :n 10)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
        (:wat::rete::query fired (:cr::q))))))
