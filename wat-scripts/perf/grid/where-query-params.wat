;; wat-scripts/perf/grid/where-query-params.wat — Clara parametric defquery.
;; Twin of where-query-params.clj. Clara test-count-some-empty:
;;   (defquery q [:?loc] [[?n <- (acc/count) :from [Temp (= ?loc loc)]]
;;                        [Wind (> kph 10) (= ?loc loc)]])
;;   (query s q :?loc "MCI") → [{?n 0, ?loc MCI}] when MCI has wind and no temps.
;; Readout is `query`, never query-by-type-string.

(:wat::core::defrecord :wqp::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wqp::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wqp::Hit  [loc <- :wat::core::String])

(:wat::rete::defrule :wqp::mark
  :when [(:wqp::Wind (?loc <- :loc) (?w <- :kph)
           (:wat::rete::core::i64::> ?w 10))]
  :then [(:wqp::Hit :loc ?loc)])

(:wat::rete::defquery :wqp::temps-at
  :params [?loc]
  :when [(?n <- (:wat::rete::acc::count) :from (:wqp::Temp (?loc <- :loc)))
         (:wqp::Wind (?loc <- :loc) (?w <- :kph)
           (:wat::rete::core::i64::> ?w 10))])

(:wat::rete::defquery :wqp::all-wind
  :params []
  :when [(:wqp::Wind (?loc <- :loc))])

(:wat::rete::defquery :wqp::hits
  :params []
  :when [(:wqp::Hit (?loc <- :loc))])

(:wat::core::defn :wqp::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules   (:wat::core::PersistentVector (:wqp::mark))
                    queries (:wat::core::PersistentVector
                              (:wqp::temps-at) (:wqp::all-wind) (:wqp::hits))
                    world (:wat::core::match (:wat::rete::fire-rules
                            (:wat::core::match (:wat::rete::insert
                              (:wat::rete::compile-all rules queries)
                              (:wqp::Wind :kph 20 :loc "MCI")
                              (:wqp::Wind :kph 20 :loc "SFO")
                              (:wqp::Temp :c 40 :loc "SFO")
                              (:wqp::Temp :c 50 :loc "SFO")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    empty (:wat::core::match (:wat::rete::fire-rules
                            (:wat::rete::compile-all rules queries)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wqp::line 1 "hits"
      (:wat::core::length (:wat::rete::query world (:wqp::hits))))
    (:wqp::line 2 "mci-empty-temps"
      (:wat::core::length (:wat::rete::query world (:wqp::temps-at) :?loc "MCI")))
    (:wqp::line 3 "sfo-two-temps"
      (:wat::core::length (:wat::rete::query world (:wqp::temps-at) :?loc "SFO")))
    (:wqp::line 4 "missing-loc"
      (:wat::core::length (:wat::rete::query world (:wqp::temps-at) :?loc "XXX")))
    (:wqp::line 5 "all-wind"
      (:wat::core::length (:wat::rete::query world (:wqp::all-wind))))
    (:wqp::line 6 "empty-world"
      (:wat::core::length (:wat::rete::query empty (:wqp::temps-at) :?loc "MCI")))))
