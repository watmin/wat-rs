;; Co-located fixture for probe_arc278_journal_service_logs.rs — arc 278 T1b.2 write-logs coverage.
;;
;; The write-LOGS half of journal' (symmetric to write-metrics): journal' given a mem-store',
;; write-logs a 1-Log batch, a separate client scans back, return the stored `data`. The Log carries
;; a concrete payload record satisfying the OPEN :wat::telemetry'::LogMessage surface (any record).

;; a concrete log payload — a user record satisfying the open LogMessage surface.
(:wat::core::defrecord :user::PriceEvent
  [asset <- :wat::core::keyword
   price <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [sh      (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
               :record (:wat::query::mem-store'::Record :rows (:wat::core::PersistentVector)))
     saddr   (:wat::query::mem-store'::Handle/addr sh)
     jh      (:wat::telemetry'::journal'/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry'::journal'::Record) :store-addr saddr)
     journal (:wat::kernel::connect' (:wat::telemetry'::journal'::Handle/addr jh))
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     msg     (:user::PriceEvent :asset :BTC :price 100000)
     l       (:wat::telemetry'::Log
               :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags :time-ns 456
               :caller :evaluator :level :wat::telemetry'::Level::Info :message msg)
     batch   (:wat::core::Vector :wat::telemetry'::Log l)
     _wr     (:wat::telemetry'::Journal/write-logs journal
               (:wat::telemetry'::Journal::WriteLogsRequest batch))
     client  (:wat::kernel::connect' saddr)
     pk      (:wat::edn::write (:wat::telemetry'::PartitionKey
                                 :namespace "probe-ns" :kind :wat::telemetry'::Kind::Log))
     resp    (:wat::query::Store/scan client
               (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp -> :wat::core::String
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::query::Row/data (:wat::core::first rows))
          "WRONG-ROW-COUNT"))
      (_ "SCAN-FAILED"))))
