;; wat-scripts/perf/grid/where-query-compat.wat — query-mouth compat.
;; Twin of where-query-compat.clj. Subject is `query` answers (binding maps),
;; not a Hit-count readout. Three-way: Clara | wat-oracle | wat-native.
;;
;;     JAVA_HOME=$HOME/opt/jdk-21.0.12+8 \
;;       bash wat-scripts/perf/grid/check-query-compat.sh
;;
;; Rows print sorted scalar bindings. A bound record prints as presence
;; (`has=?t`), never as EDN (Clara and wat write records differently).

(:wat::core::defrecord :wqc::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wqc::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wqc::Hit  [loc <- :wat::core::String])

(:wat::rete::defrule :wqc::mark
  :when [(:wqc::Wind (?loc <- :loc) (?w <- :kph)
           (:wat::rete::core::i64::> ?w 10))]
  :then [(:wqc::Hit :loc ?loc)])

(:wat::rete::defquery :wqc::q-fields
  :params []
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))])

(:wat::rete::defquery :wqc::q-plain
  :params []
  :when [(?fact <- :wqc::Temp)])

(:wat::rete::defquery :wqc::q-bound
  :params []
  :when [(?t <- :wqc::Temp (?c <- :c))])

(:wat::rete::defquery :wqc::q-at
  :params [?loc]
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))])

(:wat::rete::defquery :wqc::q-join
  :params []
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))
         (:wqc::Wind (?w <- :kph) (?loc <- :loc))])

(:wat::rete::defquery :wqc::q-count-at
  :params [?loc]
  :when [(?n <- (:wat::rete::acc::count) :from (:wqc::Temp (?loc <- :loc)))])

(:wat::rete::defquery :wqc::q-count-wind
  :params [?loc]
  :when [(?n <- (:wat::rete::acc::count) :from (:wqc::Temp (?loc <- :loc)))
         (:wqc::Wind (?loc <- :loc))])

(:wat::rete::defquery :wqc::q-no-wind
  :params []
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))
         (:wat::rete::not (:wqc::Wind (?loc <- :loc)))])

(:wat::rete::defquery :wqc::q-has-wind
  :params []
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))
         (:wat::rete::exists (:wqc::Wind (?loc <- :loc)))])

(:wat::rete::defquery :wqc::q-cool
  :params []
  :when [(:wqc::Temp (?c <- :c) (?loc <- :loc))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))])

(:wat::rete::defquery :wqc::q-Hit
  :params []
  :when [(:wqc::Hit (?loc <- :loc))])

(:wat::core::defn :wqc::has-key
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
   k       <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::length answers) 0)
    "empty"
    (:wat::core::match
      (:wat::core::PersistentMap/get (:wat::core::first answers) k)
      ((:wat::core::Some _) "yes")
      (:wat::core::None "none"))))

(:wat::core::defn :wqc::i64-of
  [p <- :wat::core::PersistentMap k <- :wat::core::String] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::PersistentMap/get p k)
    (:wat::core::String/concat "query-compat missing " k)))

(:wat::core::defn :wqc::str-of
  [p <- :wat::core::PersistentMap k <- :wat::core::String] -> :wat::core::String
  (:wat::core::Option/expect (:wat::core::PersistentMap/get p k)
    (:wat::core::String/concat "query-compat missing " k)))

(:wat::core::defn :wqc::render-strs
  [v <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String s <- :wat::core::String] -> :wat::core::String
      (:wat::core::String/concat acc (:wat::core::String/concat " " s)))
    ""
    v))

(:wat::core::defn :wqc::pairs-c-loc
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])]
  -> :wat::core::String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::String)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::String
            (:wat::core::String/concat
              (:wat::core::i64::to-string (:wqc::i64-of p "?c"))
              (:wat::core::String/concat "," (:wqc::str-of p "?loc"))))
          answers)))))

(:wat::core::defn :wqc::pairs-join
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])]
  -> :wat::core::String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::String)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::String
            (:wat::core::String/concat
              (:wat::core::i64::to-string (:wqc::i64-of p "?c"))
              (:wat::core::String/concat ","
                (:wat::core::String/concat
                  (:wat::core::i64::to-string (:wqc::i64-of p "?w"))
                  (:wat::core::String/concat "," (:wqc::str-of p "?loc"))))))
          answers)))))

(:wat::core::defn :wqc::vals-c
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])]
  -> :wat::core::String
  (:wqc::render-strs
    (:wat::core::into (:wat::core::Vector :wat::core::String)
      (:wat::core::map
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::String
          (:wat::core::i64::to-string x))
        (:wat::core::sort
          (:wat::core::into (:wat::core::Vector :wat::core::i64)
            (:wat::core::map
              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                (:wqc::i64-of p "?c"))
              answers)))))))

(:wat::core::defn :wqc::vals-loc
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])]
  -> :wat::core::String
  (:wqc::render-strs
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::String)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::String
            (:wqc::str-of p "?loc"))
          answers)))))

(:wat::core::defn :wqc::one-n
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::length answers) 0)
    ""
    (:wat::core::String/concat " "
      (:wat::core::i64::to-string (:wqc::i64-of (:wat::core::first answers) "?n")))))

(:wat::core::defn :wqc::line
  [row <- :wat::core::i64 name <- :wat::core::String body <- :wat::core::String]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat (:wat::core::String/concat " " name) body))))

(:wat::core::defn :wqc::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert session
    (:wqc::Temp :c 15 :loc "MCI")
    (:wqc::Temp :c 80 :loc "MCI")
    (:wqc::Temp :c 40 :loc "SFO")
    (:wqc::Temp :c 10 :loc "ORD")
    (:wqc::Wind :kph 20 :loc "MCI")
    (:wqc::Wind :kph 5  :loc "SFO")
    (:wqc::Wind :kph 20 :loc "LAX")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [qs (:wat::core::PersistentVector
          (:wqc::q-fields) (:wqc::q-plain) (:wqc::q-bound) (:wqc::q-at)
          (:wqc::q-join) (:wqc::q-count-at) (:wqc::q-count-wind)
          (:wqc::q-no-wind) (:wqc::q-has-wind) (:wqc::q-cool))
     world (:wat::rete::fire-rules
             (:wqc::seed (:wat::rete::compile-all (:wat::core::PersistentVector) qs)))
     fields (:wat::rete::query world (:wqc::q-fields))
     plain  (:wat::rete::query world (:wqc::q-plain))
     bound  (:wat::rete::query world (:wqc::q-bound))
     at-mci (:wat::rete::query world (:wqc::q-at) :?loc "MCI")
     at-xxx (:wat::rete::query world (:wqc::q-at) :?loc "XXX")
     join   (:wat::rete::query world (:wqc::q-join))
     n-mci  (:wat::rete::query world (:wqc::q-count-at) :?loc "MCI")
     n-lax  (:wat::rete::query world (:wqc::q-count-wind) :?loc "LAX")
     none   (:wat::rete::query world (:wqc::q-no-wind))
     some   (:wat::rete::query world (:wqc::q-has-wind))
     cool   (:wat::rete::query world (:wqc::q-cool))
     hits   (:wat::rete::query
              (:wat::rete::fire-rules
                (:wqc::seed
                  (:wat::rete::compile-all
                    (:wat::core::PersistentVector (:wqc::mark))
                    (:wat::core::PersistentVector (:wqc::q-Hit)))))
              (:wqc::q-Hit))
     empty  (:wat::rete::query
              (:wat::rete::fire-rules
                (:wat::rete::compile-all (:wat::core::PersistentVector) qs))
              (:wqc::q-fields))]
    (:wqc::line 1 "fields"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length fields)))
        (:wat::core::String/concat " ->" (:wqc::pairs-c-loc fields))))
    (:wqc::line 2 "plain"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length plain)))
        (:wat::core::String/concat " has=?c " (:wqc::has-key plain "?c"))))
    (:wqc::line 3 "fact-bind"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length bound)))
        (:wat::core::String/concat
          (:wat::core::String/concat " has=?t " (:wqc::has-key bound "?t"))
          (:wat::core::String/concat " ->" (:wqc::vals-c bound)))))
    (:wqc::line 4 "params-mci"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length at-mci)))
        (:wat::core::String/concat " ->" (:wqc::pairs-c-loc at-mci))))
    (:wqc::line 5 "params-xxx"
      (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length at-xxx))))
    (:wqc::line 6 "join"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length join)))
        (:wat::core::String/concat " ->" (:wqc::pairs-join join))))
    (:wqc::line 7 "count-mci"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length n-mci)))
        (:wat::core::String/concat " ->" (:wqc::one-n n-mci))))
    (:wqc::line 8 "count-zero"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length n-lax)))
        (:wat::core::String/concat " ->" (:wqc::one-n n-lax))))
    (:wqc::line 9 "not-wind"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length none)))
        (:wat::core::String/concat " ->" (:wqc::pairs-c-loc none))))
    (:wqc::line 10 "exists-wind"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length some)))
        (:wat::core::String/concat " ->" (:wqc::pairs-c-loc some))))
    (:wqc::line 11 "where-cool"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length cool)))
        (:wat::core::String/concat " ->" (:wqc::pairs-c-loc cool))))
    (:wqc::line 12 "derived"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length hits)))
        (:wat::core::String/concat " ->" (:wqc::vals-loc hits))))
    (:wqc::line 13 "empty"
      (:wat::core::String/concat " n=" (:wat::core::i64::to-string (:wat::core::length empty))))))
