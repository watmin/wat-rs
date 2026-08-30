(:wat::core::defrecord :n::A [k <- :wat::core::i64])
(:wat::core::defrecord :n::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :n::Ok [k <- :wat::core::i64])
;; derive Bad for k=2 only
(:wat::rete::defrule :n::mark-bad
  :when [(:n::A (?k <- :k)) (:wat::rete::where (:wat::core::= ?k 2))]
  :then [(:n::Bad ?k)])
;; Ok = A with NO Bad (negation over a DERIVED fact)
(:wat::rete::defrule :n::ok
  :when [(:n::A (?k <- :k)) (:wat::rete::not (:n::Bad (?k <- :k)))]
  :then [(:n::Ok ?k)])

(:wat::rete::defquery :n::q-Bad
  :params []
  :when [(:n::Bad)])


(:wat::rete::defquery :n::q-Ok
  :params []
  :when [(:n::Ok)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s0    (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :n) (:wat::core::PersistentVector (:n::q-Bad) (:n::q-Ok))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    s1    (:wat::core::match (:wat::rete::insert s0 (:n::A 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    s2    (:wat::core::match (:wat::rete::insert s1 (:n::A 2)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    fired (:wat::core::match (:wat::rete::fire-rules$oracle s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "Bad (expect 1) = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:n::q-Bad))))))
      (:wat::kernel::println (:wat::core::string::concat "Ok  (expect 1, k=1) = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:n::q-Ok)))))))))
