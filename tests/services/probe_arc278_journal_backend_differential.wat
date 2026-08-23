;; Co-located fixture for probe_arc278_journal_backend_differential.rs — arc 278 T1b.3.
;;
;; THE DIFFERENTIAL (thread): the store is a swappable CONFIG PARAM. The SAME journal' is run over
;; two backends — mem-store' (the oracle) and sqlite-store' (:memory:, the real backend) — selected
;; ONLY by which store's Address' is injected at start. Same write-metrics -> the two backends must
;; persist BIT-FOR-BIT identical rows. journal' is backend-blind (it names only the :wat::query::Store
;; surface); any future backend (mysql/mongo/dynamo/es/redis/wat-built) slots in the same way.

;; journal-roundtrip: GIVEN a store's Address' (the config param), start journal' against it, write a
;; Metric, then scan the store back through a fresh client and return the persisted `data`.
(:wat::core::defn :user::journal-roundtrip
  [store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [jh      (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
               :record (:wat::telemetry::journal::Record) :store-addr store-addr)
     journal (:wat::core::match (:wat::kernel::connect (:wat::telemetry::journal::Handle/addr jh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     m       (:wat::telemetry::Metric
               :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags :time-ns 123
               :start-time-ns 100 :name :requests :value (:wat::telemetry::Numeric::I64 7)
               :unit :wat::telemetry::Unit::Count)
     batch   (:wat::core::Vector :wat::telemetry::Metric m)
     _wr     (:wat::telemetry::Journal/write-metrics journal
               (:wat::telemetry::Journal::WriteMetricsRequest batch))
     client  (:wat::core::match (:wat::kernel::connect store-addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk      (:wat::edn::write (:wat::telemetry::PartitionKey
                                 :namespace "probe-ns" :kind :wat::telemetry::Kind::Metric))
     resp    (:wat::query::Store/scan client
               (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 10 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor)
        (:wat::core::if (:wat::core::= (:wat::core::count rows) 1)
          (:wat::query::Row/data (:wat::core::first rows))
          "WRONG-ROW-COUNT"))
      (_ "SCAN-FAILED"))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; run the SAME journal' over both backends; return the mem row's data IFF it equals the sqlite
;; row's data (the differential), else a mismatch sentinel the .rs catches.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh        (:wat::query::mem-store/start :locus (:wat::spawn::thread)
                  :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr      (:wat::query::mem-store::Handle/addr msh)
     ssh        (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
                  :record (:wat::query::sqlite-store::Record
                            :path ":memory:" :index-names (:wat::core::Vector :wat::core::String "by-uuid")))
     saddr      (:wat::query::sqlite-store::Handle/addr ssh)
     mem-data   (:user::journal-roundtrip maddr)
     sqlite-data (:user::journal-roundtrip saddr)]
    (:wat::core::if (:wat::core::= mem-data sqlite-data)
      mem-data
      "DIFFERENTIAL-MISMATCH")))
