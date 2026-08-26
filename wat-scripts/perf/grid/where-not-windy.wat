;; wat-scripts/perf/grid/where-not-windy.wat — windy and not a cold Temp (any loc).
;; Twin of where-not-windy.clj. Clara test-negation-with-other-conditions.
;; A cold Temp anywhere kills every windy loc.

(:wat::core::defrecord :wnwdy::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnwdy::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnwdy::Hit  [loc <- :wat::core::String])

(:wat::rete::defrule :wnwdy::windy-not-cold
  :when [(:wnwdy::Wind (?loc <- :loc) (?w <- :kph))
         (:wat::rete::where (:wat::rete::core::i64::> ?w 30))
         (:wat::rete::not (:wnwdy::Temp (?c <- :c)
                            (:wat::rete::core::i64::< ?c 20)))]
  :then [(:wnwdy::Hit :loc ?loc)])

(:wat::rete::defquery :wnwdy::q-Hit
  :params []
  :when [(?fact <- :wnwdy::Hit)])


(:wat::core::defn :wnwdy::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnwdy::q-Hit))))

(:wat::core::defn :wnwdy::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules (:wat::core::PersistentVector (:wnwdy::windy-not-cold))]
    (:wnwdy::line 1 "wind-only"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 40 :loc "MCI")))))
    (:wnwdy::line 2 "wind-hot"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 40 :loc "MCI")
            (:wnwdy::Temp :c 80 :loc "MCI")))))
    (:wnwdy::line 3 "wind-cold"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 40 :loc "MCI")
            (:wnwdy::Temp :c 10 :loc "MCI")))))
    (:wnwdy::line 4 "calm"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 20 :loc "MCI")))))
    (:wnwdy::line 5 "two-locs"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 40 :loc "MCI")
            (:wnwdy::Wind :kph 50 :loc "ORD")))))
    (:wnwdy::line 6 "two-locs-one-cold"
      (:wnwdy::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnwdy::q-Hit)))
            (:wnwdy::Wind :kph 40 :loc "MCI")
            (:wnwdy::Wind :kph 50 :loc "ORD")
            (:wnwdy::Temp :c 10 :loc "DFW")))))))
