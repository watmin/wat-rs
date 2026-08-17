;; wat-scripts/perf/grid/where-not-not.wat — :not of :not (double negation).
;; Twin of where-not-not.clj.
;;   [Wind ?loc] [:not [:not [Temp ?loc]]]  ≡  Wind and a Temp at loc
;;   [:not [:not [Temp]]]                   ≡  some Temp exists

(:wat::core::defrecord :wnn::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnn::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wnn::Hit  [loc <- :wat::core::String])
(:wat::core::defrecord :wnn::Yes  [k <- :wat::core::i64])

(:wat::rete::defrule :wnn::wind-not-not-temp
  :when [(:wnn::Wind (?loc <- :loc))
         (:wat::rete::not
           (:wat::rete::not
             (:wnn::Temp (?loc <- :loc))))]
  :then [(:wnn::Hit :loc ?loc)])

(:wat::rete::defrule :wnn::lead-not-not
  :when [(:wat::rete::not
           (:wat::rete::not
             (:wnn::Temp (?c <- :c))))]
  :then [(:wnn::Yes :k 1)])

(:wat::rete::defquery :wnn::q-Hit
  :params []
  :when [(?fact <- :wnn::Hit)])


(:wat::rete::defquery :wnn::q-Yes
  :params []
  :when [(?fact <- :wnn::Yes)])


(:wat::core::defn :wnn::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnn::q-Hit))))

(:wat::core::defn :wnn::n-yes [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wnn::q-Yes))))

(:wat::core::defn :wnn::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [mid (:wat::core::PersistentVector (:wnn::wind-not-not-temp))
                    lead (:wat::core::PersistentVector (:wnn::lead-not-not))]
    (:wnn::line 1 "wind-only"
      (:wnn::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes)))
            (:wnn::Wind :kph 40 :loc "MCI")))))
    (:wnn::line 2 "same-loc"
      (:wnn::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes)))
            (:wnn::Wind :kph 40 :loc "MCI")
            (:wnn::Temp :c 10 :loc "MCI")))))
    (:wnn::line 3 "diff-loc"
      (:wnn::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes)))
            (:wnn::Wind :kph 40 :loc "MCI")
            (:wnn::Temp :c 10 :loc "ORD")))))
    (:wnn::line 4 "temp-only"
      (:wnn::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes)))
            (:wnn::Temp :c 10 :loc "MCI")))))
    (:wnn::line 5 "lead-empty"
      (:wnn::n-yes (:wat::rete::fire-rules (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes))))))
    (:wnn::line 6 "lead-temp"
      (:wnn::n-yes
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wnn::q-Hit) (:wnn::q-Yes)))
            (:wnn::Temp :c 10 :loc "MCI")))))))
