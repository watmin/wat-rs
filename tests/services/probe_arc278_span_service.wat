;; Co-located fixture for probe_arc278_span_service.rs — arc 278 STONE Span.2 acceptance gate.
;;
;; The producer->sink->store chain: a real `span'` given a `journal'` (given a `mem-store'`),
;; incr :requests TWICE, close. On close, span' emits the counter as a Metric to the sink; the sink
;; persists it. A separate client scans the store, hydrates the one Metric, and returns its counter
;; value — which must be 2. (Asserts the deterministic value, not the whole record: close stamps a
;; nondeterministic time-ns.)

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
                :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i1   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _i2   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "probe-ns" :kind :wat::telemetry::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::core::let
            [m (:wat::edn::read (:wat::query::Row/data (:wat::core::first rows)))
             v (:wat::telemetry::Metric/value m)]
            (:wat::core::match v 
              ((:wat::telemetry::Numeric::I64 n) n)
              (_ -1)))
          -2))
      (_ -3))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
