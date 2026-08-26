;; wat-scripts/perf/grid/where-fact-bind.wat — Clara `[?t <- Temp]` fact-bind.
;; Twin of where-fact-bind.clj. Form B: `(?t <- :ns::Type …)`. `<-` binds; the
;; type keyword has `::`. Accumulate stays `(?n <- (acc) :from …)`.
;; You cannot get a record without asking for it.

(:wat::core::defrecord :wfb::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wfb::Hit [c <- :wat::core::i64])

(:wat::rete::defrule :wfb::cool
  :when [(?t <- :wfb::Temp)
         (:wat::rete::where (:wat::rete::core::i64::< (:wfb::Temp/c ?t) 20))]
  :then [(:wfb::Hit :c (:wfb::Temp/c ?t))])

(:wat::rete::defquery :wfb::q-bound
  :params []
  :when [(?t <- :wfb::Temp)])

(:wat::rete::defquery :wfb::q-plain
  :params []
  :when [(?fact <- :wfb::Temp)])

(:wat::rete::defquery :wfb::q-both
  :params []
  :when [(?t <- :wfb::Temp (?c <- :c))])

(:wat::rete::defquery :wfb::q-Hit
  :params []
  :when [(:wfb::Hit (?c <- :c))])

;; Accum query of two Temps at one loc — one group. Clara :from is a
;; fact pattern, not `[?t <- Temp]` (that form is a condition, not :from).
(:wat::rete::defquery :wfb::q-from
  :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:wfb::Temp (?loc <- :loc)))])

(:wat::core::defn :wfb::has-key
  [answers <- (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
   k       <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::length answers) 0)
    "empty"
    (:wat::core::match
      (:wat::core::PersistentMap/get (:wat::core::first answers) k)
      ((:wat::core::Some _) "yes")
      (:wat::core::None "none"))))

(:wat::core::defn :wfb::line
  [row <- :wat::core::i64 name <- :wat::core::String body <- :wat::core::String]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::i64::to-string row))
      (:wat::core::String/concat (:wat::core::String/concat " " name) body))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rules   (:wat::core::PersistentVector (:wfb::cool))
                    queries (:wat::core::PersistentVector
                              (:wfb::q-bound) (:wfb::q-plain) (:wfb::q-both)
                              (:wfb::q-Hit))
                    world (:wat::rete::fire-rules
                            (:wat::rete::insert
                              (:wat::rete::compile-all rules queries)
                              (:wfb::Temp :c 15 :loc "MCI")
                              (:wfb::Temp :c 80 :loc "MCI")))
                    bound (:wat::rete::query world (:wfb::q-bound))
                    plain (:wat::rete::query world (:wfb::q-plain))
                    both  (:wat::rete::query world (:wfb::q-both))
                    hits  (:wat::rete::query world (:wfb::q-Hit))]
    (:wfb::line 1 "bound"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string (:wat::core::length bound)))
        (:wat::core::String/concat " has=?t " (:wfb::has-key bound "?t"))))
    (:wfb::line 2 "plain"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string (:wat::core::length plain)))
        (:wat::core::String/concat " has=?t " (:wfb::has-key plain "?t"))))
    (:wfb::line 3 "both"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string (:wat::core::length both)))
        (:wat::core::String/concat
          (:wat::core::String/concat " has=?t " (:wfb::has-key both "?t"))
          (:wat::core::String/concat " has=?c " (:wfb::has-key both "?c")))))
    (:wfb::line 4 "cool"
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::i64::to-string (:wat::core::length hits)))
        (:wat::core::String/concat " -> "
          (:wat::i64::to-string
            (:wat::core::Option/expect
              (:wat::core::PersistentMap/get (:wat::core::first hits) "?c")
              "q-Hit: ?c")))))
    (:wfb::line 5 "from"
      (:wat::core::let [only-q (:wat::rete::fire-rules
                                 (:wat::rete::insert
                                   (:wat::rete::compile-all
                                     (:wat::core::PersistentVector)
                                     (:wat::core::PersistentVector (:wfb::q-from)))
                                   (:wfb::Temp :c 15 :loc "MCI")
                                   (:wfb::Temp :c 80 :loc "MCI")))
                        grouped (:wat::rete::query only-q (:wfb::q-from))]
        (:wat::core::String/concat
          " n=" (:wat::i64::to-string (:wat::core::length grouped)))))))
