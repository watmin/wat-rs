;; examples/console-demo/wat/main.wat — ambient-stdio walk-through.
;;
;; Arc 170 slice 1f-η — Console driver retired. Previously this
;; example wired a Console spawn-driver thread + a ConsoleLogger
;; handle plumbed through producer scope. With the runtime
;; orchestrator (slice 1f-γ) + ambient stdio trio (slices
;; 1f-β-i/ii/iii) + the ambient `:wat::kernel::println` /
;; `eprintln` ops (slice 1f-α), producers print directly. No
;; spawn, no pool, no handle.
;;
;; Contract — ambient ops EDN-encode their argument and write one
;; line per call. Nothing free-form crosses the boundary; every
;; emission is `:wat::edn::read`-parseable. Format selection
;; (the old Console-handle-mediated EDN/Json/Pretty/NoTagEdn/
;; NoTagJson showcase) no longer applies — the ambient surface is
;; deliberately EDN-only. Apps wanting alternate formats compose
;; their own producer-side helper that bypasses the ambient ops
;; (writing through a custom service driver in user code) — but
;; the default path is the EDN one this demo walks.
;;
;; Run:
;;   cargo run -p console-demo                 # shows stdout
;;   cargo run -p console-demo 2>&1 >/dev/null # shows stderr
;;   cargo run -p console-demo 2>err.log       # split streams


;; ─── Domain enum — what the trader emits as structured events ──

(:wat::core::enum :demo::Event
  (Buy
    (price :wat::core::f64)
    (qty :wat::core::i64))
  (Sell
    (price :wat::core::f64)
    (qty :wat::core::i64)
    (reason :wat::core::String))
  (CircuitBreak
    (reason :wat::core::String)))


;; ─── Wiring — five events, ambient println / eprintln routing.
;;
;; :debug + :info shaped emissions go through stdout
;; (`:wat::kernel::println`); :warn + :error go through stderr
;; (`:wat::kernel::eprintln`). The ambient ops EDN-encode each
;; value before writing, so the produced lines round-trip via
;; `:wat::edn::read` cleanly. `:user::main` returns
;; `:wat::core::nil` (arc 170 slice 1e canonical entry shape).

(:wat::core::define
  (:user::main -> :wat::core::nil)
  (:wat::core::let
    [;; Routine flow → stdout
     _a (:wat::kernel::println (:demo::Event::Buy 100.5 7))
     _b (:wat::kernel::println (:demo::Event::Sell 102.25 3 "stop-loss"))
     ;; Diagnostic detail → stdout
     _c (:wat::kernel::println (:demo::Event::Buy 99.0 12))
     ;; Concerning event → stderr
     _d (:wat::kernel::eprintln (:demo::Event::CircuitBreak "spike-volume"))
     ;; Failure → stderr
     _e (:wat::kernel::eprintln (:demo::Event::CircuitBreak "exchange-disconnected"))]
    :wat::core::nil))
