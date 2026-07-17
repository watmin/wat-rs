;; Co-located fixture for probe_arc278_journal_service.rs — arc 278 STONE T1b.2 acceptance gate.
;;
;; The composition of everything the groundwork proved: `journal'` (a defservice holding a
;; `:wat::query::Store` peer) is GIVEN a `mem-store'`, `write-metrics` a 1-Metric batch; then a
;; SEPARATE client scans the same store back and we return the stored row's `data` (the Metric's
;; tagged EDN). The .rs golden-compares it — proving the whole write path end-to-end.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [;; the backend store
     sh      (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
               :record (:wat::query::mem-store'::Record :rows (:wat::core::PersistentVector)))
     saddr   (:wat::query::mem-store'::Handle/addr sh)
     ;; journal', GIVEN the store's addr (dials it in :init, holds it in :ephemeral)
     jh      (:wat::telemetry'::journal'/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry'::journal'::Record) :store-addr saddr)
     journal (:wat::kernel::connect' (:wat::telemetry'::journal'::Handle/addr jh))
     ;; a test metric (same shape as the metric_edn golden — deterministic namespace -> deterministic pk)
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     m       (:wat::telemetry'::Metric
               :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags :time-ns 123
               :start-time-ns 100 :name :requests :value (:wat::telemetry'::Numeric::I64 7)
               :unit :wat::telemetry'::Unit::Count)
     batch   (:wat::core::Vector :wat::telemetry'::Metric m)
     _wr     (:wat::telemetry'::Journal/write-metrics journal
               (:wat::telemetry'::Journal::WriteMetricsRequest batch))
     ;; verify through a SEPARATE client on the same store
     client  (:wat::kernel::connect' saddr)
     pk      (:wat::edn::write (:wat::telemetry'::PartitionKey
                                 :namespace "probe-ns" :kind :wat::telemetry'::Kind::Metric))
     resp    (:wat::query::Store/scan client
               (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp -> :wat::core::String
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::query::Row/data (:wat::core::first rows))
          "WRONG-ROW-COUNT"))
      (_ "SCAN-FAILED"))))
