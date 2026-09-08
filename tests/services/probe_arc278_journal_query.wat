;; Co-located fixture for probe_arc278_journal_query.rs — arc 278 T2: the CloudWatch READ side.
;;
;; Write 2 Metrics (at t=1s and t=2s) into namespace "probe-ns", then query-metrics back:
;;   - a BROAD window [0, 3s] returns BOTH (2),
;;   - a NARROW window [1.5s, 3s] returns ONE (the t=2s metric; the t=1s one is filtered out).
;; Returns broad*10 + narrow = 21 — proving read-back (scan + hydrate) AND time-range filtering.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     m1    (:wat::telemetry::Metric :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1000000000 :start-time-ns 0 :name :a
             :value (:wat::telemetry::Numeric::I64 {:val 1}) :unit :wat::telemetry::Unit::Count)
     m2    (:wat::telemetry::Metric :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2000000000 :start-time-ns 0 :name :b
             :value (:wat::telemetry::Numeric::I64 {:val 2}) :unit :wat::telemetry::Unit::Count)
     _wr   (:wat::telemetry::Journal/write-metrics journal
             (:wat::telemetry::Journal::WriteMetricsRequest (:wat::core::Vector :- [:wat::telemetry::Metric] m1 m2)))
     bq    (:wat::telemetry::Journal/query-metrics journal
             (:wat::telemetry::Journal::QueryMetricsRequest :namespace "probe-ns"
               :time-lo 0 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     nq    (:wat::telemetry::Journal/query-metrics journal
             (:wat::telemetry::Journal::QueryMetricsRequest :namespace "probe-ns"
               :time-lo 1500000000 :time-hi 3000000000 :limit 100 :cursor :wat::core::None))
     bc    (:wat::core::match bq [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv
             [:wat::telemetry::Journal::QueryMetricsResponse::Success {:metrics ms :cursor _c} (:wat::core::count ms)]
             [_ -1])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
     nc    (:wat::core::match nq [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv
             [:wat::telemetry::Journal::QueryMetricsResponse::Success {:metrics ms :cursor _c} (:wat::core::count ms)]
             [_ -1])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])]
    (:wat::core::+ (:wat::core::* bc 10) nc)))
