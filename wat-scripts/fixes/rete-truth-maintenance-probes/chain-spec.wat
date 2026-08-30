(:wat::core::defrecord :c::A [k <- :wat::core::i64])
(:wat::core::defrecord :c::B [k <- :wat::core::i64])
(:wat::core::defrecord :c::C [k <- :wat::core::i64])

;; R1: A → B (single input match)
(:wat::rete::defrule :c::r1
  :when [(:c::A (?k <- :k))]
  :then [(:c::B ?k)])

;; R2: B JOIN A (derived B joined with the ORIGINAL input A, same k) → C
(:wat::rete::defrule :c::r2
  :when [(:c::B (?k <- :k))
         (:c::A (?k <- :k))]
  :then [(:c::C ?k)])

(:wat::rete::defquery :c::q-A
  :params []
  :when [(:c::A)])


(:wat::rete::defquery :c::q-B
  :params []
  :when [(:c::B)])


(:wat::rete::defquery :c::q-C
  :params []
  :when [(:c::C)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s0    (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :c) (:wat::core::PersistentVector (:c::q-A) (:c::q-B) (:c::q-C))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    s1    (:wat::core::match (:wat::rete::insert s0 (:c::A 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    s2    (:wat::core::match (:wat::rete::insert s1 (:c::A 2)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    fired (:wat::core::match (:wat::rete::fire-rules$oracle s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "A (input)  = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-A))))))
      (:wat::kernel::println (:wat::core::string::concat "B (derived)= " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-B))))))
      (:wat::kernel::println (:wat::core::string::concat "C (expect 2)= " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-C)))))))))
