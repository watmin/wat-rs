;; Co-located fixture for probe_arc278_journal_service_sqlite_on_process.rs — arc 278 T1b.3 / U3.
;;
;; The REAL backend (sqlite) on a PROCESS fork — closes the deferred U3. Both sqlite-store' and
;; journal' fork to processes; sqlite-store' opens its OWN Connection in its :init inside the child
;; (THE CIRCUIT — a resource is opened by the worker that owns it; only the addr crosses the wire).
;; journal' (a process child) dials sqlite-store' (another process child) via grant-before-dial.
;;
;; Same golden as the mem-on-process + thread-differential tiers — so sqlite ≡ mem on a fork too.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [sh      (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
               :record (:wat::query::sqlite-store::Record
                         :path ":memory:" :index-names (:wat::core::Vector :wat::core::String "by-uuid")))
     saddr   (:wat::query::sqlite-store::Handle/addr sh)
     ;; journal' on a PROCESS; grant journal's child pid to sqlite-store's gate before :init dials.
     jh      (:wat::telemetry::journal/start
               :locus (:wat::spawn::process/post-spawn
                        (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                          (:wat::query::sqlite-store/grant sh
                            (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
               :record (:wat::telemetry::journal::Record) :store-addr saddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     m       (:wat::telemetry::Metric
               :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags :time-ns 123
               :start-time-ns 100 :name :requests :value (:wat::telemetry::Numeric::I64 7)
               :unit :wat::telemetry::Unit::Count)
     batch   (:wat::core::Vector :wat::telemetry::Metric m)
     _wr     (:wat::telemetry::Journal/write-metrics journal
               (:wat::telemetry::Journal::WriteMetricsRequest batch))
     client  (:wat::core::match (:wat::kernel::connect saddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk      (:wat::edn::write (:wat::telemetry::PartitionKey
                                 :namespace "probe-ns" :kind :wat::telemetry::Kind::Metric))
     resp    (:wat::query::Store/scan client
               (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::query::Row/data (:wat::core::first rows))
          "WRONG-ROW-COUNT"))
      (_ "SCAN-FAILED"))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
