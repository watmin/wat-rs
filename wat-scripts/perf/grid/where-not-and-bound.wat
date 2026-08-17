;; wat-scripts/perf/grid/where-not-and-bound.wat — :not of :and joined on a bound field.
;; Twin of where-not-and-bound.clj. Clara test-complex-negation:
;;   [:not [:and [?t <- Temp] [Cold (= temperature (:temperature ?t))]]]
;; Inner :and is a join on the Temp's temperature, not "both types exist".
;; Wat binds the shared field (?c) — same join, no whole-fact ?t.
;; Rows 5–8: Wind seeds ?loc (Clara negation-with-prior-bindings).

(:wat::core::defrecord :wnab::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnab::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnab::Cold [c <- :wat::core::i64])
(:wat::core::defrecord :wnab::Hit  [k <- :wat::core::i64])
(:wat::core::defrecord :wnab::At   [loc <- :wat::core::String])

(:wat::rete::defrule :wnab::not-match-temp
  :when [(:wat::rete::not
           (:wat::rete::and
             (:wnab::Temp (?c <- :c))
             (:wnab::Cold (?c <- :c))))]
  :then [(:wnab::Hit :k 1)])

(:wat::rete::defrule :wnab::prior-not-match
  :when [(:wnab::Wind (?l <- :loc))
         (:wat::rete::not
           (:wat::rete::and
             (:wnab::Temp (?l <- :loc) (?c <- :c))
             (:wnab::Cold (?c <- :c))))]
  :then [(:wnab::At :loc ?l)])

(:wat::rete::defquery :wnab::q-Hit
  :params []
  :when [(?fact <- :wnab::Hit)])


(:wat::rete::defquery :wnab::q-At
  :params []
  :when [(?fact <- :wnab::At)])


(:wat::core::defn :wnab::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnab::q-Hit))))

(:wat::core::defn :wnab::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnab::q-At))))

(:wat::core::defn :wnab::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wnab::not-match-temp))
                    pref (:wat::core::PersistentVector (:wnab::prior-not-match))]
    (:wnab::line 1 "empty"
      (:wnab::n-hit (:wat::rete::fire-rules (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At))))))
    (:wnab::line 2 "temp-only"
      (:wnab::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Temp :c 10 :loc "MCI")))))
    (:wnab::line 3 "mismatch"
      (:wnab::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Temp :c 10 :loc "MCI")
            (:wnab::Cold :c 20)))))
    (:wnab::line 4 "match"
      (:wnab::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Temp :c 10 :loc "MCI")
            (:wnab::Cold :c 10)))))
    (:wnab::line 5 "prior-empty"
      (:wnab::n-at (:wat::rete::fire-rules (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At))))))
    (:wnab::line 6 "prior-wind"
      (:wnab::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Wind :kph 10 :loc "MCI")))))
    (:wnab::line 7 "prior-other-loc"
      (:wnab::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Wind :kph 10 :loc "MCI")
            (:wnab::Temp :c 10 :loc "ORD")
            (:wnab::Cold :c 10)))))
    (:wnab::line 8 "prior-same-loc"
      (:wnab::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all pref (:wat::core::PersistentVector (:wnab::q-Hit) (:wnab::q-At)))
            (:wnab::Wind :kph 10 :loc "MCI")
            (:wnab::Temp :c 10 :loc "MCI")
            (:wnab::Cold :c 10)))))))
