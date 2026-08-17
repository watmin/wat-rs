;; wat-scripts/perf/grid/where-not-and.wat — :not of :and (Clara negated conjunction).
;; Twin of where-not-and.clj. Clara test-negated-conjunction:
;;   [:not [:and [Wind > 30] [Temp < 20]]]
;; Empty world matches. Wind+Temp together does not.
;; Rows 5–8: same :and after a Station prefix (join-filter, ?loc seeded).

(:wat::core::defrecord :wna::Temp    [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wna::Wind    [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wna::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wna::Hit     [k <- :wat::core::i64])
(:wat::core::defrecord :wna::At      [loc <- :wat::core::String])

(:wat::rete::defrule :wna::not-cold-and-windy
  :when [(:wat::rete::not
           (:wat::rete::and
             (:wna::Wind (?w <- :kph)
               (:wat::rete::core::i64::> ?w 30))
             (:wna::Temp (?c <- :c)
               (:wat::rete::core::i64::< ?c 20))))]
  :then [(:wna::Hit :k 1)])

(:wat::rete::defrule :wna::station-not-both
  :when [(:wna::Station (?loc <- :loc))
         (:wat::rete::not
           (:wat::rete::and
             (:wna::Wind (?loc <- :loc) (?w <- :kph)
               (:wat::rete::core::i64::> ?w 30))
             (:wna::Temp (?loc <- :loc) (?c <- :c)
               (:wat::rete::core::i64::< ?c 20))))]
  :then [(:wna::At :loc ?loc)])

(:wat::core::defn :wna::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query-by-type-string s "wna::Hit")))

(:wat::core::defn :wna::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query-by-type-string s "wna::At")))

(:wat::core::defn :wna::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wna::not-cold-and-windy))
                    pref (:wat::core::PersistentVector (:wna::station-not-both))]
    (:wna::line 1 "empty"
      (:wna::n-hit (:wat::rete::fire-rules (:wat::rete::compile lead))))
    (:wna::line 2 "wind-only"
      (:wna::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile lead)
            (:wna::Wind :kph 40 :loc "MCI")))))
    (:wna::line 3 "temp-only"
      (:wna::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile lead)
            (:wna::Temp :c 10 :loc "MCI")))))
    (:wna::line 4 "both"
      (:wna::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile lead)
            (:wna::Wind :kph 40 :loc "MCI")
            (:wna::Temp :c 10 :loc "MCI")))))
    (:wna::line 5 "prefix-empty"
      (:wna::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile pref)
            (:wna::Station :loc "MCI")))))
    (:wna::line 6 "prefix-wind"
      (:wna::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile pref)
            (:wna::Station :loc "MCI")
            (:wna::Wind :kph 40 :loc "MCI")))))
    (:wna::line 7 "prefix-temp"
      (:wna::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile pref)
            (:wna::Station :loc "MCI")
            (:wna::Temp :c 10 :loc "MCI")))))
    (:wna::line 8 "prefix-both"
      (:wna::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile pref)
            (:wna::Station :loc "MCI")
            (:wna::Wind :kph 40 :loc "MCI")
            (:wna::Temp :c 10 :loc "MCI")))))))
