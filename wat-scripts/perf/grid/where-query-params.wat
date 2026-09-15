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
           (:wat::rete::i64::> ?w 10))]
  :then [(:wqp::Hit :loc ?loc)])

(:wat::rete::defquery :wqp::temps-at
  :params [?loc]
  :when [(?n <- (:wat::rete::acc::count) :from (:wqp::Temp (?loc <- :loc)))
         (:wqp::Wind (?loc <- :loc) (?w <- :kph)
           (:wat::rete::i64::> ?w 10))])

(:wat::rete::defquery :wqp::all-wind
  :params []
  :when [(:wqp::Wind (?loc <- :loc))])

(:wat::rete::defquery :wqp::hits
  :params []
  :when [(:wqp::Hit (?loc <- :loc))])

(:wat::core::defn :wqp::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat
        (:wat::string::concat " " name)
        (:wat::string::concat " n=" (:wat::i64::to-string n))))))

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
                              (:wqp::Temp :c 50 :loc "SFO")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
                    empty (:wat::core::match (:wat::rete::fire-rules
                            (:wat::rete::compile-all rules queries)) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
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
