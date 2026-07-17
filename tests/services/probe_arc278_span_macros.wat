;; Co-located fixture for probe_arc278_span_macros.rs — arc 278 STONE Span.3 acceptance gate.
;;
;; The call-site ergonomics: `with-span` (inline open/use/close) + `timed` (the timing widget).
;; Inside one with-span: incr :requests TWICE + `timed` a body once. On the macro's close, span'
;; emits exactly THREE Metrics — :requests (ONE aggregated counter, value 2), :fetch/count, and
;; :fetch/duration. A client then scans the store and returns the row count, which must be 3
;; (proving with-span opened+closed, timed fed Span/timed, and incr aggregated rather than fanned out).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store'::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store'::Handle/addr msh)
     jh    (:wat::telemetry'::journal'/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry'::journal'::Record) :store-addr maddr)
     jaddr (:wat::telemetry'::journal'::Handle/addr jh)
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     ;; the whole caller surface: a sink addr + a fresh span; no open/close by hand.
     _ws   (:wat::telemetry'::with-span span jaddr "probe-ns" tags
             (:wat::core::do
               (:wat::telemetry'::Span/incr span (:wat::telemetry'::Span::IncrRequest :name :requests))
               (:wat::telemetry'::Span/incr span (:wat::telemetry'::Span::IncrRequest :name :requests))
               (:wat::telemetry'::timed span :fetch 42)))
     client (:wat::kernel::connect' maddr)
     pk    (:wat::edn::write (:wat::telemetry'::PartitionKey
                               :namespace "probe-ns" :kind :wat::telemetry'::Kind::Metric))
     resp  (:wat::query::Store/scan client
             (:wat::query::Store::ScanRequest :pk pk :sk-lo "#" :sk-hi "#z" :limit 20 :cursor :wat::core::None))]
    (:wat::core::match resp -> :wat::core::i64
      ((:wat::query::Store::ScanResponse::Success rows _cursor) (:wat::core::count rows))
      (_ -1))))
