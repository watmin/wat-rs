;; Co-located fixture for probe_arc278_journal_service_logs.rs — arc 278 T1b.2 write-logs coverage.
;;
;; The write-LOGS half of journal' (symmetric to write-metrics): journal' given a mem-store',
;; write-logs a 1-Log batch, a separate client scans back, return the stored `data`. The Log's
;; message is OPAQUE (arc 278 Stone B): the producer `edn::write`s its payload record at the call
;; site, so a plain String crosses the wire and is stored verbatim.

;; a concrete log payload — a user record the producer `edn::write`s into the opaque message String.
(:wat::core::defrecord :user::PriceEvent
  [asset <- :wat::core::keyword
   price <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [sh      (:wat::query::mem-store/start :locus (:wat::spawn::thread)
               :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     saddr   (:wat::query::mem-store::Handle/addr sh)
     jh      (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry::journal::Record) :store-addr saddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     msg     (:wat::edn::write (:user::PriceEvent :asset :BTC :price 100000))
     l       (:wat::telemetry::Log
               :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags :time-ns 456
               :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info :message msg)
     batch   (:wat::core::Vector :- [:wat::telemetry::Log] l)
     _wr     (:wat::telemetry::Journal/write-logs journal
               (:wat::telemetry::Journal::WriteLogsRequest batch))
     client  (:wat::core::match (:wat::kernel::connect saddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk      (:wat::edn::write (:wat::telemetry::PartitionKey
                                 :namespace "probe-ns" :kind :wat::telemetry::Kind::Log))
     resp    (:wat::query::Store/scan client
               (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::query::Row/data (:wat::core::first rows))
          "WRONG-ROW-COUNT"))
      (_ "SCAN-FAILED"))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
