;; wat-scripts/perf/grid/where-or-and.wat — condition `:and` inside `:or`.
;; Twin of where-or-and.clj. Clara test-disjunction-with-nested-and:
;;   [:or [Temp < 0] [:and [Temp < 20] [Wind > 30]]]
;; Really cold, or (cold AND windy). One Hit per loc.

(:wat::core::defrecord :woa::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :woa::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :woa::Hit  [loc <- :wat::core::String])

(:wat::rete::defrule :woa::really-cold-or-cold-and-windy
  :when [(:wat::rete::or
           (:woa::Temp (?loc <- :loc) (?c <- :c)
             (:wat::rete::core::i64::< ?c 0))
           (:wat::rete::and
             (:woa::Temp (?loc <- :loc) (?c <- :c)
               (:wat::rete::core::i64::< ?c 20))
             (:woa::Wind (?loc <- :loc) (?w <- :kph)
               (:wat::rete::core::i64::> ?w 30))))]
  :then [(:woa::Hit :loc ?loc)])

(:wat::rete::defquery :woa::q-Hit
  :params []
  :when [(?fact <- :woa::Hit)])


(:wat::core::defn :woa::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:woa::q-Hit))))

(:wat::core::defn :woa::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules (:wat::core::PersistentVector (:woa::really-cold-or-cold-and-windy))]
    (:woa::line 1 "really-cold"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Temp :c -10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woa::line 2 "cold-and-windy"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Temp :c 15 :loc "MCI")
            (:woa::Wind :kph 50 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woa::line 3 "mild-only"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Temp :c 15 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woa::line 4 "wind-only"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Wind :kph 50 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woa::line 5 "really-cold-and-windy"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Temp :c -10 :loc "MCI")
            (:woa::Wind :kph 50 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woa::line 6 "hot-and-windy"
      (:woa::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:woa::q-Hit)))
            (:woa::Temp :c 25 :loc "MCI")
            (:woa::Wind :kph 50 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
