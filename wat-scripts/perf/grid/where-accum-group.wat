;; wat-scripts/perf/grid/where-accum-group.wat — unbound grouping in a leading :from.
;; Twin of where-accum-group.clj.
;;   [?c <- (acc/count) :from [Temp (?loc <- :loc)]]
;; Temps at MCI and ORD are two groups, not one global count.
;; Empty world with a group key does not emit bag-wide 0.
;; Acc-first + Wind at ?loc: Clara defers the accum; Wind MCI and no temps → {?c 0, ?loc MCI}.

(:wat::core::defrecord :wag::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wag::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wag::Busy [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :wag::count-by-loc
  :when [(?n <- (:wat::rete::acc::count) :from (:wag::Temp (?loc <- :loc)))]
  :then [(:wag::Busy :loc ?loc :n ?n)])

(:wat::rete::defrule :wag::acc-first-wind
  :when [(?n <- (:wat::rete::acc::count) :from (:wag::Temp (?loc <- :loc)))
         (:wag::Wind (?loc <- :loc) (?w <- :kph)
           (:wat::rete::i64::> ?w 10))]
  :then [(:wag::Busy :loc ?loc :n ?n)])

(:wat::rete::defquery :wag::q-Busy
  :params []
  :when [(?fact <- :wag::Busy)])


(:wat::core::defn :wag::n-busy [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wag::q-Busy))))

(:wat::core::defn :wag::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [by  (:wat::core::PersistentVector (:wag::count-by-loc))
                    af  (:wat::core::PersistentVector (:wag::acc-first-wind))]
    (:wag::line 1 "two-locs"
      (:wag::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy)))
            (:wag::Temp :c 10 :loc "MCI")
            (:wag::Temp :c 20 :loc "MCI")
            (:wag::Temp :c 30 :loc "ORD")))))
    (:wag::line 2 "empty-group"
      (:wag::n-busy (:wat::rete::fire-rules (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy))))))
    (:wag::line 3 "one-loc"
      (:wag::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy)))
            (:wag::Temp :c 10 :loc "MCI")
            (:wag::Temp :c 20 :loc "MCI")))))
    (:wag::line 4 "acc-first-wind-empty-temp"
      (:wag::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all af (:wat::core::PersistentVector (:wag::q-Busy)))
            (:wag::Wind :kph 20 :loc "MCI")))))
    (:wag::line 5 "acc-first-two-winds"
      (:wag::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all af (:wat::core::PersistentVector (:wag::q-Busy)))
            (:wag::Wind :kph 20 :loc "MCI")
            (:wag::Wind :kph 20 :loc "SFO")
            (:wag::Temp :c 40 :loc "SFO")
            (:wag::Temp :c 50 :loc "SFO")))))))
