;; :wat::std::telemetry::Console — render-and-print dispatcher
;; factory for use with :wat::std::telemetry::Service<E,G>.
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

(:wat::core::enum :wat::std::telemetry::Console::Format
  :Edn          ;; render via :wat::edn::write (compact, tagged — round-trip-safe via :wat::edn::read)
  :Json         ;; render via :wat::edn::write-json (compact, sentinel-tagged JSON — round-trip-safe)
  :Pretty       ;; render via :wat::edn::write-pretty (multi-line indented, tagged)
  :NoTagEdn     ;; render via :wat::edn::write-notag (compact, no struct/enum tag — lossy, human-friendly)
  :NoTagJson)   ;; render via :wat::edn::write-json-natural (natural JSON for ELK/DataDog ingestion — lossy)

;; The factory. Returns a closure that captures con-tx + format.
;; When the substrate Service calls dispatcher(entry), the closure:
;;   1. Renders entry to text via the format-selected wat-edn primitive
;;   2. Appends a newline (one entry, one line)
;;   3. Sends through Console/out (tagged stdout-write — fire-and-forget)
;;
;; Closure-over con-tx + format is wat-lambda-with-captured-environment
;; per the lambda contract documented in the runtime.

(:wat::core::define
  (:wat::std::telemetry::Console/dispatcher<E>
    (con-tx :wat::std::service::Console::Tx)
    (format :wat::std::telemetry::Console::Format)
    -> :fn(E)->())
  (:wat::core::lambda ((entry :E) -> :())
    (:wat::core::let*
      (((line :String)
        (:wat::core::match format -> :String
          (:wat::std::telemetry::Console::Format::Edn
            (:wat::edn::write entry))
          (:wat::std::telemetry::Console::Format::Json
            (:wat::edn::write-json entry))
          (:wat::std::telemetry::Console::Format::Pretty
            (:wat::edn::write-pretty entry))
          (:wat::std::telemetry::Console::Format::NoTagEdn
            (:wat::edn::write-notag entry))
          (:wat::std::telemetry::Console::Format::NoTagJson
            (:wat::edn::write-json-natural entry))))
       ((with-newline :String)
        (:wat::core::string::concat line "\n")))
      (:wat::std::service::Console/out con-tx with-newline))))
