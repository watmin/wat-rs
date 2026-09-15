;; fn-headed :then produces Hit; another rule exists that derived Hit.
;; rule_produces must list Hit, not the fn name, so exists raises.

(:wat::core::defrecord :fhp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :fhp::Hit  [c <- :wat::core::i64])
(:wat::core::defrecord :fhp::Seen [c <- :wat::core::i64])

(:wat::rete::core::defn :fhp::as-hit [c <- :wat::core::i64] -> :fhp::Hit
  (:fhp::Hit :c c))

(:wat::rete::defrule :fhp::cool
  :when [(:fhp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))]
  :then [(:fhp::as-hit ?c)])

(:wat::rete::defrule :fhp::seen
  :when [(:fhp::Temp (?c <- :c))
         (:wat::rete::exists (:fhp::Hit (?c <- :c)))]
  :then [(:fhp::Seen :c ?c)])

(:wat::rete::defquery :fhp::q-Hit  :params [] :when [(?f <- :fhp::Hit)])
(:wat::rete::defquery :fhp::q-Seen :params [] :when [(?f <- :fhp::Seen)])

(:wat::core::defn :user::source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::core::match (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:fhp::cool) (:fhp::seen))
                         (:wat::core::PersistentVector (:fhp::q-Hit) (:fhp::q-Seen))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                    fired (:wat::core::match (:wat::rete::fire-rules
                            (:wat::core::match (:wat::rete::insert s0 (:fhp::Temp :c 10)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:fhp::q-Hit)))
      (:wat::core::length (:wat::rete::query fired (:fhp::q-Seen))))))
