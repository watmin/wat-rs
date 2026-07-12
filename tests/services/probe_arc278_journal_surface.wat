;; Co-located fixture for probe_arc278_journal_surface.rs — arc 278 stone T1b.1 acceptance gate.
;;
;; A throwaway toy `:wat::telemetry'::Journal` satisfier (NOT `journal'`, that is stone T1b.2)
;; proving the surface freezes, is satisfiable via `:satisfies`, and replies through the wire —
;; mirrors `mem-store'`'s satisfaction of `Store` (wat/query/mem.wat).
(:wat::service::defservice :probe::toy-journal'
  :satisfies :wat::telemetry'::Journal
  :durable   []
  :ephemeral []
  :impls
  [(write-metrics [s req]
     (:wat::service::Outcome::Reply s (:wat::telemetry'::Journal::WriteMetricsResponse::Success)))
   (write-logs [s req]
     (:wat::service::Outcome::Reply s (:wat::telemetry'::Journal::WriteLogsResponse::Success)))])

;; `:probe::run` — start the toy on a thread, dial it, call `write-metrics` with a 1-element
;; `Metric` batch, and return the raw response (the .rs asserts it is `WriteMetricsResponse::Success`).
(:wat::core::defn :probe::run [] -> :wat::telemetry'::Journal::WriteMetricsResponse
  (:wat::core::let
    [h       (:probe::toy-journal'/start :locus (:wat::spawn::thread) :record (:probe::toy-journal'::Record))
     journal (:wat::kernel::connect' (:probe::toy-journal'::Handle/addr h))
     tags    (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     m       (:wat::telemetry'::Metric
               "probe-ns"                          ;; namespace     (spliced from Scope)
               (:wat::core::Uuid/nil)               ;; uuid          (spliced)
               tags                                 ;; tags          (spliced)
               123                                  ;; time-ns       (spliced)
               100                                  ;; start-time-ns (own)
               :requests                            ;; name          (own)
               (:wat::telemetry'::Numeric::I64 7)   ;; value         (own)
               :wat::telemetry'::Unit::Count)        ;; unit          (own)
     batch   (:wat::core::Vector :wat::telemetry'::Metric m)]
    (:wat::telemetry'::Journal/write-metrics journal (:wat::telemetry'::Journal::WriteMetricsRequest batch))))
