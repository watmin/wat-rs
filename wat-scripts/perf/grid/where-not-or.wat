;; wat-scripts/perf/grid/where-not-or.wat — :not of :or (Clara negated disjunction).
;; Twin of where-not-or.clj. Clara test-negated-disjunction:
;;   [:not [:or [Wind > 30] [Temp < 20]]]
;; Empty world matches. Either fact kills it. Retract restores.
;; Rows 6–8: same :or after a Station prefix (join-filter, ?loc seeded).

(:wat::core::defrecord :wno::Temp    [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wno::Wind    [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wno::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wno::Hit     [k <- :wat::core::i64])
(:wat::core::defrecord :wno::At      [loc <- :wat::core::String])

(:wat::rete::defrule :wno::not-cold-or-windy
  :when [(:wat::rete::not
           (:wat::rete::or
             (:wno::Wind (?w <- :kph)
               (:wat::rete::i64::> ?w 30))
             (:wno::Temp (?c <- :c)
               (:wat::rete::i64::< ?c 20))))]
  :then [(:wno::Hit :k 1)])

(:wat::rete::defrule :wno::station-not-either
  :when [(:wno::Station (?loc <- :loc))
         (:wat::rete::not
           (:wat::rete::or
             (:wno::Wind (?loc <- :loc) (?w <- :kph)
               (:wat::rete::i64::> ?w 30))
             (:wno::Temp (?loc <- :loc) (?c <- :c)
               (:wat::rete::i64::< ?c 20))))]
  :then [(:wno::At :loc ?loc)])

(:wat::rete::defquery :wno::q-Hit
  :params []
  :when [(?fact <- :wno::Hit)])


(:wat::rete::defquery :wno::q-At
  :params []
  :when [(?fact <- :wno::At)])


(:wat::core::defn :wno::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wno::q-Hit))))

(:wat::core::defn :wno::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wno::q-At))))

(:wat::core::defn :wno::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wno::not-cold-or-windy))
                    pref (:wat::core::PersistentVector (:wno::station-not-either))]
    (:wno::line 1 "empty"
      (:wno::n-hit (:wat::rete::fire-rules (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At))))))
    (:wno::line 2 "wind"
      (:wno::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Wind :kph 40 :loc "MCI")))))
    (:wno::line 3 "temp"
      (:wno::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Temp :c 10 :loc "MCI")))))
    (:wno::line 4 "both"
      (:wno::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Wind :kph 40 :loc "MCI")
            (:wno::Temp :c 10 :loc "MCI")))))
    (:wno::line 5 "retract-wind"
      (:wno::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::retract
            (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
              (:wno::Wind :kph 40 :loc "MCI"))
            (:wno::Wind :kph 40 :loc "MCI")))))
    (:wno::line 6 "prefix-empty"
      (:wno::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Station :loc "MCI")))))
    (:wno::line 7 "prefix-wind"
      (:wno::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Station :loc "MCI")
            (:wno::Wind :kph 40 :loc "MCI")))))
    (:wno::line 8 "prefix-temp"
      (:wno::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wno::q-Hit) (:wno::q-At)))
            (:wno::Station :loc "MCI")
            (:wno::Temp :c 10 :loc "MCI")))))))
