;; wat-scripts/perf/grid/where-not-and-not.wat — nested :not inside :and.
;; Twin of where-not-and-not.clj. Clara test-complex-negation:
;;   [:not [:and [Temp ?loc] [:not [Cold]]]]
;; and nested-negation-with-prior-bindings (issue 304):
;;   [Wind ?l] [:not [:and [Temp ?l ?c] [:not [Cold ?c]]]]
;; Inner :not is a join-filter on the Temp's temperature, not "no Cold exists".

(:wat::core::defrecord :wnan::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnan::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnan::Cold [c <- :wat::core::i64])
(:wat::core::defrecord :wnan::Hit  [k <- :wat::core::i64])
(:wat::core::defrecord :wnan::At   [loc <- :wat::core::String])

(:wat::rete::defrule :wnan::lead
  :when [(:wat::rete::not
           (:wat::rete::and
             (:wnan::Temp (?c <- :c))
             (:wat::rete::not (:wnan::Cold (?c <- :c)))))]
  :then [(:wnan::Hit :k 1)])

(:wat::rete::defrule :wnan::nested
  :when [(:wnan::Wind (?l <- :loc))
         (:wat::rete::not
           (:wat::rete::and
             (:wnan::Temp (?l <- :loc) (?c <- :c))
             (:wat::rete::not (:wnan::Cold (?c <- :c)))))]
  :then [(:wnan::At :loc ?l)])

(:wat::rete::defquery :wnan::q-Hit
  :params []
  :when [(?fact <- :wnan::Hit)])


(:wat::rete::defquery :wnan::q-At
  :params []
  :when [(?fact <- :wnan::At)])


(:wat::core::defn :wnan::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnan::q-Hit))))

(:wat::core::defn :wnan::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnan::q-At))))

(:wat::core::defn :wnan::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wnan::lead))
                    nest (:wat::core::PersistentVector (:wnan::nested))]
    (:wnan::line 1 "lead-empty"
      (:wnan::n-hit (:wat::rete::fire-rules (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At))))))
    (:wnan::line 2 "lead-temp"
      (:wnan::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Temp :c 10 :loc "MCI")))))
    (:wnan::line 3 "lead-mismatch"
      (:wnan::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Temp :c 10 :loc "MCI")
            (:wnan::Cold :c 20)))))
    (:wnan::line 4 "lead-match"
      (:wnan::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Temp :c 10 :loc "MCI")
            (:wnan::Cold :c 10)))))
    (:wnan::line 5 "nest-wind"
      (:wnan::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all nest (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Wind :kph 10 :loc "MCI")))))
    (:wnan::line 6 "nest-cold20"
      (:wnan::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all nest (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Wind :kph 10 :loc "MCI")
            (:wnan::Temp :c 10 :loc "MCI")
            (:wnan::Cold :c 20)))))
    (:wnan::line 7 "nest-cold10"
      (:wnan::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all nest (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Wind :kph 10 :loc "MCI")
            (:wnan::Temp :c 10 :loc "MCI")
            (:wnan::Cold :c 10)))))
    (:wnan::line 8 "nest-issue304"
      (:wnan::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all nest (:wat::core::PersistentVector (:wnan::q-Hit) (:wnan::q-At)))
            (:wnan::Wind :kph 10 :loc "MCI")
            (:wnan::Temp :c 10 :loc "MCI")
            (:wnan::Cold :c 20)
            (:wnan::Wind :kph 20 :loc "ORD")
            (:wnan::Temp :c 20 :loc "ORD")))))))
