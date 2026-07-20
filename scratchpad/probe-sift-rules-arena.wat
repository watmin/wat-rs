;; scratchpad probe — arena rule graph + paging, SMALL SCALE (N=90, limit=10 -> 9 pages) before
;; committing to the full fixture. Validates: (1) Lemma/Deduction extraction on a richer graph
;; (2-level cascade Temp->Hot->Alert, graded-parallel Temp->Critical, Pressure->Rising->Surge,
;; Flow->Stall); (2) cursor-paging end to end; (3) exact deduction math.
;;
;; cat 0: Temp c=10  (cold)      -> 0
;; cat 1: Temp c=60  (hot)       -> Hot(Lemma)+Alert(Ded) = 1
;; cat 2: Temp c=95  (very hot)  -> Hot+Alert+Critical    = 2
;; cat 3: Pressure p=50  (norm)  -> 0
;; cat 4: Pressure p=150 (rise)  -> Rising(Lemma)+Surge(Ded) = 1
;; cat 5: Flow f=20 (norm)       -> 0
;; cat 6: Flow f=2  (stall)      -> Stall(Ded) = 1
;; cat 7: Temp c=5  (v cold)     -> 0
;; cat 8: Pressure p=80 (norm)   -> 0
;; per-cycle(9) = 5 deductions. N=90 -> 10 cycles -> 50 deductions.

(:wat::core::defrecord :arena::Temp     [c <- :wat::core::i64])
(:wat::core::defrecord :arena::Pressure [p <- :wat::core::i64])
(:wat::core::defrecord :arena::Flow     [f <- :wat::core::i64])
(:wat::core::defrecord :arena::Hot      [c <- :wat::core::i64])
(:wat::core::defrecord :arena::Alert    [c <- :wat::core::i64])
(:wat::core::defrecord :arena::Critical [c <- :wat::core::i64])
(:wat::core::defrecord :arena::Rising   [p <- :wat::core::i64])
(:wat::core::defrecord :arena::Surge    [p <- :wat::core::i64])
(:wat::core::defrecord :arena::Stall    [f <- :wat::core::i64])

(:wat::query::sift-rules-defsvc
  :name :arena::my-sift
  :defs [(:wat::core::defrecord :arena::Temp     [c <- :wat::core::i64])
         (:wat::core::defrecord :arena::Pressure [p <- :wat::core::i64])
         (:wat::core::defrecord :arena::Flow     [f <- :wat::core::i64])
         (:wat::core::defrecord :arena::Hot      [c <- :wat::core::i64])
         (:wat::core::defrecord :arena::Alert    [c <- :wat::core::i64])
         (:wat::core::defrecord :arena::Critical [c <- :wat::core::i64])
         (:wat::core::defrecord :arena::Rising   [p <- :wat::core::i64])
         (:wat::core::defrecord :arena::Surge    [p <- :wat::core::i64])
         (:wat::core::defrecord :arena::Stall    [f <- :wat::core::i64])]
  :rules [(:wat::rete::defrule :arena::hot-rule
            :when [(:arena::Temp (?c <- :c) (:wat::core::> ?c 50))]
            :then (:wat::rete::insert (:arena::Hot :c ?c)))
          (:wat::rete::defrule :arena::alert-rule
            :when [(:arena::Hot (?c <- :c))]
            :then (:wat::rete::insert (:arena::Alert :c ?c)))
          (:wat::rete::defrule :arena::critical-rule
            :when [(:arena::Temp (?c <- :c) (:wat::core::> ?c 90))]
            :then (:wat::rete::insert (:arena::Critical :c ?c)))
          (:wat::rete::defrule :arena::rising-rule
            :when [(:arena::Pressure (?p <- :p) (:wat::core::> ?p 100))]
            :then (:wat::rete::insert (:arena::Rising :p ?p)))
          (:wat::rete::defrule :arena::surge-rule
            :when [(:arena::Rising (?p <- :p))]
            :then (:wat::rete::insert (:arena::Surge :p ?p)))
          (:wat::rete::defrule :arena::stall-rule
            :when [(:arena::Flow (?f <- :f) (:wat::core::< ?f 5))]
            :then (:wat::rete::insert (:arena::Stall :f ?f)))])

(:wat::core::defrecord :arena::PageAcc
  [done  <- :wat::core::bool
   cur   <- (:wat::core::Option :wat::core::String)
   acc   <- :wat::core::i64
   clean <- :wat::core::bool
   pages <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::kernel::connect' jaddr)
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs  (:wat::core::range 0 3600)
     logs  (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                 (:wat::core::let
                   [cat (:wat::core::mod i 9)
                    msg (:wat::core::if (:wat::core::= cat 0)
                          (:wat::edn::write (:arena::Temp :c 10))
                          (:wat::core::if (:wat::core::= cat 1)
                            (:wat::edn::write (:arena::Temp :c 60))
                            (:wat::core::if (:wat::core::= cat 2)
                              (:wat::edn::write (:arena::Temp :c 95))
                              (:wat::core::if (:wat::core::= cat 3)
                                (:wat::edn::write (:arena::Pressure :p 50))
                                (:wat::core::if (:wat::core::= cat 4)
                                  (:wat::edn::write (:arena::Pressure :p 150))
                                  (:wat::core::if (:wat::core::= cat 5)
                                    (:wat::edn::write (:arena::Flow :f 20))
                                    (:wat::core::if (:wat::core::= cat 6)
                                      (:wat::edn::write (:arena::Flow :f 2))
                                      (:wat::core::if (:wat::core::= cat 7)
                                        (:wat::edn::write (:arena::Temp :c 5))
                                        (:wat::edn::write (:arena::Pressure :p 80))))))))))]
                   (:wat::telemetry::Log :namespace "arena-rules-ns" :uuid (:wat::core::Uuid/nil) :tags tags
                     :time-ns (:wat::core::i64::+ i 1) :caller :probe
                     :level :wat::telemetry::Level::Info :message msg)))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:arena::my-sift'/start :locus (:wat::spawn::thread)
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::kernel::connect' (:arena::my-sift'::Handle/addr sh))
     page-idxs (:wat::core::range 0 10)
     initial (:arena::PageAcc :done false :cur :wat::core::None :acc 0 :clean true :pages 0)
     final (:wat::core::foldl
             (:wat::core::fn [state <- :arena::PageAcc _i <- :wat::core::i64] -> :arena::PageAcc
               (:wat::core::if (:arena::PageAcc/done state)
                 state
                 (:wat::core::let
                   [resp (:arena::my-sift/sift-rules svc
                           (:arena::my-sift::SiftRulesRequest :namespace "arena-rules-ns"
                             :time-lo 0 :time-hi 100000000 :limit 500
                             :cursor (:arena::PageAcc/cur state)))]
                   (:wat::core::match resp -> :arena::PageAcc
                     ((:arena::my-sift::SiftRulesResponse::Deductions items cur)
                       (:wat::core::let
                         [page-clean (:wat::core::foldl
                                       (:wat::core::fn [ok <- :wat::core::bool v <- :wat::core::Value] -> :wat::core::bool
                                         (:wat::core::if ok
                                           (:wat::core::not
                                             (:wat::core::or
                                               (:wat::core::= (:wat::core::type v) "arena::Hot")
                                               (:wat::core::= (:wat::core::type v) "arena::Rising")))
                                           false))
                                       true
                                       items)
                          new-acc (:wat::core::+ (:arena::PageAcc/acc state) (:wat::core::length items))
                          new-clean (:wat::core::and (:arena::PageAcc/clean state) page-clean)
                          new-pages (:wat::core::i64::+ (:arena::PageAcc/pages state) 1)]
                         (:wat::core::match cur -> :arena::PageAcc
                           (:wat::core::None (:arena::PageAcc :done true :cur :wat::core::None :acc new-acc :clean new-clean :pages new-pages))
                           ((:wat::core::Some c) (:arena::PageAcc :done false :cur (:wat::core::Some c) :acc new-acc :clean new-clean :pages new-pages)))))
                     ((:arena::my-sift::SiftRulesResponse::Fatal _err)
                       (:arena::PageAcc :done true :cur :wat::core::None :acc -999999 :clean false :pages -1))))))
             initial
             page-idxs)]
    (:wat::kernel::println
      (:wat::core::string::concat "acc=" (:wat::core::string::concat (:wat::core::str (:arena::PageAcc/acc final))
        (:wat::core::string::concat " clean=" (:wat::core::string::concat (:wat::core::str (:arena::PageAcc/clean final))
          (:wat::core::string::concat " pages=" (:wat::core::str (:arena::PageAcc/pages final))))))))))
