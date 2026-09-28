;; wat-scripts/perf/grid/where-query-compat.wat — query-mouth compat.
;; Twin of where-query-compat.clj. Subject is `query` answers (binding maps),
;; not a Hit-count readout. Three-way: Clara | wat-oracle | wat-native.
;;
;;     JAVA_HOME=$HOME/opt/jdk-21.0.12+8 \
;;       bash wat-scripts/perf/grid/check-query-compat.sh
;;
;; Rows print sorted scalar bindings. A bound record prints as presence
;; (`has=?t`), never as EDN (Clara and wat write records differently).

(:wat::core::defrecord :wqc::Temp [c <- wat.type/i64 loc <- wat.type/String])
(:wat::core::defrecord :wqc::Wind [kph <- wat.type/i64 loc <- wat.type/String])
(:wat::core::defrecord :wqc::Hit  [loc <- wat.type/String])

(:wat::rete::defrule :wqc::mark
  :when [(:wqc::Wind (?loc :- :loc) (?w :- :kph)
           (:wat::rete::i64::> ?w 10))]
  :then [(:wqc::Hit :loc ?loc)])

(:wat::rete::defquery :wqc::q-fields
  :params []
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))])

(:wat::rete::defquery :wqc::q-plain
  :params []
  :when [(?fact :- :wqc::Temp)])

(:wat::rete::defquery :wqc::q-bound
  :params []
  :when [(?t :- :wqc::Temp (?c :- :c))])

(:wat::rete::defquery :wqc::q-at
  :params [?loc]
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))])

(:wat::rete::defquery :wqc::q-join
  :params []
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))
         (:wqc::Wind (?w :- :kph) (?loc :- :loc))])

(:wat::rete::defquery :wqc::q-count-at
  :params [?loc]
  :when [(?n :- (:wat::rete::acc::count) :from (:wqc::Temp (?loc :- :loc)))])

(:wat::rete::defquery :wqc::q-count-wind
  :params [?loc]
  :when [(?n :- (:wat::rete::acc::count) :from (:wqc::Temp (?loc :- :loc)))
         (:wqc::Wind (?loc :- :loc))])

(:wat::rete::defquery :wqc::q-no-wind
  :params []
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))
         (:wat::rete::not (:wqc::Wind (?loc :- :loc)))])

(:wat::rete::defquery :wqc::q-has-wind
  :params []
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))
         (:wat::rete::exists (:wqc::Wind (?loc :- :loc)))])

(:wat::rete::defquery :wqc::q-cool
  :params []
  :when [(:wqc::Temp (?c :- :c) (?loc :- :loc))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))])

(:wat::rete::defquery :wqc::q-Hit
  :params []
  :when [(:wqc::Hit (?loc :- :loc))])

(:wat::core::defn :wqc::has-key
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])
   k       <- wat.type/String]
  -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::length answers) 0)
    "empty"
    (:wat::core::match
      (:wat::map::get (:wat::core::first answers) k)
      [:wat::core::Option.Some {:value _} "yes"]
      [:wat::core::Option.None {} "none"])))

(:wat::core::defn :wqc::i64-of
  [p <- wat.type/PersistentMap k <- wat.type/String] -> wat.type/i64
  (:wat::core::Option/expect (:wat::map::get p k)
    (:wat::string::concat "query-compat missing " k)))

(:wat::core::defn :wqc::str-of
  [p <- wat.type/PersistentMap k <- wat.type/String] -> wat.type/String
  (:wat::core::Option/expect (:wat::map::get p k)
    (:wat::string::concat "query-compat missing " k)))

(:wat::core::defn :wqc::render-strs
  [v <- (wat.type/Vector :- [wat.type/String])] -> wat.type/String
  (:wat::core::foldl
    (:wat::core::fn [acc <- wat.type/String s <- wat.type/String] -> wat.type/String
      (:wat::string::concat acc (:wat::string::concat " " s)))
    ""
    v))

(:wat::core::defn :wqc::pairs-c-loc
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])]
  -> wat.type/String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (wat.type/Vector :- [wat.type/String])
        (:wat::core::map
          (:wat::core::fn [p <- wat.type/PersistentMap] -> wat.type/String
            (:wat::string::concat
              (:wat::i64::to-string (:wqc::i64-of p "?c"))
              (:wat::string::concat "," (:wqc::str-of p "?loc"))))
          answers)))))

(:wat::core::defn :wqc::pairs-join
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])]
  -> wat.type/String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (wat.type/Vector :- [wat.type/String])
        (:wat::core::map
          (:wat::core::fn [p <- wat.type/PersistentMap] -> wat.type/String
            (:wat::string::concat
              (:wat::i64::to-string (:wqc::i64-of p "?c"))
              (:wat::string::concat ","
                (:wat::string::concat
                  (:wat::i64::to-string (:wqc::i64-of p "?w"))
                  (:wat::string::concat "," (:wqc::str-of p "?loc"))))))
          answers)))))

(:wat::core::defn :wqc::vals-c
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])]
  -> wat.type/String
  (:wqc::render-strs
    (:wat::core::into (wat.type/Vector :- [wat.type/String])
      (:wat::core::map
        (:wat::core::fn [x <- wat.type/i64] -> wat.type/String
          (:wat::i64::to-string x))
        (:wat::core::sort
          (:wat::core::into (wat.type/Vector :- [wat.type/i64])
            (:wat::core::map
              (:wat::core::fn [p <- wat.type/PersistentMap] -> wat.type/i64
                (:wqc::i64-of p "?c"))
              answers)))))))

(:wat::core::defn :wqc::vals-loc
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])]
  -> wat.type/String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (wat.type/Vector :- [wat.type/String])
        (:wat::core::map
          (:wat::core::fn [p <- wat.type/PersistentMap] -> wat.type/String
            (:wqc::str-of p "?loc"))
          answers)))))

(:wat::core::defn :wqc::one-n
  [answers <- (wat.type/PersistentVector :- [wat.type/PersistentMap])]
  -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::length answers) 0)
    ""
    (:wat::string::concat " "
      (:wat::i64::to-string (:wqc::i64-of (:wat::core::first answers) "?n")))))

(:wat::core::defn :wqc::line
  [row <- wat.type/i64 name <- wat.type/String body <- wat.type/String]
  -> wat.type/nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat (:wat::string::concat " " name) body))))

(:wat::core::defn :wqc::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert session
    (:wqc::Temp :c 15 :loc "MCI")
    (:wqc::Temp :c 80 :loc "MCI")
    (:wqc::Temp :c 40 :loc "SFO")
    (:wqc::Temp :c 10 :loc "ORD")
    (:wqc::Wind :kph 20 :loc "MCI")
    (:wqc::Wind :kph 5  :loc "SFO")
    (:wqc::Wind :kph 20 :loc "LAX")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [qs (:wat::core::PersistentVector
          (:wqc::q-fields) (:wqc::q-plain) (:wqc::q-bound) (:wqc::q-at)
          (:wqc::q-join) (:wqc::q-count-at) (:wqc::q-count-wind)
          (:wqc::q-no-wind) (:wqc::q-has-wind) (:wqc::q-cool))
     world (:wat::core::match (:wat::rete::fire-rules
             (:wqc::seed (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector) qs) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")]))) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     fields (:wat::rete::query world (:wqc::q-fields))
     plain  (:wat::rete::query world (:wqc::q-plain))
     bound  (:wat::rete::query world (:wqc::q-bound))
     at-mci (:wat::rete::query world (:wqc::q-at) :?loc "MCI")
     at-xxx (:wat::rete::query world (:wqc::q-at) :?loc "XXX")
     join   (:wat::rete::query world (:wqc::q-join))
     n-mci  (:wat::rete::query world (:wqc::q-count-at) :?loc "MCI")
     n-lax  (:wat::rete::query world (:wqc::q-count-wind) :?loc "LAX")
     none   (:wat::rete::query world (:wqc::q-no-wind))
     some   (:wat::rete::query world (:wqc::q-has-wind))
     cool   (:wat::rete::query world (:wqc::q-cool))
     hits   (:wat::rete::query
              (:wat::core::match (:wat::rete::fire-rules
                (:wqc::seed
                  (:wat::core::match (:wat::rete::compile-all
                    (:wat::core::PersistentVector (:wqc::mark))
                    (:wat::core::PersistentVector (:wqc::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")]))) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
              (:wqc::q-Hit))
     empty  (:wat::rete::query
              (:wat::core::match (:wat::rete::fire-rules
                (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector) qs) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
              (:wqc::q-fields))]
    (:wqc::line 1 "fields"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length fields)))
        (:wat::string::concat " ->" (:wqc::pairs-c-loc fields))))
    (:wqc::line 2 "plain"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length plain)))
        (:wat::string::concat " has=?c " (:wqc::has-key plain "?c"))))
    (:wqc::line 3 "fact-bind"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length bound)))
        (:wat::string::concat
          (:wat::string::concat " has=?t " (:wqc::has-key bound "?t"))
          (:wat::string::concat " ->" (:wqc::vals-c bound)))))
    (:wqc::line 4 "params-mci"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length at-mci)))
        (:wat::string::concat " ->" (:wqc::pairs-c-loc at-mci))))
    (:wqc::line 5 "params-xxx"
      (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length at-xxx))))
    (:wqc::line 6 "join"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length join)))
        (:wat::string::concat " ->" (:wqc::pairs-join join))))
    (:wqc::line 7 "count-mci"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length n-mci)))
        (:wat::string::concat " ->" (:wqc::one-n n-mci))))
    (:wqc::line 8 "count-zero"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length n-lax)))
        (:wat::string::concat " ->" (:wqc::one-n n-lax))))
    (:wqc::line 9 "not-wind"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length none)))
        (:wat::string::concat " ->" (:wqc::pairs-c-loc none))))
    (:wqc::line 10 "exists-wind"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length some)))
        (:wat::string::concat " ->" (:wqc::pairs-c-loc some))))
    (:wqc::line 11 "where-cool"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length cool)))
        (:wat::string::concat " ->" (:wqc::pairs-c-loc cool))))
    (:wqc::line 12 "derived"
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length hits)))
        (:wat::string::concat " ->" (:wqc::vals-loc hits))))
    (:wqc::line 13 "empty"
      (:wat::string::concat " n=" (:wat::i64::to-string (:wat::core::length empty))))))
