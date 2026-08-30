;; wat-scripts/perf/grid/where-not-where.wat — :not of :where (Clara test-in-negation).
;; Twin of where-not-where.clj. Clara test-test-in-negation:
;;   [Temp ?a] [Wind ?b] [:not [:test (= ?a ?b)]]
;; Same loc → 0. Different loc → 1.

(:wat::core::defrecord :wnw::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnw::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnw::Hit  [a <- :wat::core::String b <- :wat::core::String])

(:wat::rete::defrule :wnw::not-same-loc
  :when [(:wnw::Temp (?a <- :loc))
         (:wnw::Wind (?b <- :loc))
         (:wat::rete::not
           (:wat::rete::where (:wat::rete::core::string::= ?a ?b)))]
  :then [(:wnw::Hit :a ?a :b ?b)])

(:wat::rete::defquery :wnw::q-Hit
  :params []
  :when [(?fact <- :wnw::Hit)])


(:wat::core::defn :wnw::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnw::q-Hit))))

(:wat::core::defn :wnw::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules (:wat::core::PersistentVector (:wnw::not-same-loc))]
    (:wnw::line 1 "same-loc"
      (:wnw::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnw::q-Hit)))
            (:wnw::Temp :c 10 :loc "MCI")
            (:wnw::Wind :kph 10 :loc "MCI"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnw::line 2 "diff-loc"
      (:wnw::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnw::q-Hit)))
            (:wnw::Temp :c 10 :loc "MCI")
            (:wnw::Wind :kph 10 :loc "ORD"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnw::line 3 "temp-only"
      (:wnw::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnw::q-Hit)))
            (:wnw::Temp :c 10 :loc "MCI"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wnw::line 4 "two-diff"
      (:wnw::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnw::q-Hit)))
            (:wnw::Temp :c 10 :loc "MCI")
            (:wnw::Temp :c 12 :loc "ORD")
            (:wnw::Wind :kph 10 :loc "DFW")
            (:wnw::Wind :kph 20 :loc "SEA"))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
