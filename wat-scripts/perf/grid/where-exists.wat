;; wat-scripts/perf/grid/where-exists.wat — leading AND mid-chain :exists.
;; Twin of where-exists.clj.
;;   [:exists [Wind ?loc]]                         — one Hit per distinct loc
;;   [:exists [Temp ?loc]] [:exists [Wind ?loc]]   — both must exist at loc
;;   [:or [:exists [Caw]] [:exists [Temp < 20]]]   — either presence
;;   [Loc ?loc] [:exists [Wind ?loc]]              — left seed; two winds → one Hit

(:wat::core::defrecord :wex::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wex::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wex::Caw  [t <- :wat::core::i64 w <- :wat::core::i64])
(:wat::core::defrecord :wex::Loc  [loc <- :wat::core::String])
(:wat::core::defrecord :wex::At   [loc <- :wat::core::String])
(:wat::core::defrecord :wex::Hit  [k <- :wat::core::i64])

(:wat::rete::defrule :wex::lead-wind
  :when [(:wat::rete::exists (:wex::Wind (?loc <- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::both-exist
  :when [(:wat::rete::exists (:wex::Temp (?loc <- :loc)))
         (:wat::rete::exists (:wex::Wind (?loc <- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::or-exists
  :when [(:wat::rete::or
           (:wat::rete::exists (:wex::Caw (?t <- :t)))
           (:wat::rete::exists
             (:wex::Temp (?c <- :c)
               (:wat::rete::core::i64::< ?c 20))))]
  :then [(:wex::Hit :k 1)])

;; Mid-chain: Loc is the left token. Exists binds nothing; two Winds → one At.
(:wat::rete::defrule :wex::mid-wind
  :when [(:wex::Loc (?loc <- :loc))
         (:wat::rete::exists (:wex::Wind (?loc <- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::mid-both
  :when [(:wex::Loc (?loc <- :loc))
         (:wat::rete::exists (:wex::Temp (?loc <- :loc)))
         (:wat::rete::exists (:wex::Wind (?loc <- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defquery :wex::q-At
  :params []
  :when [(?fact <- :wex::At)])


(:wat::rete::defquery :wex::q-Hit
  :params []
  :when [(?fact <- :wex::Hit)])


(:wat::core::defn :wex::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wex::q-At))))

(:wat::core::defn :wex::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wex::q-Hit))))

(:wat::core::defn :wex::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wex::lead-wind))
                    both (:wat::core::PersistentVector (:wex::both-exist))
                    ore  (:wat::core::PersistentVector (:wex::or-exists))
                    mid  (:wat::core::PersistentVector (:wex::mid-wind))
                    mboth (:wat::core::PersistentVector (:wex::mid-both))]
    (:wex::line 1 "lead-empty"
      (:wex::n-at (:wat::rete::fire-rules (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))))))
    (:wex::line 2 "lead-two-same"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "MCI")))))
    (:wex::line 3 "lead-two-locs"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")))))
    (:wex::line 4 "lead-retract"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::retract
            (:wat::rete::retract
              (:wat::rete::insert (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
                (:wex::Wind :kph 50 :loc "MCI")
                (:wex::Wind :kph 60 :loc "MCI"))
              (:wex::Wind :kph 50 :loc "MCI"))
            (:wex::Wind :kph 60 :loc "MCI")))))
    (:wex::line 5 "and-wind-only"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")))))
    (:wex::line 6 "and-diff-locs"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "ORD")))))
    (:wex::line 7 "and-both-mci"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")))))
    (:wex::line 8 "and-two-cities"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Temp :c 70 :loc "ORD")))))
    (:wex::line 9 "or-empty"
      (:wex::n-hit (:wat::rete::fire-rules (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))))))
    (:wex::line 10 "or-caw"
      (:wex::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Caw :t 10 :w 10)))))
    (:wex::line 11 "or-temp"
      (:wex::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Temp :c 10 :loc "MCI")))))
    (:wex::line 12 "or-both"
      (:wex::n-hit
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Caw :t 10 :w 10)
            (:wex::Temp :c 10 :loc "MCI")))))
    (:wex::line 13 "mid-loc-only"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Loc :loc "MCI")))))
    (:wex::line 14 "mid-wind-only"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Wind :kph 50 :loc "MCI")))))
    (:wex::line 15 "mid-two-winds"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Loc :loc "MCI")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "MCI")))))
    (:wex::line 16 "mid-two-locs"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")))))
    (:wex::line 17 "mid-both-one-city"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mboth (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")))))
    (:wex::line 18 "mid-both-two-cities"
      (:wex::n-at
        (:wat::rete::fire-rules
          (:wat::rete::insert (:wat::rete::compile-all mboth (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit)))
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")
            (:wex::Temp :c 70 :loc "ORD")))))))
