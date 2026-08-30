;; wat-scripts/perf/grid/where-not-fact.wat — leading :not of one fact.
;; Twin of where-not-fact.clj. Clara test-simple-negation:
;;   [:not [Temp < 20]]
;; Empty world matches. A cold Temp kills it. Retract restores.

(:wat::core::defrecord :wnf::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnf::Hit  [k <- :wat::core::i64])

(:wat::rete::defrule :wnf::not-cold
  :when [(:wat::rete::not
           (:wnf::Temp (?c <- :c)
             (:wat::rete::core::i64::< ?c 20)))]
  :then [(:wnf::Hit :k 1)])

(:wat::rete::defquery :wnf::q-Hit
  :params []
  :when [(?fact <- :wnf::Hit)])


(:wat::core::defn :wnf::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnf::q-Hit))))

(:wat::core::defn :wnf::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules (:wat::core::PersistentVector (:wnf::not-cold))]
    (:wnf::line 1 "empty"
      (:wnf::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnf::q-Hit)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnf::line 2 "cold"
      (:wnf::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnf::q-Hit)))
            (:wnf::Temp :c 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnf::line 3 "hot"
      (:wnf::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnf::q-Hit)))
            (:wnf::Temp :c 80 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnf::line 4 "retract-cold"
      (:wnf::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::retract
            (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnf::q-Hit)))
              (:wnf::Temp :c 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
            (:wnf::Temp :c 10 :loc "MCI"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnf::line 5 "partial-retract"
      (:wnf::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::retract
            (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnf::q-Hit)))
              (:wnf::Temp :c 10 :loc "MCI")
              (:wnf::Temp :c 15 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
            (:wnf::Temp :c 10 :loc "MCI"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
