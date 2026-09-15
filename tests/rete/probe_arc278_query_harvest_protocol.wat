;; Harvest is fire-time. Insert does not refresh query-memory.
;; fire → query=1 → insert more → query still 1 → fire → query=2.

(:wat::core::defrecord :qhp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :qhp::Hit  [c <- :wat::core::i64])

(:wat::rete::defrule :qhp::cool
  :when [(:qhp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))]
  :then [(:qhp::Hit ?c)])

(:wat::rete::defquery :qhp::q-Hit :params [] :when [(?f <- :qhp::Hit)])

(:wat::core::defn :user::protocol [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::core::match (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:qhp::cool))
                         (:wat::core::PersistentVector (:qhp::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                    s1 (:wat::core::match (:wat::rete::insert s0 (:qhp::Temp :c 10)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    f1 (:wat::core::match (:wat::rete::fire-rules s1) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
                    n1 (:wat::core::length (:wat::rete::query f1 (:qhp::q-Hit)))
                    s2 (:wat::core::match (:wat::rete::insert f1 (:qhp::Temp :c 15)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    n2 (:wat::core::length (:wat::rete::query s2 (:qhp::q-Hit)))
                    f2 (:wat::core::match (:wat::rete::fire-rules s2) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
                    n3 (:wat::core::length (:wat::rete::query f2 (:qhp::q-Hit)))]
    (:wat::core::PersistentVector n1 n2 n3)))
