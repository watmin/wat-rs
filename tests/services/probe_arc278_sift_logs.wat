;; Co-located fixture for probe_arc278_sift_logs.rs — arc 278 Stone 2: the `sift-logs` op RED
;; gate, proven on BOTH loci (R31/R32 SATISFACTIO LIMEN TRANSIT / QVANTVMVIS PROCVL IDEM NEXVS —
;; loci-agnostic is NON-NEGOTIABLE; thread-only would be a failure). THREAD: the compile-once/
;; apply-per-row filter loop in-process. PROCESS: the SAME sift across a FORK — the Sieve's
;; ::-source String + the opaque Log messages cross the wire, journal' dials mem-store' via
;; grant-before-dial (the post-spawn hook grants journal's child pid to the store's gate before
;; :init dials), the predicate evals in the child. Same ops → same result on both loci.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

;; a mixed page of 3 Logs (Info/Error/Warn); a PURE predicate (level = :error) must return ONLY
;; the survivor (count 1) — the other 2 are filtered server-side.
(:wat::core::defn :user::sift-pure-survivors [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:probe::Note :text "info-one")))
     l2    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Error
             :message (:wat::edn::write (:probe::Note :text "error-one")))
     l3    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 3000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Warn
             :message (:wat::edn::write (:probe::Note :text "warn-one")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1 l2 l3)))
     sieve (:wat::query::sieve-pred
             (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
               (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))
     sr    (:wat::telemetry::Journal/sift-logs journal
             (:wat::telemetry::Journal::SiftLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 4000000000 :limit 100 :cursor :wat::core::None :sieve sieve))]
    (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::SiftLogsResponse::Success ls _c) (:wat::core::count ls))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; an IMPURE predicate (fn body does IO) must be REJECTED — ::Fatal, never a silent pass
;; (no-hidden-failures floor). Returns true iff the response is ::Fatal.
(:wat::core::defn :user::sift-impure-rejected [] -> :wat::core::bool
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Error
             :message (:wat::edn::write (:probe::Note :text "one")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1)))
     sieve (:wat::query::sieve-pred
             (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
               (:wat::core::do (:wat::kernel::println "impure predicate side effect") true)))
     sr    (:wat::telemetry::Journal/sift-logs journal
             (:wat::telemetry::Journal::SiftLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 4000000000 :limit 100 :cursor :wat::core::None :sieve sieve))]
    (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::SiftLogsResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ── PROCESS locus — the loci-agnostic proof (R31/R32). SAME scenario as the thread fns above,
;; across a FORK: mem-store' + journal' both on processes, journal' dialing mem-store' via
;; grant-before-dial. The Sieve's ::-source String crosses to the journal' child; the opaque Log
;; messages cross; the predicate is read-string'd + eval-ast!'d + applied IN THE CHILD. Same ops →
;; same result: a pure `level = :error` predicate returns ONLY the survivor (count 1). ──
(:wat::core::defn :user::sift-pure-survivors-process [] -> :wat::core::i64
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
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:probe::Note :text "info-one")))
     l2    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Error
             :message (:wat::edn::write (:probe::Note :text "error-one")))
     l3    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 3000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Warn
             :message (:wat::edn::write (:probe::Note :text "warn-one")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1 l2 l3)))
     sieve (:wat::query::sieve-pred
             (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
               (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))
     sr    (:wat::telemetry::Journal/sift-logs journal
             (:wat::telemetry::Journal::SiftLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 4000000000 :limit 100 :cursor :wat::core::None :sieve sieve))]
    (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::SiftLogsResponse::Success ls _c) (:wat::core::count ls))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; the impure-reject, across a FORK — the no-hidden-failures floor holds loci-agnostically: an
;; impure predicate is REJECTED (::Fatal) in the child too, never a silent pass.
(:wat::core::defn :user::sift-impure-rejected-process [] -> :wat::core::bool
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
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Error
             :message (:wat::edn::write (:probe::Note :text "one")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1)))
     sieve (:wat::query::sieve-pred
             (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
               (:wat::core::do (:wat::kernel::println "impure predicate side effect") true)))
     sr    (:wat::telemetry::Journal/sift-logs journal
             (:wat::telemetry::Journal::SiftLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 4000000000 :limit 100 :cursor :wat::core::None :sieve sieve))]
    (:wat::core::match sr ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::SiftLogsResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
