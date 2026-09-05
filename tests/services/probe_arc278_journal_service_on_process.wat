;; Co-located fixture for probe_arc278_journal_service_on_process.rs — arc 278 T1b.2 loci parity.
;;
;; The FORK path of journal' T1b.2: both `mem-store'` and `journal'` fork to PROCESSES. journal'
;; (a process child) must dial mem-store' (another process child), so journal's pid is granted to
;; mem-store's accept-gate via a `post-spawn` hook BEFORE journal's :init dials (grant-before-dial,
;; from probe_arc278_s2s_peer_on_process). The verifying `client` is the OWNER (parent) connecting
;; to its own mem-store' child — no grant needed (as in probe_arc278_mem_store_on_process).
;;
;; Proves journal' — a reserved-ns, peer-holding service — round-trips write-metrics across the
;; fork exactly as on a thread. Returns the stored row's `data`; the .rs golden-compares it.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [sh      (:wat::query::mem-store/start :locus (:wat::spawn::process)
               :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     saddr   (:wat::query::mem-store::Handle/addr sh)
     ;; journal' on a PROCESS; the post-spawn hook grants journal's child pid to mem-store's gate
     ;; BEFORE journal''s :init dials the store (grant-before-dial ordering).
     jh      (:wat::telemetry::journal/start
               :locus (:wat::spawn::process/post-spawn
                        (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                          (:wat::query::mem-store/grant sh
                            (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
               :record (:wat::telemetry::journal::Record) :store-addr saddr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     m       (:wat::telemetry::Metric
               :namespace "probe-ns" :uuid (:wat::uuid::nil) :tags tags :time-ns 123 :event-id (:wat::uuid::nil)
               :start-time-ns 100 :name :requests :value (:wat::telemetry::Numeric::I64 7)
               :unit :wat::telemetry::Unit::Count)
     batch   (:wat::core::Vector :- [:wat::telemetry::Metric] m)
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
      (_ "SCAN-FAILED"))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
