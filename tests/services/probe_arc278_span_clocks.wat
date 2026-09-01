;; Co-located fixture for probe_arc278_span_clocks.rs — arc 278 item (c) stone B.
;; Two independent cadences. Time arrives as I/O: poll-until on observed store counts;
;; nap is select' on a one-shot after — never a sleep-then-assert.

(:wat::core::defn :probe::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done)))
    ((:wat::spawn::ServiceEvent::Message _i _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _i) nil)
    ((:wat::spawn::ServiceEvent::Lost _i _c) nil)
    ((:wat::spawn::ServiceEvent::Malformed _i _c) nil)
    ((:wat::spawn::ServiceEvent::Rejected _i _c) nil)
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _p) nil)
    ((:wat::spawn::ServiceEvent::Admin _m) nil)))

(:wat::core::defn :probe::count-kind
  [client <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   ns     <- :wat::core::String
   kind   <- :wat::telemetry::Kind]
  -> :wat::core::i64
  (:wat::core::let
    [pk   (:wat::edn::write (:wat::telemetry::PartitionKey :namespace ns :kind kind))
     resp (:wat::query::Store/scan client
            (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:wat::query::Store::ScanResponse::Success rows _cursor) (:wat::core::count rows))
          (_ -1)))
      (_ -1))))

(:wat::core::defn :probe::poll-until-kind
  [client  <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   ns      <- :wat::core::String
   kind    <- :wat::telemetry::Kind
   target  <- :wat::core::i64
   attempts <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -2
    (:wat::core::let
      [n (:probe::count-kind client ns kind)]
      (:wat::core::if (:wat::i64::>= n target)
        n
        (:wat::core::let [_ (:probe::nap 10)]
          (:probe::poll-until-kind client ns kind target (:wat::i64::- attempts 1)))))))

;; Row 1: ONLY logs. Logs cadence 20ms, metrics cadence 2000ms. Observe ≥1 log row and 0 metric rows.
(:wat::core::defn :user::only-logs [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "only-logs" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 20
                :metrics-flush-after-ms 2000)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _l    (:wat::telemetry::log span :wat::telemetry::Level::Info "tick")
     nlog  (:probe::poll-until-kind client "only-logs" :wat::telemetry::Kind::Log 1 80)
     nmet  (:probe::count-kind client "only-logs" :wat::telemetry::Kind::Metric)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= nlog 1) (:wat::core::= nmet 0)) 1 0)))

;; Row 2: ONLY counts. Metrics cadence 20ms, logs cadence 2000ms.
(:wat::core::defn :user::only-counts [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "only-counts" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 2000
                :metrics-flush-after-ms 20)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _i    (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
     nmet  (:probe::poll-until-kind client "only-counts" :wat::telemetry::Kind::Metric 1 80)
     nlog  (:probe::count-kind client "only-counts" :wat::telemetry::Kind::Log)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= nmet 1) (:wat::core::= nlog 0)) 1 0)))

;; Row 3: re-arm. Log, wait for first flush, log again, wait for second. No Span/flush.
(:wat::core::defn :user::rearm [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "rearm-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 20
                :metrics-flush-after-ms 2000)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _l1   (:wat::telemetry::log span :wat::telemetry::Level::Info "a")
     n1    (:probe::poll-until-kind client "rearm-ns" :wat::telemetry::Kind::Log 1 80)
     _l2   (:wat::telemetry::log span :wat::telemetry::Level::Info "b")
     n2    (:probe::poll-until-kind client "rearm-ns" :wat::telemetry::Kind::Log 2 80)]
    n2))

;; Row 4: idle span. Never logs or counts. Several 20ms intervals → zero writes.
(:wat::core::defn :user::idle [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     span-rec (:wat::telemetry::span::Record
                :namespace "idle-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 20
                :metrics-flush-after-ms 20)
     sph   (:wat::telemetry::span/start :locus (:wat::spawn::thread)
             :record span-rec :sink-addr jaddr)
     span  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr sph)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     ;; Observe for several intervals: if any write appears, fail immediately.
     seen (:probe::poll-until-kind client "idle-ns" :wat::telemetry::Kind::Log 1 15)
     nmet (:probe::count-kind client "idle-ns" :wat::telemetry::Kind::Metric)]
    ;; poll-until returns -2 when attempts exhausted without reaching target — that is SUCCESS here.
    (:wat::core::if (:wat::core::and (:wat::core::= seen -2) (:wat::core::= nmet 0)) 1 0)))

;; Row 10: non-default interval honoured. Fast span (20ms) flushes before slow (2000ms).
(:wat::core::defn :user::cadence [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     fast-rec (:wat::telemetry::span::Record
                :namespace "fast-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 20
                :metrics-flush-after-ms 2000)
     slow-rec (:wat::telemetry::span::Record
                :namespace "slow-ns" :uuid (:wat::uuid::nil) :tags tags :start-time-ns 0
                :counters (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                :durations (:wat::core::HashMap :- [:wat::core::keyword :wat::telemetry::Samples])
                :logs (:wat::core::Vector :- [:wat::telemetry::Log])
                :logs-flush-after-ms 2000
                :metrics-flush-after-ms 2000)
     fasth (:wat::telemetry::span/start :locus (:wat::spawn::thread) :record fast-rec :sink-addr jaddr)
     slowh (:wat::telemetry::span/start :locus (:wat::spawn::thread) :record slow-rec :sink-addr jaddr)
     fast  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr fasth)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     slow  (:wat::core::match (:wat::kernel::connect (:wat::telemetry::span::Handle/addr slowh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _lf   (:wat::telemetry::log fast :wat::telemetry::Level::Info "fast")
     _ls   (:wat::telemetry::log slow :wat::telemetry::Level::Info "slow")
     nfast (:probe::poll-until-kind client "fast-ns" :wat::telemetry::Kind::Log 1 80)
     nslow (:probe::count-kind client "slow-ns" :wat::telemetry::Kind::Log)]
    (:wat::core::if (:wat::core::and (:wat::i64::>= nfast 1) (:wat::core::= nslow 0)) 1 0)))
