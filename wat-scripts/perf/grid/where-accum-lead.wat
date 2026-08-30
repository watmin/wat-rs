;; wat-scripts/perf/grid/where-accum-lead.wat — leading accumulate (no left fact).
;; Twin of where-accum-lead.clj. Clara test-count:
;;   [?c <- (acc/count) :from [Reading]]
;; Empty world: count = 0 and it fires. Three facts: count = 3.
;; Leading max on empty: no token (None). Mid-chain accum stays where-accum-where.

(:wat::core::defrecord :wal::Reading [v <- :wat::core::i64])
(:wat::core::defrecord :wal::Busy    [n <- :wat::core::i64])

(:wat::rete::defrule :wal::count-zero
  :when [(?n <- (:wat::rete::acc::count) :from (:wal::Reading))
         (:wat::rete::where (:wat::rete::core::i64::= ?n 0))]
  :then [(:wal::Busy :n ?n)])

(:wat::rete::defrule :wal::count-three
  :when [(?n <- (:wat::rete::acc::count) :from (:wal::Reading))
         (:wat::rete::where (:wat::rete::core::i64::= ?n 3))]
  :then [(:wal::Busy :n ?n)])

(:wat::rete::defrule :wal::max-hi
  :when [(?m <- (:wat::rete::acc::max ?v) :from (:wal::Reading (?v <- :v)))
         (:wat::rete::where (:wat::rete::core::i64::> ?m 40))]
  :then [(:wal::Busy :n ?m)])

(:wat::rete::defquery :wal::q-Busy
  :params []
  :when [(?fact <- :wal::Busy)])


(:wat::core::defn :wal::n-busy [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wal::q-Busy))))

(:wat::core::defn :wal::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [z (:wat::core::PersistentVector (:wal::count-zero))
                    t (:wat::core::PersistentVector (:wal::count-three))
                    m (:wat::core::PersistentVector (:wal::max-hi))]
    (:wal::line 1 "count0-empty"
      (:wal::n-busy (:wat::core::match (:wat::rete::fire-rules (:wat::rete::compile-all z (:wat::core::PersistentVector (:wal::q-Busy)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wal::line 2 "count0-three"
      (:wal::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all z (:wat::core::PersistentVector (:wal::q-Busy)))
            (:wal::Reading :v 1) (:wal::Reading :v 2) (:wal::Reading :v 3)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wal::line 3 "count3-three"
      (:wal::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all t (:wat::core::PersistentVector (:wal::q-Busy)))
            (:wal::Reading :v 1) (:wal::Reading :v 2) (:wal::Reading :v 3)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wal::line 4 "count3-two"
      (:wal::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all t (:wat::core::PersistentVector (:wal::q-Busy)))
            (:wal::Reading :v 1) (:wal::Reading :v 2)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wal::line 5 "max-empty"
      (:wal::n-busy (:wat::core::match (:wat::rete::fire-rules (:wat::rete::compile-all m (:wat::core::PersistentVector (:wal::q-Busy)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wal::line 6 "max-50"
      (:wal::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all m (:wat::core::PersistentVector (:wal::q-Busy)))
            (:wal::Reading :v 50) (:wal::Reading :v 40)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
