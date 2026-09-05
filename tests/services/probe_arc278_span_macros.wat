;; Co-located fixture for probe_arc278_span_macros.rs — arc 278 STONE Span.3 acceptance gate.
;;
;; The call-site ergonomics: `with-span` (inline open/use/close) + `timed` (the timing widget).
;; Inside one with-span: incr :requests TWICE + `timed` a body once. On the macro's close, span'
;; emits FOUR Metrics — :requests (ONE aggregated counter, value 2), :fetch/count, :fetch/duration,
;; and one :fetch/sample. A client then scans the store and returns the row count, which must be 4
;; (proving with-span opened+closed, timed fed Span/timed, incr aggregated rather than fanned out,
;; and close emits fidelity samples).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     ;; the whole caller surface: a sink addr + a fresh span; no open/close by hand.
     _ws   (:wat::telemetry::with-span span jaddr "probe-ns" tags
             (:wat::core::do
               (:wat::core::match (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
                 ((:wat::kernel::RecvOutcome::Message _resp) nil)
                 ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
               (:wat::core::match (:wat::telemetry::Span/incr span (:wat::telemetry::Span::IncrRequest :name :requests))
                 ((:wat::kernel::RecvOutcome::Message _resp) nil)
                 ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message _c) :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
               (:wat::telemetry::timed span :fetch 42)))
     client (:wat::core::match (:wat::kernel::connect maddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     pk    (:wat::edn::write (:wat::telemetry::PartitionKey
                               :namespace "probe-ns" :kind :wat::telemetry::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows _cursor) (:wat::core::count rows))
      (_ -1))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
