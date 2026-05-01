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
    (logger :wat::telemetry::ConsoleLogger)
    -> :wat::core::unit)
  (:wat::core::let*
    (;; Routine flow → stdout
     ((_a :wat::core::unit)
      (:wat::telemetry::ConsoleLogger/info logger
        (:demo::Event::Buy 100.5 7)))
     ((_b :wat::core::unit)
      (:wat::telemetry::ConsoleLogger/info logger
        (:demo::Event::Sell 102.25 3 "stop-loss")))
     ;; Diagnostic detail → stdout
     ((_c :wat::core::unit)
      (:wat::telemetry::ConsoleLogger/debug logger
        (:demo::Event::Buy 99.0 12)))
     ;; Concerning event → stderr
     ((_d :wat::core::unit)
      (:wat::telemetry::ConsoleLogger/warn logger
        (:demo::Event::CircuitBreak "spike-volume")))
     ;; Failure → stderr
     ((_e :wat::core::unit)
      (:wat::telemetry::ConsoleLogger/error logger
        (:demo::Event::CircuitBreak "exchange-disconnected"))))
    ()))


;; ─── Helper — build a logger with a chosen format ───────────────
;;
;; The logger captures a Console::Handle = (Tx, AckRx) — every
;; Console/out and Console/err call goes through the handle and
;; blocks until the driver acks the write (arc 089 slice 5,
;; mini-TCP via paired channels).

(:wat::core::define
  (:demo::make-logger
    (handle :wat::std::service::Console::Handle)
    (caller :wat::core::keyword)
    (format :wat::telemetry::Console::Format)
    -> :wat::telemetry::ConsoleLogger)
  (:wat::telemetry::ConsoleLogger/new
    handle caller
    (:wat::core::lambda ((_u :wat::core::unit) -> :wat::time::Instant)
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
    -> :wat::core::unit)
  (:wat::core::let*
    (((con-spawn :wat::std::service::Console::Spawn)
      (:wat::std::service::Console/spawn stdout stderr 1))
     ((con-pool :wat::kernel::HandlePool<wat::std::service::Console::Handle>)
      (:wat::core::first con-spawn))
     ((con-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::second con-spawn))
     ((_inner :wat::core::unit)
      (:wat::core::let*
        (((handle :wat::std::service::Console::Handle)
          (:wat::kernel::HandlePool::pop con-pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish con-pool))
         ;; ── EDN format (tagged, round-trip-safe) ──────────────
         ((_banner-edn :wat::core::unit)
          (:wat::std::service::Console/out handle
            "\n=== :Edn (tagged, round-trip-safe) ===\n"))
         ((edn-logger :wat::telemetry::ConsoleLogger)
          (:demo::make-logger handle :market.observer
            :wat::telemetry::Console::Format::Edn))
         ((_run-edn :wat::core::unit) (:demo::run edn-logger))
         ;; ── NoTagEdn (lossy, human-friendly) ──────────────────
         ((_banner-notag-edn :wat::core::unit)
          (:wat::std::service::Console/out handle
            "\n=== :NoTagEdn (lossy, human-friendly) ===\n"))
         ((notag-edn-logger :wat::telemetry::ConsoleLogger)
          (:demo::make-logger handle :market.observer
            :wat::telemetry::Console::Format::NoTagEdn))
         ((_run-notag-edn :wat::core::unit) (:demo::run notag-edn-logger))
         ;; ── JSON (round-trip-safe via sentinels) ──────────────
         ((_banner-json :wat::core::unit)
          (:wat::std::service::Console/out handle
            "\n=== :Json (round-trip-safe sentinel-encoded) ===\n"))
         ((json-logger :wat::telemetry::ConsoleLogger)
          (:demo::make-logger handle :market.observer
            :wat::telemetry::Console::Format::Json))
         ((_run-json :wat::core::unit) (:demo::run json-logger))
         ;; ── NoTagJson (natural JSON for ingestion tooling) ────
         ((_banner-notag-json :wat::core::unit)
          (:wat::std::service::Console/out handle
            "\n=== :NoTagJson (natural JSON for ELK/DataDog) ===\n"))
         ((notag-json-logger :wat::telemetry::ConsoleLogger)
          (:demo::make-logger handle :market.observer
            :wat::telemetry::Console::Format::NoTagJson))
         ((_run-notag-json :wat::core::unit) (:demo::run notag-json-logger))
         ;; ── Pretty (tagged, multi-line) ───────────────────────
         ((_banner-pretty :wat::core::unit)
          (:wat::std::service::Console/out handle
            "\n=== :Pretty (tagged, multi-line) ===\n"))
         ((pretty-logger :wat::telemetry::ConsoleLogger)
          (:demo::make-logger handle :market.observer
            :wat::telemetry::Console::Format::Pretty)))
        (:demo::run pretty-logger)))
     ((_join :Result<wat::core::unit,Vec<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result con-driver)))
    ()))
