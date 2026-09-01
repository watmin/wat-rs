;; NON-VACUITY TWIN of `probe_arc278_import_accounting_ceiling.wat` — the byte-for-byte identical
;; program at the DEFAULT `max-session-bytes` (1 GiB), where a 15_172-byte import is nowhere near the
;; ceiling. It must import AND FIRE AND DERIVE, printing 1.
;;
;; Without this row, a ceiling of zero — or a check that refuses unconditionally, or one placed
;; before any work — satisfies every assertion its twin makes. The derived answer is here rather
;; than a bare "it did not die" because a session that imports and then cannot fire would still
;; print a happy word.
(:wat::core::defrecord :ia::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :ia::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :ia::q-Hit :params [] :when [(?fact <- :ia::Hit)])

(:wat::rete::defrule :ia::cool
  :when [(:ia::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:ia::Hit ?c)])

(:wat::core::defn :ia::compiled [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all
      (:wat::core::PersistentVector (:ia::cool))
      (:wat::core::PersistentVector (:ia::q-Hit)))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :ia::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:ia::Temp :c 10))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [e  (:wat::rete::export (:ia::compiled))
                    s1 (:wat::rete::import e)
                    s2 (:wat::core::match (:wat::rete::fire-rules (:ia::seed s1))
                         ((:wat::rete::FireOutcome::Fired __fired) __fired)
                         ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
                           (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
                         ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
                           (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println (:wat::core::length (:wat::rete::query s2 (:ia::q-Hit))))))
