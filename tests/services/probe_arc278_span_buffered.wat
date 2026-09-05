;; Co-located fixture for probe_arc278_span_buffered.rs — arc 278 item (c) stone A.
;;
;; Double-count gate: incr :requests ×3, flush, incr ×2, close. Sum of emitted :requests
;; metrics must be exactly 5, never 8.

(:wat::core::defn :probe::sum-requests
  [rows <- (:wat::core::Vector :- [:wat::query::Row])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  row <- :wat::query::Row] -> :wat::core::i64
      (:wat::core::let
        [m (:wat::edn::read (:wat::query::Row/data row))
         nm (:wat::telemetry::Metric/name m)]
        (:wat::core::if (:wat::core::= nm :requests)
          (:wat::core::match (:wat::telemetry::Metric/value m)
            ((:wat::telemetry::Numeric::I64 n) (:wat::core::+ acc n))
            (_ acc))
          acc)))
    0 rows))

(:wat::core::defn :user::double-count [] -> :wat::core::i64
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
                :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS
                :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
                :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i1   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _i2   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _i3   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _f    (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     _i4   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _i5   (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "probe-ns" :kind :wat::telemetry::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor)
            (:probe::sum-requests rows))
          (_ -2)))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; Row 6+7: flush, flush again with nothing new (second emits nothing), close with nothing new
;; (Done, no extra rows). After incr×1 + two flushes + close, exactly one :requests row of 1.
(:wat::core::defn :user::flush-empty [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "empty-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
                :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS
                :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
                :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i    (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     _f1   (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     _f2   (:wat::telemetry::Span/flush span (:wat::telemetry::Span::FlushRequest))
     _c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "empty-ns" :kind :wat::telemetry::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor)
            (:wat::core::count rows))
          (_ -2)))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; Rows 4+5: one timed set of three samples 10, 20, 30. Expect /count=3, /duration=60,
;; and three /sample values {10,20,30}. Returns 1 on success.
(:wat::core::defn :user::duration-fidelity [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "dur-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
                :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS
                :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
                :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _t1   (:wat::telemetry::Span/timed span (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 10))
     _t2   (:wat::telemetry::Span/timed span (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 20))
     _t3   (:wat::telemetry::Span/timed span (:wat::telemetry::Span::TimedRequest :name :fetch :nanos 30))
     _c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "dur-ns" :kind :wat::telemetry::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor)
            ;; 5 rows = :fetch/count + :fetch/duration + 3 :fetch/sample.
            ;; Packed: 1000*nrows + sum of all I64 values. 3+60+10+20+30 = 123 → 5123.
            (:wat::core::let
              [nrows (:wat::core::count rows)
               total
               (:wat::core::foldl
                 (:wat::core::fn [acc <- :wat::core::i64  row <- :wat::query::Row] -> :wat::core::i64
                   (:wat::core::let
                     [m (:wat::edn::read (:wat::query::Row/data row))]
                     (:wat::core::match (:wat::telemetry::Metric/value m)
                       ((:wat::telemetry::Numeric::I64 n) (:wat::core::+ acc n))
                       (_ acc))))
                 0 rows)]
              (:wat::core::+ (:wat::core::* nrows 1000) total)))
          (_ -2)))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; Rows 2+3: five logs, close. All five land in the store, in order. A write-through-per-line
;; implementation still gets 5 rows; the batching proof is that they survive until one close-flush
;; (and the double-count test already forces the shared path). Order is the row-3 gate.
(:wat::core::defn :user::logs-survive [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "log-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms :wat::telemetry::span::DEFAULT-LOGS-FLUSH-AFTER-MS
                :metrics-flush-after-ms :wat::telemetry::span::DEFAULT-METRICS-FLUSH-AFTER-MS
                :logs-max :wat::telemetry::span::DEFAULT-LOGS-MAX
                :duration-samples-max :wat::telemetry::span::DEFAULT-DURATION-SAMPLES-MAX)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _l1   (:wat::telemetry::log span :wat::telemetry::Level::Info "one")
     _l2   (:wat::telemetry::log span :wat::telemetry::Level::Info "two")
     _l3   (:wat::telemetry::log span :wat::telemetry::Level::Info "three")
     _l4   (:wat::telemetry::log span :wat::telemetry::Level::Info "four")
     _l5   (:wat::telemetry::log span :wat::telemetry::Level::Info "five")
     _c    (:wat::telemetry::Span/close span (:wat::telemetry::Span::CloseRequest))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "log-ns" :kind :wat::telemetry::Kind::Log))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor)
            (:wat::core::count rows))
          (_ -2)))
      ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
