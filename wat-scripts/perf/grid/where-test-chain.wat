;; wat-scripts/perf/grid/where-test-chain.wat — Test→Join→Test→Test (Clara test-simple-test).
;;
;; Twin of where-test-chain.clj. Spoken order used to derive nothing on native
;; (3.7 walked Test children of a HashJoin, not the next Test). Joins-first was
;; already green. 1↔2 must print the same n= / pair. Clara 0.24.0 does.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-test-chain.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-test-chain.clj

(:wat::core::defn :wtc::row-count [] -> :wat::core::i64 2)

(:wat::core::defrecord :wtc::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wtc::Pair [a <- :wat::core::i64 b <- :wat::core::i64])

;; ROW 1 — spoken: filter, join, filter, filter. t1<20, t2<20, t1<t2.
;; Facts 15/10/80 at MCI → one pair {10 15}.
(:wat::rete::defrule :wtc::spoken
  :when
  [(:wtc::Temp (?t1 <- :c) (?loc <- :loc))
   (:wat::rete::where (:wat::rete::core::i64::< ?t1 20))
   (:wtc::Temp (?t2 <- :c) (?loc <- :loc))
   (:wat::rete::where (:wat::rete::core::i64::< ?t2 20))
   (:wat::rete::where (:wat::rete::core::i64::< ?t1 ?t2))]
  :then
  [(:wtc::Pair :a ?t1 :b ?t2)])

;; ROW 2 — joins first, then the three filters. Same set as row 1.
(:wat::rete::defrule :wtc::join-first
  :when
  [(:wtc::Temp (?t1 <- :c) (?loc <- :loc))
   (:wtc::Temp (?t2 <- :c) (?loc <- :loc))
   (:wat::rete::where (:wat::rete::core::i64::< ?t1 20))
   (:wat::rete::where (:wat::rete::core::i64::< ?t2 20))
   (:wat::rete::where (:wat::rete::core::i64::< ?t1 ?t2))]
  :then
  [(:wtc::Pair :a ?t1 :b ?t2)])

(:wat::rete::defquery :wtc::q-Pair
  :params []
  :when [(:wtc::Pair (?a <- :a) (?b <- :b))])


(:wat::core::defn :wtc::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wtc::spoken))
      ((:wat::core::= row 2) (:wtc::join-first))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-test-chain: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

(:wat::core::defn :wtc::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert session
    (:wtc::Temp :c 15 :loc "MCI")
    (:wtc::Temp :c 10 :loc "MCI")
    (:wtc::Temp :c 80 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :wtc::render [fired <- :wat::rete::Session] -> :wat::core::String
  (:wat::core::let [pairs (:wat::rete::query fired (:wtc::q-Pair))
                    n     (:wat::core::length pairs)
                    shown (:wat::core::foldl
                             (:wat::core::fn [acc <- :wat::core::String  p <- :wat::core::PersistentMap]
                               -> :wat::core::String
                               (:wat::core::String/concat acc
                                 (:wat::core::String/concat " "
                                   (:wat::core::String/concat
                                     (:wat::core::i64::to-string
                                       (:wat::core::Option/expect
                                         (:wat::core::PersistentMap/get p "?a")
                                         "q-Pair: ?a"))
                                     (:wat::core::String/concat ","
                                       (:wat::core::i64::to-string
                                         (:wat::core::Option/expect
                                           (:wat::core::PersistentMap/get p "?b")
                                           "q-Pair: ?b")))))))
                             ""
                             pairs)]
    (:wat::core::String/concat
      (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
      (:wat::core::String/concat " ->" shown))))

(:wat::core::defn :wtc::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules (:wtc::build-rules row)
                    rule  (:wat::core::first rules)
                    fired (:wat::core::match (:wat::rete::fire-rules (:wtc::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wtc::q-Pair))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    name  (:wat::core::foldl
                             (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String]
                               -> :wat::core::String seg)
                             (:wat::rete::Rule/name rule)
                             (:wat::core::string::split (:wat::rete::Rule/name rule) "::"))]
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat (:wat::core::String/concat " " name) (:wtc::render fired)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wtc::run-row row)))
    nil
    (:wat::core::range 1 3)))
