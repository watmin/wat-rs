;; examples/console-demo/wat/main.wat — ConsoleLogger walk-through.
;;
;; Wires the substrate's stdout-attached structured logger:
;;   - Console driver — N tagged-stdio writer threads (Console/spawn)
;;   - ConsoleLogger — closure over (con-tx, caller, clock, format)
;;
;; Producer calls `(ConsoleLogger/<level> logger entry)`. Per
;; emission, the substrate stamps `:wat::time::now`, identifies the
;; caller via the captured keyword, builds a 4-tuple, renders as EDN
;; (or JSON or Pretty per the captured format), routes to stdout for
;; :debug + :info or stderr for :warn + :error.
;;
;; Run:
;;   cargo run -p console-demo                 # shows stdout
;;   cargo run -p console-demo 2>&1 >/dev/null # shows stderr
;;   cargo run -p console-demo 2>err.log       # split streams

(:wat::config::set-capacity-mode! :error)


;; ─── Domain enum — what the trader emits as structured events ──

(:wat::core::enum :demo::Event
  (Buy
    (price :f64)
    (qty :i64))
  (Sell
    (price :f64)
    (qty :i64)
    (reason :String))
  (CircuitBreak
    (reason :String)))


;; ─── Producer body — emits all four levels ───────────────────────
;;
;; The producer doesn't see con-tx, caller-id, or clock. They're all
;; closed over inside the ConsoleLogger. Per emission: `(/info|/warn
;; |/error|/debug logger entry)` — caller never self-identifies.

(:wat::core::define
  (:demo::run
    (logger :wat::std::telemetry::ConsoleLogger)
    -> :())
  (:wat::core::let*
    (;; Routine flow → stdout
     ((_a :())
      (:wat::std::telemetry::ConsoleLogger/info logger
        (:demo::Event::Buy 100.5 7)))
     ((_b :())
      (:wat::std::telemetry::ConsoleLogger/info logger
        (:demo::Event::Sell 102.25 3 "stop-loss")))
     ;; Diagnostic detail → stdout
     ((_c :())
      (:wat::std::telemetry::ConsoleLogger/debug logger
        (:demo::Event::Buy 99.0 12)))
     ;; Concerning event → stderr
     ((_d :())
      (:wat::std::telemetry::ConsoleLogger/warn logger
        (:demo::Event::CircuitBreak "spike-volume")))
     ;; Failure → stderr
     ((_e :())
      (:wat::std::telemetry::ConsoleLogger/error logger
        (:demo::Event::CircuitBreak "exchange-disconnected"))))
    ()))


;; ─── Helper — build a logger with a chosen format ───────────────

(:wat::core::define
  (:demo::make-logger
    (con-tx :wat::std::service::Console::Tx)
    (caller :wat::core::keyword)
    (format :wat::std::telemetry::Console::Format)
    -> :wat::std::telemetry::ConsoleLogger)
  (:wat::std::telemetry::ConsoleLogger/new
    con-tx caller
    (:wat::core::lambda ((_u :()) -> :wat::time::Instant)
      (:wat::time::now))
    format))


;; ─── Wiring — owns Console driver. Per CIRCUIT.md.
;; Runs the producer body THREE times — once per format — so a single
;; `cargo run` shows EDN / JSON / Pretty side by side.

(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((con-spawn :wat::std::service::Console::Spawn)
      (:wat::std::service::Console/spawn stdout stderr 1))
     ((con-pool :wat::kernel::HandlePool<wat::std::service::Console::Tx>)
      (:wat::core::first con-spawn))
     ((con-driver :wat::kernel::ProgramHandle<()>)
      (:wat::core::second con-spawn))
     ((_inner :())
      (:wat::core::let*
        (((con-tx :wat::std::service::Console::Tx)
          (:wat::kernel::HandlePool::pop con-pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish con-pool))
         ;; ── EDN format (tagged, round-trip-safe) ──────────────
         ((_banner-edn :())
          (:wat::std::service::Console/out con-tx
            "\n=== :Edn (tagged, round-trip-safe) ===\n"))
         ((edn-logger :wat::std::telemetry::ConsoleLogger)
          (:demo::make-logger con-tx :market.observer
            :wat::std::telemetry::Console::Format::Edn))
         ((_run-edn :()) (:demo::run edn-logger))
         ;; ── NoTagEdn (lossy, human-friendly) ──────────────────
         ((_banner-notag-edn :())
          (:wat::std::service::Console/out con-tx
            "\n=== :NoTagEdn (lossy, human-friendly) ===\n"))
         ((notag-edn-logger :wat::std::telemetry::ConsoleLogger)
          (:demo::make-logger con-tx :market.observer
            :wat::std::telemetry::Console::Format::NoTagEdn))
         ((_run-notag-edn :()) (:demo::run notag-edn-logger))
         ;; ── JSON (round-trip-safe via sentinels) ──────────────
         ((_banner-json :())
          (:wat::std::service::Console/out con-tx
            "\n=== :Json (round-trip-safe sentinel-encoded) ===\n"))
         ((json-logger :wat::std::telemetry::ConsoleLogger)
          (:demo::make-logger con-tx :market.observer
            :wat::std::telemetry::Console::Format::Json))
         ((_run-json :()) (:demo::run json-logger))
         ;; ── NoTagJson (natural JSON for ingestion tooling) ────
         ((_banner-notag-json :())
          (:wat::std::service::Console/out con-tx
            "\n=== :NoTagJson (natural JSON for ELK/DataDog) ===\n"))
         ((notag-json-logger :wat::std::telemetry::ConsoleLogger)
          (:demo::make-logger con-tx :market.observer
            :wat::std::telemetry::Console::Format::NoTagJson))
         ((_run-notag-json :()) (:demo::run notag-json-logger))
         ;; ── Pretty (tagged, multi-line) ───────────────────────
         ((_banner-pretty :())
          (:wat::std::service::Console/out con-tx
            "\n=== :Pretty (tagged, multi-line) ===\n"))
         ((pretty-logger :wat::std::telemetry::ConsoleLogger)
          (:demo::make-logger con-tx :market.observer
            :wat::std::telemetry::Console::Format::Pretty)))
        (:demo::run pretty-logger))))
    (:wat::kernel::join con-driver)))
