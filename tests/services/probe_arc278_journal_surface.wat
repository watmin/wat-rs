;; Co-located fixture for probe_arc278_journal_surface.rs — arc 278 stone T1b.1 acceptance gate.
;;
;; A throwaway toy `:wat::telemetry'::Journal` satisfier (NOT `journal'`, that is stone T1b.2)
;; proving the surface freezes, is satisfiable via `:satisfies`, and replies through the wire —
;; mirrors `mem-store'`'s satisfaction of `Store` (wat/query/mem.wat).
(:wat::service::defservice :probe::toy-journal
  :satisfies :wat::telemetry::Journal
  :durable   []
  :ephemeral []
  :impls
  [(write-metrics [s ctx req]
     (:wat::service::Outcome::Reply s (:wat::telemetry::Journal::WriteMetricsResponse::Success)))
   (write-logs [s ctx req]
     (:wat::service::Outcome::Reply s (:wat::telemetry::Journal::WriteLogsResponse::Success)))
   (query-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (query-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::QueryLogsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Log]) :wat::core::None)))
   ;; arc 278 Stone 2 — sift-logs/sift-metrics widened the Journal surface; the toy must
   ;; implement every feature to satisfy it (mirrors the query-* stubs above; the sieve is
   ;; unused by this throwaway toy).
   (sift-metrics [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftMetricsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Metric]) :wat::core::None)))
   (sift-logs [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::telemetry::Journal::SiftLogsResponse::Success
         (:wat::core::Vector :- [:wat::telemetry::Log]) :wat::core::None)))])

;; `:probe::run` — start the toy on a thread, dial it, call `write-metrics` with a 1-element
;; `Metric` batch, and return the raw response (the .rs asserts it is `WriteMetricsResponse::Success`).
(:wat::core::defn :probe::run [] -> :wat::telemetry::Journal::WriteMetricsResponse
  (:wat::core::let
    [h       (:probe::toy-journal/start :locus (:wat::spawn::thread) :record (:probe::toy-journal::Record))
     journal (:wat::core::match (:wat::kernel::connect (:probe::toy-journal::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     ;; Arc 294 item (C) — kwargs construction of the spliced Metric (bare-positional retired).
     m       (:wat::telemetry::Metric
               :namespace     "probe-ns"                      ;; spliced from Scope
               :uuid          (:wat::uuid::nil)          ;; spliced
               :tags          tags                            ;; spliced
               :time-ns       123 :event-id (:wat::uuid::nil)                             ;; spliced
               :start-time-ns 100                             ;; own
               :name          :requests                       ;; own
               :value         (:wat::telemetry::Numeric::I64 7) ;; own
               :unit          :wat::telemetry::Unit::Count)    ;; own
     batch   (:wat::core::Vector :- [:wat::telemetry::Metric] m)]
    ;; arc 278 recv'-wall: the client-method returns a matchable RecvOutcome — unwrap the ::Message
    ;; to the inner WriteMetricsResponse (the .rs asserts the raw Success response, not a RecvOutcome).
    (:wat::core::match
      (:wat::telemetry::Journal/write-metrics journal (:wat::telemetry::Journal::WriteMetricsRequest batch))
      ((:wat::kernel::RecvOutcome::Message resp) resp)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
