;; wat-scripts/perf/grid/where-accum-from-left.wat — leftover on accumulate :from.
;; Twin of where-accum-from-left.clj. Clara 0.24.0:
;;   [Temp (= ?loc loc) (= ?c c)]
;;   [?n <- (acc/count) :from [Wind (= ?loc loc) (> kph ?c)]]
;; Empty :from still fires with count 0. Field form (no extra ?w bind) so
;; the count is not grouped. n= is the SUM of Hit.n (two-locs has two Hits).

(:wat::core::defrecord :wafl::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wafl::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wafl::Hit  [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :wafl::count-winds-above-temp
  :when
  [(:wafl::Temp (?loc <- :loc) (?c <- :c))
   (?n <- (:wat::rete::acc::count) :from
     (:wafl::Wind (?loc <- :loc)
       (:wat::rete::core::i64::> :kph ?c)))]
  :then
  [(:wafl::Hit :loc ?loc :n ?n)])

(:wat::rete::defquery :wafl::q-Hit
  :params []
  :when [(?fact <- :wafl::Hit)])

(:wat::core::defn :wafl::sum-n [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64
                     p   <- :wat::core::PersistentMap]
      -> :wat::core::i64
      (:wat::core::let [f (:wat::core::Option/expect
                             (:wat::core::PersistentMap/get p "?fact")
                             "query: ?fact")]
        (:wat::core::i64::+ acc (:wafl::Hit/n f))))
    0
    (:wat::rete::query s (:wafl::q-Hit))))

(:wat::core::defn :wafl::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [base (:wat::core::match (:wat::rete::compile-all
                           (:wat::core::PersistentVector (:wafl::count-winds-above-temp))
                           (:wat::core::PersistentVector (:wafl::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))]
    (:wafl::line 1 "empty"
      (:wafl::sum-n (:wat::core::match (:wat::rete::fire-rules base) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 2 "temp-only"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base (:wafl::Temp :c 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 3 "below"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base
            (:wafl::Temp :c 10 :loc "MCI")
            (:wafl::Wind :kph 5 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 4 "above"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base
            (:wafl::Temp :c 10 :loc "MCI")
            (:wafl::Wind :kph 20 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 5 "equal"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base
            (:wafl::Temp :c 10 :loc "MCI")
            (:wafl::Wind :kph 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 6 "two-of-three"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base
            (:wafl::Temp :c 10 :loc "MCI")
            (:wafl::Wind :kph 5 :loc "MCI")
            (:wafl::Wind :kph 20 :loc "MCI")
            (:wafl::Wind :kph 30 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wafl::line 7 "two-locs"
      (:wafl::sum-n
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert base
            (:wafl::Temp :c 10 :loc "MCI")
            (:wafl::Wind :kph 20 :loc "MCI")
            (:wafl::Temp :c 10 :loc "ORD")
            (:wafl::Wind :kph 5 :loc "ORD")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
