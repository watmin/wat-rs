(:wat::core::defrecord :c::A [k <- :wat::core::i64])
(:wat::core::defrecord :c::B [k <- :wat::core::i64])
(:wat::core::defrecord :c::C [k <- :wat::core::i64])

;; R1: A → B (single input match)
(:wat::rete::defrule :c::r1
  :when [(:c::A (?k :- :k))]
  :then [(:c::B ?k)])

;; R2: B JOIN A (derived B joined with the ORIGINAL input A, same k) → C
(:wat::rete::defrule :c::r2
  :when [(:c::B (?k :- :k))
         (:c::A (?k :- :k))]
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
  (:wat::core::let [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :c) (:wat::core::PersistentVector (:c::q-A) (:c::q-B) (:c::q-C))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                    s1 (:wat::core::match (:wat::rete::insert s0 (:c::A 1)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    s2 (:wat::core::match (:wat::rete::insert s1 (:c::A 2)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    fired (:wat::rete::fire-fixpoint s2)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "A (input, queried) = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-A))))))
      (:wat::kernel::println (:wat::string::concat "B (derived)        = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-B))))))
      (:wat::kernel::println (:wat::string::concat "C (B join A)       = " (:wat::core::str (:wat::core::length (:wat::rete::query fired (:c::q-C)))))))))
