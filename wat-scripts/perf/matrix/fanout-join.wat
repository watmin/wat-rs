;; matrix dim: SELECTIVITY / FAN-OUT. One join, low selectivity: F Lefts × F Rights per key share the key →
;; F² joined Pairs per key, K keys → K·F² derived. High F = token explosion (the classic RETE join stress).
;; stdin = [keys fanout] EDN; stdout = #perf/Result record (println ∀T→EDN). Times native vs wat-spec.
(:wat::core::defrecord :fan::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])
(:wat::core::defrecord :fan::Right [key <- :wat::core::i64  rid <- :wat::core::i64])
(:wat::core::defrecord :fan::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])
(:wat::core::defrecord :perf::FanResult
  [keys <- :wat::core::i64 fanout <- :wat::core::i64 pairs <- :wat::core::i64 native-ns <- :wat::core::i64])

(:wat::rete::defquery :fan::q-Pair
  :params []
  :when [(?fact <- :fan::Pair)])


;; seed Left(k,f)+Right(k,f) for f in 0..fanout, threaded onto session s, for one key k.
(:wat::core::defn :fan::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert acc (:fan::Left :key k :lid f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) (:fan::Right :key k :rid f)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    s
    (:wat::core::range 0 fanout)))

(:fan::Pair :key 0 :lid 0 :rid 0)  ;; touch ctor (unused warning guard; harmless)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    keys   (:wat::core::Option/expect   (:wat::core::get params 0) "[keys fanout]")
                    fanout (:wat::core::Option/expect   (:wat::core::get params 1) "[keys fanout]")
                    c1   (:wat::core::quote (:fan::Left  (?k <- :key) (?l <- :lid)))
                    c2   (:wat::core::quote (:fan::Right (?k <- :key) (?r <- :rid)))
                    rhs  (:wat::core::quote (:fan::Pair ?k ?l ?r))
                    rule (:wat::rete::Rule :name "fan" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs))
                    s0   (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:fan::q-Pair)))
                    staged (:wat::core::foldl
                              (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
                                (:fan::seed-key acc k fanout))
                              s0
                              (:wat::core::range 0 keys))
                    n0 (:wat::time::now)  fn (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))       n1 (:wat::time::now)
                    pairs   (:wat::core::length (:wat::rete::query fn (:fan::q-Pair)))
                    nat-ns  (:wat::core::i64::- (:wat::time::epoch-nanos n1) (:wat::time::epoch-nanos n0))]
    (:wat::kernel::println (:perf::FanResult :keys keys :fanout fanout :pairs pairs :native-ns nat-ns))))
