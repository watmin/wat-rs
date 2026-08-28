;; wat-scripts/perf/grid/where-accum-where.wat — :where on an accum result.
;; Twin of where-accum-where.clj.
;;   count = 3  → Busy.  count = 2 → not.
;;   max > 40   → Busy.  max ≤ 40 → not.
;; max-not-below (token-var in :not) is where-not-bound, not this family.

(:wat::core::defrecord :waw::Station [loc <- :wat::core::String])
(:wat::core::defrecord :waw::Reading [loc <- :wat::core::String v <- :wat::core::i64])
(:wat::core::defrecord :waw::Busy    [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :waw::count-eq-3
  :when [(:waw::Station (?loc <- :loc))
         (?n <- (:wat::rete::acc::count) :from (:waw::Reading (?loc <- :loc)))
         (:wat::rete::where (:wat::rete::i64::= ?n 3))]
  :then [(:waw::Busy :loc ?loc :n ?n)])

(:wat::rete::defrule :waw::max-gt-40
  :when [(:waw::Station (?loc <- :loc))
         (?m <- (:wat::rete::acc::max ?v) :from (:waw::Reading (?loc <- :loc) (?v <- :v)))
         (:wat::rete::where (:wat::rete::i64::> ?m 40))]
  :then [(:waw::Busy :loc ?loc :n ?m)])

(:wat::rete::defquery :waw::q-Busy
  :params []
  :when [(?fact <- :waw::Busy)])


(:wat::core::defn :waw::n-busy [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:waw::q-Busy))))

(:wat::core::defn :waw::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat
        (:wat::string::concat " " name)
        (:wat::string::concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [cnt (:wat::core::PersistentVector (:waw::count-eq-3))
                    mx  (:wat::core::PersistentVector (:waw::max-gt-40))]
    (:waw::line 1 "count-eq-3"
      (:waw::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all cnt (:wat::core::PersistentVector (:waw::q-Busy)))
            (:waw::Station :loc "OSL")
            (:waw::Reading :loc "OSL" :v 1)
            (:waw::Reading :loc "OSL" :v 2)
            (:waw::Reading :loc "OSL" :v 3)))))
    (:waw::line 2 "count-eq-3-miss"
      (:waw::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all cnt (:wat::core::PersistentVector (:waw::q-Busy)))
            (:waw::Station :loc "OSL")
            (:waw::Reading :loc "OSL" :v 1)
            (:waw::Reading :loc "OSL" :v 2)))))
    (:waw::line 3 "max-gt-40"
      (:waw::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mx (:wat::core::PersistentVector (:waw::q-Busy)))
            (:waw::Station :loc "OSL")
            (:waw::Reading :loc "OSL" :v 50)
            (:waw::Reading :loc "OSL" :v 40)))))
    (:waw::line 4 "max-le-40"
      (:waw::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mx (:wat::core::PersistentVector (:waw::q-Busy)))
            (:waw::Station :loc "OSL")
            (:waw::Reading :loc "OSL" :v 40)
            (:waw::Reading :loc "OSL" :v 30)))))
    (:waw::line 5 "count-two-stations"
      (:waw::n-busy
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all cnt (:wat::core::PersistentVector (:waw::q-Busy)))
            (:waw::Station :loc "OSL")
            (:waw::Station :loc "BGO")
            (:waw::Reading :loc "OSL" :v 1)
            (:waw::Reading :loc "OSL" :v 2)
            (:waw::Reading :loc "OSL" :v 3)
            (:waw::Reading :loc "BGO" :v 1)
            (:waw::Reading :loc "BGO" :v 2)
            (:waw::Reading :loc "BGO" :v 3)))))))
