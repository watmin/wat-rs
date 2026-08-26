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
;;   cargo run -p console-demo                 # five EDN lines on stdout, stderr empty


;; ─── Domain enum — what the trader emits as structured events ──

(:wat::core::defenum :demo::Event :wat::enum::Pure
  :Buy          [price <- :wat::core::f64  qty <- :wat::core::i64]
  :Sell         [price <- :wat::core::f64  qty <- :wat::core::i64  reason <- :wat::core::String]
  :CircuitBreak [reason <- :wat::core::String])


;; ─── Wiring — five events, every one through ambient `println`.
;;
;; ⚠ THERE IS NO BENIGN STDERR WRITE. `:wat::kernel::eprintln` is
;; wat's PANIC channel (`wat/kernel/diagnostics.wat:52`; registered
;; in `src/check.rs` as a TERMINATING form): it emits to stderr and
;; then TERMINATES the program non-zero. This demo used to route
;; ":warn / :error" events through it as though it were a second
;; ordinary print. It is not, and that version died on its first
;; "concerning" event without ever reaching the last one.
;;
;; What the substrate actually offers is the IPC triangle:
;;   stdout     — complex RETURN values (this demo's five events)
;;   stderr     — complex ERROR values (panic cascades; terminating)
;;   exit code  — a SIGNAL telling the parent which channel to read
;; So a program that has something to SAY says it on stdout. Only a
;; program that is DYING writes to stderr, and it dies as it writes.
;;
;; The ambient ops EDN-encode each value and write one line per
;; call, so every emission round-trips through `:wat::edn::read`.
;; `:user::main` returns bare `nil` (arc 170 slice 1e entry shape).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
      [_a (:wat::kernel::println (:demo::Event::Buy 100.5 7))
       _b (:wat::kernel::println (:demo::Event::Sell 102.25 3 "stop-loss"))
       _c (:wat::kernel::println (:demo::Event::Buy 99.0 12))
       _d (:wat::kernel::println (:demo::Event::CircuitBreak "spike-volume"))
       _e (:wat::kernel::println (:demo::Event::CircuitBreak "exchange-disconnected"))]
      nil))
