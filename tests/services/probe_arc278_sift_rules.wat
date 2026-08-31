;; tests/services/probe_arc278_sift_rules.wat — arc 278 task #6: the `sift-rules-defsvc` macro's
;; RED gate, proven end-to-end on BOTH loci. Co-located fixture for the sibling .rs, slurped via
;; startup_beside(file!()).
;;
;; Modeled on tests/services/probe_arc278_sift_arena.wat (the flood-and-sift arena's
;; mem-store'/journal' setup + grant-before-dial process chain) and
;; tests/services/probe_arc278_sift_logs.wat (the multi-:user::-fn-per-file, both-loci shape).
;;
;; A producer floods N=240 Logs whose messages are user `:usr::Temp` facts (30 HOT, c>50; 210
;; cold). `sift-rules-defsvc` compiles TWO rules over `:usr::Temp` (hot-Temp -> Hot, hot-Temp ->
;; Warn), so each hot Temp fires BOTH rules — one seed, two deductions. Expect EXACTLY 60
;; (30 hot x 2 rules); a cold Temp contributes 0. Both THREAD and PROCESS loci (process needs
;; grant-before-dial at every hop, mirroring the arena's chain).
;;
;; A second scenario proves the fail-closed guard: a Log whose message type is NOT among the
;; macro's `:defs` (a `:usr::Other` record, deliberately never listed) makes the WHOLE page
;; `::Fatal` — never a silent skip.

(:wat::core::defrecord :usr::Temp  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot   [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Other [x <- :wat::core::i64]) ;; deliberately NOT in :defs below

(:wat::query::sift-rules-defsvc
  :name :usr::my-sift
  :defs [(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
         (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
         (:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])]
  :rules [(:wat::rete::defrule :usr::hot-rule
            :when [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))]
            :then [(:usr::Hot :c ?c)])
          (:wat::rete::defrule :usr::warn-rule
            :when [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))]
            :then [(:usr::Warn :c ?c)])])

;; ── shared log-building helper form, inlined per :user:: fn (a plain top-level defn would not
;; cross a PROCESS fork's sift service child, so each entry point builds its own 240-log Vector) ──

;; ── THREAD locus — flood 240 Logs (30 hot / 210 cold), sift-rules, expect 60 deductions. ──
(:wat::core::defn :user::sift-rules-thread [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     idxs  (:wat::core::range 0 240)
     logs  (:wat::core::into (:wat::core::Vector :- [:wat::telemetry::Log])
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                 (:wat::core::let
                   [hot? (:wat::i64::< i 30)
                    c    (:wat::core::if hot? 60 10)
                    msg  (:wat::edn::write (:usr::Temp :c c))]
                   (:wat::telemetry::Log :namespace "sift-rules-ns" :uuid (:wat::uuid::nil) :tags tags
                     :time-ns (:wat::i64::+ i 1) :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site)
                     :level :wat::telemetry::Level::Info :message msg)))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:usr::my-sift'/start :locus (:wat::spawn::thread)
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:usr::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-rules-ns" :time-lo 0 :time-hi 100000 :limit 300 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:usr::my-sift::SiftRulesResponse::Deductions items _cur) (:wat::core::length items))
      ((:usr::my-sift::SiftRulesResponse::Fatal _err) -1)
      ((:usr::my-sift::SiftRulesResponse::RequestTooLarge _bytes _cap)
        (:wat::kernel::assertion-failed! "sift-rules: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
      ((:usr::my-sift::SiftRulesResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; ── PROCESS locus — the loci-agnostic proof. SAME scenario across a FORK: mem-store' + journal'
;; + my-sift' all on process, grant-before-dial at every hop (mirrors the arena's chain). ──
(:wat::core::defn :user::sift-rules-process [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     idxs  (:wat::core::range 0 240)
     logs  (:wat::core::into (:wat::core::Vector :- [:wat::telemetry::Log])
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                 (:wat::core::let
                   [hot? (:wat::i64::< i 30)
                    c    (:wat::core::if hot? 60 10)
                    msg  (:wat::edn::write (:usr::Temp :c c))]
                   (:wat::telemetry::Log :namespace "sift-rules-ns" :uuid (:wat::uuid::nil) :tags tags
                     :time-ns (:wat::i64::+ i 1) :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site)
                     :level :wat::telemetry::Level::Info :message msg)))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:usr::my-sift'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:usr::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-rules-ns" :time-lo 0 :time-hi 100000 :limit 300 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:usr::my-sift::SiftRulesResponse::Deductions items _cur) (:wat::core::length items))
      ((:usr::my-sift::SiftRulesResponse::Fatal _err) -1)
      ((:usr::my-sift::SiftRulesResponse::RequestTooLarge _bytes _cap)
        (:wat::kernel::assertion-failed! "sift-rules: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
      ((:usr::my-sift::SiftRulesResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; ── THREAD locus — fail-closed: one Log's message is `:usr::Other`, NOT among :defs. The WHOLE
;; page must come back ::Fatal (never a silent skip / partial result). ──
(:wat::core::defn :user::sift-rules-fatal-thread [] -> :wat::core::bool
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "sift-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:usr::Temp :c 60)))
     l2    (:wat::telemetry::Log :namespace "sift-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:usr::Other :x 1)))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1 l2)))
     sh    (:usr::my-sift'/start :locus (:wat::spawn::thread)
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:usr::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-rules-fatal-ns" :time-lo 0 :time-hi 100000 :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:usr::my-sift::SiftRulesResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; ── PROCESS locus — same fail-closed guard, across a FORK. ──
(:wat::core::defn :user::sift-rules-fatal-process [] -> :wat::core::bool
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "sift-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:usr::Temp :c 60)))
     l2    (:wat::telemetry::Log :namespace "sift-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:usr::Other :x 1)))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1 l2)))
     sh    (:usr::my-sift'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:usr::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:usr::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:usr::my-sift/sift-rules svc
             (:usr::my-sift::SiftRulesRequest :namespace "sift-rules-fatal-ns" :time-lo 0 :time-hi 100000 :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:usr::my-sift::SiftRulesResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
