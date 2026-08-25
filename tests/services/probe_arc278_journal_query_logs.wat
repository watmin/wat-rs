;; Co-located fixture for probe_arc278_journal_query_logs.rs — arc 278 T2: query-logs (thread).
;; Symmetric to journal_query (metrics): write 2 Logs (t=1s,2s), query-logs BROAD [0,3s] -> 2,
;; NARROW [1.5s,3s] -> 1. Returns 2*10+1 = 21. Closes the un-probed query-logs gap.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     l1    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:probe::Note :text "one")))
     l2    (:wat::telemetry::Log :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2000000000 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Warn
             :message (:wat::edn::write (:probe::Note :text "two")))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry::Log l1 l2)))
     bq    (:wat::telemetry::Journal/query-logs journal
             (:wat::telemetry::Journal::QueryLogsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     nq    (:wat::telemetry::Journal/query-logs journal
             (:wat::telemetry::Journal::QueryLogsRequest :namespace "probe-ns"
               :time-lo 1500000000 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     bc    (:wat::core::match bq ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
             ((:wat::telemetry::Journal::QueryLogsResponse::Success ls _c) (:wat::core::count ls))
             (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     nc    (:wat::core::match nq ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
             ((:wat::telemetry::Journal::QueryLogsResponse::Success ls _c) (:wat::core::count ls))
             (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))]
    (:wat::core::+ (:wat::core::* bc 10) nc)))
