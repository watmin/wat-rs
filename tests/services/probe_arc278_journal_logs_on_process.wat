;; Co-located fixture for probe_arc278_journal_logs_on_process.rs — arc 278 T2 loci parity.
;; write-logs AND query-logs across a FORK: journal' + mem-store' on processes (grant-before-dial).
;; Write 2 Logs, then query-logs [0,3s]; the forked journal' scans + hydrates Logs in the child and
;; the response crosses the wire back. Returns the count (must be 2). Closes write-logs + query-logs
;; loci-parity holes in one shot.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [sh    (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     saddr (:wat::query::mem-store::Handle/addr sh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant sh
                          (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr saddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:probe::Note :text "one")))
     l2    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2000000000 :event-id (:wat::uuid::nil) :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Warn
             :message (:wat::edn::write (:probe::Note :text "two")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l1 l2)))
     bq    (:wat::telemetry::Journal/query-logs journal
             (:wat::telemetry::Journal::QueryLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))]
    (:wat::core::match bq ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::telemetry::Journal::QueryLogsResponse::Success ls _c) (:wat::core::count ls))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
