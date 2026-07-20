;; Phase 0 scale probe: wire the RICH arena rule graph (validated in probe-arena-rich-graph.wat)
;; through sift-rules-defsvc + mem-store'/journal', flood N=800 (10-category cycle, 80 cycles),
;; page at :limit 100 (8 exact pages), on BOTH loci. Confirms the mem-store' duplication ruin does
;; NOT bite under 1000 total rows (brief's Phase-0 guidance) before wiring the real fixture files.
;; Expect EXACTLY 720 terminal Deductions (80 cycles x 9 deductions/cycle) on both loci.

(:wat::core::defrecord :arena::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
(:wat::core::defrecord :arena::Client [ip <- :wat::core::String  geo <- :arena::Geo  reputation <- :wat::core::i64])
(:wat::core::defenum   :arena::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defrecord :arena::Route  [method <- :arena::Method  path <- :wat::core::String  status <- :wat::core::i64])
(:wat::core::defrecord :arena::Timing [dns-ns <- :wat::core::i64  total-ns <- :wat::core::i64])
(:wat::core::defrecord :arena::Event  [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])

(:wat::query::sift-rules-defsvc
  :name :arena::my-sift
  :defs [(:wat::core::defrecord :arena::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
         (:wat::core::defrecord :arena::Client [ip <- :wat::core::String  geo <- :arena::Geo  reputation <- :wat::core::i64])
         (:wat::core::defenum   :arena::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
         (:wat::core::defrecord :arena::Route  [method <- :arena::Method  path <- :wat::core::String  status <- :wat::core::i64])
         (:wat::core::defrecord :arena::Timing [dns-ns <- :wat::core::i64  total-ns <- :wat::core::i64])
         (:wat::core::defrecord :arena::Event  [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])
         (:wat::core::defrecord :arena::Suspect [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])
         (:wat::core::defrecord :arena::Flagged [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing])
         (:wat::core::defrecord :arena::Anomaly  [client <- :arena::Client])
         (:wat::core::defrecord :arena::Breach   [client <- :arena::Client])
         (:wat::core::defrecord :arena::Overflow [bytes  <- :wat::core::i64])
         (:wat::core::defrecord :arena::Critical [client <- :arena::Client])]
  :rules [(:wat::rete::defrule :arena::suspect-rule
            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes))
                   (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 500000))
                   (:wat::rete::where (:wat::core::< (:arena::Client/reputation ?client) 0))
                   (:wat::rete::where (:wat::core::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
            :then (:wat::rete::insert (:arena::Suspect :client ?client :route ?route :timing ?timing :bytes ?bytes)))
          (:wat::rete::defrule :arena::anomaly-rule
            :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 5000000))
                   (:wat::rete::where (:wat::core::= (:arena::Route/status ?route) 200))]
            :then (:wat::rete::insert (:arena::Anomaly :client ?client)))
          (:wat::rete::defrule :arena::breach-rule
            :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 2000000))
                   (:wat::rete::where (:wat::core::= (:arena::Route/status ?route) 200))]
            :then (:wat::rete::insert (:arena::Breach :client ?client)))
          (:wat::rete::defrule :arena::overflow-rule
            :when [(:arena::Event (?bytes <- :bytes))
                   (:wat::rete::where (:wat::core::> ?bytes 10000000))]
            :then (:wat::rete::insert (:arena::Overflow :bytes ?bytes)))
          (:wat::rete::defrule :arena::flagged-rule
            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::core::= (:arena::Route/method ?route) :arena::Method::POST))
                   (:wat::rete::where (:wat::core::< (:arena::Client/reputation ?client) -50))]
            :then (:wat::rete::insert (:arena::Flagged :client ?client :route ?route :timing ?timing)))
          (:wat::rete::defrule :arena::critical-rule
            :when [(:arena::Flagged (?client <- :client) (?timing <- :timing))
                   (:wat::rete::where (:wat::core::> (:arena::Timing/dns-ns ?timing) 300000))]
            :then (:wat::rete::insert (:arena::Critical :client ?client)))])

(:wat::core::defrecord :arena::PageAcc
  [done  <- :wat::core::bool
   cur   <- (:wat::core::Option :wat::core::String)
   acc   <- :wat::core::i64
   clean <- :wat::core::bool])

;; log-for-i: 10-category cycle (see probe-arena-rich-graph.wat's validated math).
(:wat::core::defn :arena::log-for-i
  [i <- :wat::core::i64  tags <- (:wat::core::HashMap :wat::core::keyword :wat::core::String)]
  -> :wat::telemetry::Log
  (:wat::core::let
    [cat (:wat::core::mod i 10)
     mk  (:wat::core::fn
           [ctry <- :wat::core::String rep <- :wat::core::i64 method <- :arena::Method status <- :wat::core::i64
            total-ns <- :wat::core::i64 dns-ns <- :wat::core::i64 bytes <- :wat::core::i64]
           -> :arena::Event
           (:arena::Event
             :client (:arena::Client :ip "1.2.3.4" :geo (:arena::Geo :country ctry :asn 64500) :reputation rep)
             :route  (:arena::Route :method method :path "/api" :status status)
             :timing (:arena::Timing :dns-ns dns-ns :total-ns total-ns)
             :bytes  bytes))
     ev (:wat::core::if (:wat::core::= cat 0)
          (mk "US" 50 :arena::Method::GET 200 100000 10000 1000)
          (:wat::core::if (:wat::core::= cat 1)
            (mk "US" -10 :arena::Method::GET 200 6000000 10000 1000)
            (:wat::core::if (:wat::core::= cat 2)
              (mk "XX" -10 :arena::Method::GET 200 3000000 10000 1000)
              (:wat::core::if (:wat::core::= cat 3)
                (mk "XX" -20 :arena::Method::GET 200 6000000 10000 1000)
                (:wat::core::if (:wat::core::= cat 4)
                  (mk "XX" -5 :arena::Method::GET 404 6000000 10000 1000)
                  (:wat::core::if (:wat::core::= cat 5)
                    (mk "US" 50 :arena::Method::GET 200 100000 10000 15000000)
                    (:wat::core::if (:wat::core::= cat 6)
                      (mk "US" -60 :arena::Method::POST 200 100000 500000 1000)
                      (:wat::core::if (:wat::core::= cat 7)
                        (mk "US" -60 :arena::Method::POST 200 100000 100000 1000)
                        (:wat::core::if (:wat::core::= cat 8)
                          (mk "XX" -60 :arena::Method::POST 200 9000000 900000 20000000)
                          (mk "CA" 10 :arena::Method::PUT 500 50000 20000 500))))))))))
     msg (:wat::edn::write ev)]
    (:wat::telemetry::Log :namespace "arena-scale-ns" :uuid (:wat::core::Uuid/nil) :tags tags
      :time-ns (:wat::core::i64::+ i 1) :caller :probe
      :level :wat::telemetry::Level::Info :message msg)))

(:wat::core::defn :user::sift-rules-scale-thread [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::kernel::connect' jaddr)
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs  (:wat::core::range 0 100)
     logs  (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log (:arena::log-for-i i tags))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:arena::my-sift'/start :locus (:wat::spawn::thread)
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::kernel::connect' (:arena::my-sift'::Handle/addr sh))
     page-idxs (:wat::core::range 0 12)
     initial (:arena::PageAcc :done false :cur :wat::core::None :acc 0 :clean true)
     final (:wat::core::foldl
             (:wat::core::fn [state <- :arena::PageAcc _i <- :wat::core::i64] -> :arena::PageAcc
               (:wat::core::if (:arena::PageAcc/done state)
                 state
                 (:wat::core::let
                   [resp (:arena::my-sift/sift-rules svc
                           (:arena::my-sift::SiftRulesRequest :namespace "arena-scale-ns"
                             :time-lo 0 :time-hi 100000000 :limit 100
                             :cursor (:arena::PageAcc/cur state)))]
                   (:wat::core::match resp -> :arena::PageAcc
                     ((:arena::my-sift::SiftRulesResponse::Deductions items cur)
                       (:wat::core::let
                         [page-clean (:wat::core::foldl
                                       (:wat::core::fn [ok <- :wat::core::bool v <- :wat::core::Value] -> :wat::core::bool
                                         (:wat::core::if ok
                                           (:wat::core::not
                                             (:wat::core::or
                                               (:wat::core::= (:wat::core::type v) "arena::Suspect")
                                               (:wat::core::= (:wat::core::type v) "arena::Flagged")))
                                           false))
                                       true
                                       items)
                          new-acc (:wat::core::+ (:arena::PageAcc/acc state) (:wat::core::length items))
                          new-clean (:wat::core::and (:arena::PageAcc/clean state) page-clean)]
                         (:wat::core::match cur -> :arena::PageAcc
                           (:wat::core::None (:arena::PageAcc :done true :cur :wat::core::None :acc new-acc :clean new-clean))
                           ((:wat::core::Some c) (:arena::PageAcc :done false :cur (:wat::core::Some c) :acc new-acc :clean new-clean)))))
                     ((:arena::my-sift::SiftRulesResponse::Fatal _err)
                       (:arena::PageAcc :done true :cur :wat::core::None :acc -999999 :clean false))))))
             initial
             page-idxs)]
    (:wat::core::if (:arena::PageAcc/clean final) (:arena::PageAcc/acc final) -1)))

(:wat::core::defn :user::sift-rules-scale-process [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::kernel::connect' jaddr)
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs  (:wat::core::range 0 100)
     logs  (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log (:arena::log-for-i i tags))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:arena::my-sift'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::kernel::connect' (:arena::my-sift'::Handle/addr sh))
     page-idxs (:wat::core::range 0 12)
     initial (:arena::PageAcc :done false :cur :wat::core::None :acc 0 :clean true)
     final (:wat::core::foldl
             (:wat::core::fn [state <- :arena::PageAcc _i <- :wat::core::i64] -> :arena::PageAcc
               (:wat::core::if (:arena::PageAcc/done state)
                 state
                 (:wat::core::let
                   [resp (:arena::my-sift/sift-rules svc
                           (:arena::my-sift::SiftRulesRequest :namespace "arena-scale-ns"
                             :time-lo 0 :time-hi 100000000 :limit 100
                             :cursor (:arena::PageAcc/cur state)))]
                   (:wat::core::match resp -> :arena::PageAcc
                     ((:arena::my-sift::SiftRulesResponse::Deductions items cur)
                       (:wat::core::let
                         [page-clean (:wat::core::foldl
                                       (:wat::core::fn [ok <- :wat::core::bool v <- :wat::core::Value] -> :wat::core::bool
                                         (:wat::core::if ok
                                           (:wat::core::not
                                             (:wat::core::or
                                               (:wat::core::= (:wat::core::type v) "arena::Suspect")
                                               (:wat::core::= (:wat::core::type v) "arena::Flagged")))
                                           false))
                                       true
                                       items)
                          new-acc (:wat::core::+ (:arena::PageAcc/acc state) (:wat::core::length items))
                          new-clean (:wat::core::and (:arena::PageAcc/clean state) page-clean)]
                         (:wat::core::match cur -> :arena::PageAcc
                           (:wat::core::None (:arena::PageAcc :done true :cur :wat::core::None :acc new-acc :clean new-clean))
                           ((:wat::core::Some c) (:arena::PageAcc :done false :cur (:wat::core::Some c) :acc new-acc :clean new-clean)))))
                     ((:arena::my-sift::SiftRulesResponse::Fatal _err)
                       (:arena::PageAcc :done true :cur :wat::core::None :acc -999999 :clean false))))))
             initial
             page-idxs)]
    (:wat::core::if (:arena::PageAcc/clean final) (:arena::PageAcc/acc final) -1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::string::concat "THREAD  (want 720): " (:wat::core::str (:user::sift-rules-scale-thread))))
    (:wat::kernel::println (:wat::core::string::concat "PROCESS (want 720): " (:wat::core::str (:user::sift-rules-scale-process))))))
