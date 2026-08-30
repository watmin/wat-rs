;; probe-seed-insert-vs-insert-all.wat — arc 278, the SEEDING surface.
;;
;; WHY: the Clara grid times ONLY `fire`. At fanout [40000] the fire is ~46 ms of a ~5.3 s wall
;; clock — 0.9%. The other ~96% is SEEDING, and every one of the nine grid axes seeds with the
;; per-fact verb `:wat::rete::insert` inside an interpreted `foldl`, while the native batch verb
;; `:wat::rete::insert-all` (delegating to `insert-all'`) has been on the disk the whole time.
;; `insert-all`'s own doc names the defect: "ONE rebuild — not N rebuilds (`insert` × N)".
;;
;; This probe MEASURES the two seeding paths against each other on identical facts. It asserts
;; nothing about which is faster; it prints both so the disk decides.
;;
;; stdin = [n]  (n = how many Left facts to stage)
;; stdout = one #probe/SeedTimes EDN line.

(:wat::core::defrecord :seedp::Left [key <- :wat::core::i64  lid <- :wat::core::i64])

(:wat::core::defrecord :probe::SeedTimes
  [n              <- :wat::core::i64
   per-fact-ns    <- :wat::core::i64
   batch-ns       <- :wat::core::i64
   per-fact-facts <- :wat::core::i64
   batch-facts    <- :wat::core::i64])

(:wat::core::defn :seedp::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; an empty session — one rule so `compile` has something to chew, never fired here.
(:wat::core::defn :seedp::fresh [] -> :wat::rete::Session
  (:wat::rete::compile
    (:wat::core::PersistentVector
      (:wat::rete::Rule
        :name "noop"
        :lhs (:wat::core::PersistentVector (:wat::core::quote (:seedp::Left (?k <- :key))))
        :rhs (:wat::core::PersistentVector)))))

;; PATH A — the grid's current shape: N calls to the per-fact verb, threaded through a foldl.
(:wat::core::defn :seedp::seed-per-fact [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::match (:wat::rete::insert acc (:seedp::Left :key i :lid i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    s
    (:wat::core::range 0 n)))

;; PATH B — build the fact vector, then ONE call to the native batch verb.
(:wat::core::defn :seedp::seed-batch [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    s
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:seedp::Left :key i :lid i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 n))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [params (:wat::core::match (:wat::kernel::readln )
              ((:wat::kernel::ReadlnOutcome::Datum __d) __d)
              (:wat::kernel::ReadlnOutcome::Eof     (:wat::kernel::assertion-failed! "readln: eof"  :wat::core::None :wat::core::None))
              (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop" :wat::core::None :wat::core::None)))
     n   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")
     a0  (:wat::time::now)
     sa  (:seedp::seed-per-fact (:seedp::fresh) n)
     a1  (:wat::time::now)
     b0  (:wat::time::now)
     sb  (:seedp::seed-batch (:seedp::fresh) n)
     b1  (:wat::time::now)]
    (:wat::kernel::println
      (:probe::SeedTimes
        :n              n
        :per-fact-ns    (:seedp::ns-between a0 a1)
        :batch-ns       (:seedp::ns-between b0 b1)
        ;; both paths must stage the SAME number of facts — a faster path that stages fewer
        ;; is not faster, it is wrong. This is the non-vacuity guard on the comparison.
        :per-fact-facts (:wat::core::PersistentVector/length (:wat::rete::Session/facts sa))
        :batch-facts    (:wat::core::PersistentVector/length (:wat::rete::Session/facts sb))))))
