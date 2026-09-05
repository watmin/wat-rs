;; Co-located fixture for probe_ex001_journal_same_ns.rs — excursus 001 SORTKEY.
;;
;; THE GATE the journal census proposed and stopped short of: three Metrics at ONE time-ns,
;; distinct event-ids, written through journal' into mem-store AND sqlite-store(:memory:),
;; read back via query-metrics (the hydration path). All three must survive on both
;; backends. Pre-SortKey this sequence stored 1 row (last-wins by time-only sk).
;;
;; Shape copied from tests/services/probe_arc278_journal_backend_differential.wat:
;; helper parameterized on the store Address, both stores started, run both, compare.

(:wat::core::defn :user::eid [s <- :wat::core::String] -> :wat::core::Uuid
  (:wat::core::Option/expect (:wat::uuid::from-string s) "canonical uuid"))

(:wat::core::defn :user::metric
  [name <- :wat::core::keyword  eid <- :wat::core::Uuid]
  -> :wat::telemetry::Metric
  (:wat::core::let [tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])]
    (:wat::telemetry::Metric
      :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags
      :time-ns 1000000000 :event-id eid
      :start-time-ns 0 :name name
      :value (:wat::telemetry::Numeric::I64 1) :unit :wat::telemetry::Unit::Count)))

(:wat::core::defn :user::join-names
  [ms <- (:wat::core::Vector :- [:wat::telemetry::Metric])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String m <- :wat::telemetry::Metric] -> :wat::core::String
      (:wat::core::let [n (:wat::keyword::to-string (:wat::telemetry::Metric/name m))]
        (:wat::core::if (:wat::core::= acc "")
          n
          (:wat::string::concat acc (:wat::string::concat "," n)))))
    ""
    ms))

;; write three same-ns Metrics, query-metrics [0, 2s], return "count=N;names=…"
(:wat::core::defn :user::same-ns-roundtrip
  [store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [jh (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
           :record (:wat::telemetry::journal::Record) :store-addr store-addr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh))
               ((:wat::kernel::ConnectOutcome::Connected p) p)
               ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
               ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
               ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     m1 (:user::metric :a (:user::eid "00000000-0000-0000-0000-000000000001"))
     m2 (:user::metric :b (:user::eid "00000000-0000-0000-0000-000000000002"))
     m3 (:user::metric :c (:user::eid "00000000-0000-0000-0000-000000000003"))
     _wr (:wat::telemetry::Journal/write-metrics journal
           (:wat::telemetry::Journal::WriteMetricsRequest
             (:wat::core::Vector :- [:wat::telemetry::Metric] m1 m2 m3)))
     q (:wat::telemetry::Journal/query-metrics journal
         (:wat::telemetry::Journal::QueryMetricsRequest
           :namespace "probe-ns" :time-lo 0 :time-hi 2000000000 :limit 10 :cursor :wat::core::None))]
    (:wat::core::match q
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat::telemetry::Journal::QueryMetricsResponse::Success ms _c)
            (:wat::core::format "count={n};names={names}"
              :n (:wat::core::count ms)
              :names (:user::join-names ms)))
          (_ "QUERY-FAILED")))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     ssh   (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::sqlite-store::Record
                       :path ":memory:" :index-names (:wat::core::Vector :- [:wat::core::String] "by-uuid")))
     saddr (:wat::query::sqlite-store::Handle/addr ssh)
     mem   (:user::same-ns-roundtrip maddr)
     sql   (:user::same-ns-roundtrip saddr)]
    (:wat::core::if (:wat::core::= mem sql)
      mem
      (:wat::core::format "DIFFERENTIAL-MISMATCH mem={mem} sqlite={sql}" :mem mem :sql sql))))
