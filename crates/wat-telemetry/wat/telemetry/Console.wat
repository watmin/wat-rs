;; :wat::telemetry::Console — render-and-print dispatcher
;; factory for use with :wat::telemetry::Service<E,G>.
;;
;; Arc 081. The substrate's Console destination for the telemetry
;; service contract. Composes:
;;   - arc 080's Service<E,G> (queue-fronted shell, generic over E)
;;   - arc 079's :wat::edn::write / write-json (renderer)
;;   - existing wat/std/service/Console.wat (Console::Tx → stdout)
;;
;; The Console destination is not a separate queue-fronted service.
;; It's a DISPATCHER FACTORY: given a Console::Tx and a format
;; choice, returns a closure that renders any entry as a single
;; line and sends it via Console/out.
;;
;; The format knob is picked once at factory-call time; per-line
;; cost is one render + one tagged-stdout send. No batching at the
;; print layer — one entry, one line. EDN's deterministic write
;; means each line is independently parseable.
;;
;; Usage shape:
;;
;;   (let* ((con-spawn (Console/spawn stdout stderr 2))
;;          (con-pool  (first con-spawn))
;;          (con-drv   (second con-spawn))
;;          (con-tx    (HandlePool::pop con-pool))
;;          (_finish   (HandlePool::finish con-pool))
;;          ;; Build the dispatcher.
;;          (dispatcher (Console/dispatcher con-tx :Edn))
;;          ;; Build a stats-translator (consumer-side; per arc 080's
;;          ;; "substrate ships zero entry variants" rule).
;;          (translator (...))
;;          (cadence    (Service/null-metrics-cadence))
;;          ;; Spawn the substrate Service with the Console-backed
;;          ;; dispatcher.
;;          (svc (Service/spawn 1 cadence dispatcher translator)))
;;     ...)

(:wat::core::enum :wat::telemetry::Console::Format
  :Edn          ;; render via :wat::edn::write (compact, tagged — round-trip-safe via :wat::edn::read)
  :Json         ;; render via :wat::edn::write-json (compact, sentinel-tagged JSON — round-trip-safe)
  :Pretty       ;; render via :wat::edn::write-pretty (multi-line indented, tagged)
  :NoTagEdn     ;; render via :wat::edn::write-notag (compact, no struct/enum tag — lossy, human-friendly)
  :NoTagJson)   ;; render via :wat::edn::write-json-natural (natural JSON for ELK/DataDog ingestion — lossy)


;; The shape `Console/dispatcher` returns and Service<E,G>'s
;; per-batch dispatch contract takes (arc 089 slice 3). Aliasing
;; spares every downstream signature from `:fn(wat::core::Vector<E>)->()`
;; nested inside another generic.
(:wat::core::typealias :wat::telemetry::Console::Dispatcher<E>
  :fn(wat::core::Vector<E>)->wat::core::unit)

;; The factory. Returns a closure that captures con-tx + format.
;; When the substrate Service calls dispatcher(entries), the closure
;; foldls each entry through the format-selected wat-edn primitive
;; and writes one line per entry through Console/out. Console-shaped
;; sinks are per-line by nature; the per-batch contract (arc 089
;; slice 3) doesn't change that — we just iterate inside the
;; dispatcher instead of having Service/loop iterate for us.
;;
;; Closure-over con-tx + format is wat-lambda-with-captured-environment
;; per the lambda contract documented in the runtime.

;; ─── Internal — render a single entry into the line shape ───────
;;
;; Lifted out of the dispatcher's lambda so the inner foldl reads
;; flat (one let* per function per memory `feedback_simple_forms_per_func`).
(:wat::core::define
  (:wat::telemetry::Console::render-line<E>
    (entry :E)
    (format :wat::telemetry::Console::Format)
    -> :wat::core::String)
  (:wat::core::let*
    (((line :wat::core::String)
      (:wat::core::match format -> :wat::core::String
        (:wat::telemetry::Console::Format::Edn
          (:wat::edn::write entry))
        (:wat::telemetry::Console::Format::Json
          (:wat::edn::write-json entry))
        (:wat::telemetry::Console::Format::Pretty
          (:wat::edn::write-pretty entry))
        (:wat::telemetry::Console::Format::NoTagEdn
          (:wat::edn::write-notag entry))
        (:wat::telemetry::Console::Format::NoTagJson
          (:wat::edn::write-json-natural entry)))))
    (:wat::core::string::concat line "\n")))


(:wat::core::define
  (:wat::telemetry::Console/dispatcher<E>
    (handle :wat::std::service::Console::Handle)
    (format :wat::telemetry::Console::Format)
    -> :wat::telemetry::Console::Dispatcher<E>)
  (:wat::core::lambda ((entries :wat::core::Vector<E>) -> :wat::core::unit)
    (:wat::core::foldl entries ()
      (:wat::core::lambda ((_acc :wat::core::unit) (entry :E) -> :wat::core::unit)
        (:wat::std::service::Console/out handle
          (:wat::telemetry::Console::render-line entry format))))))
