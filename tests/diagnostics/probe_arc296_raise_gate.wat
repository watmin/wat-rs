;; tests/diagnostics/probe_arc296_raise_gate.wat — co-located fixture for the raise gate probe.
;;
;; Arc 296 S3: :wat::kernel::raise! re-gated to require :wat::core::Error.
;;
;; IPC de-prime (arc 278): the sandboxed-raise leg was migrated off the non-prime
;; `:wat::test::run-thread` (spawn-thread + Thread/join-result → :wat::kernel::RunResult)
;; onto the PRIMED peer wire — a direct `(:wat::test::spawn-peer (:wat::spawn::process)
;; (:wat::core::forms …))` child + `(:wat::kernel::recv' p)`. `RunResult` is GONE.
;; NOTE: the retired harness here was run-THREAD, not run-hermetic; the primed replacement
;; is the :process tier (per the migration template + the sibling diagnostics). "The raise
;; is caught" now = the child crash surfaces as recv' → Lost[cause] with cause a
;; LociDiedError::Panic whose message is the raised Fault's human message ("boom").
;;
;; GREEN after: startup boots and main runs. Proves:
;; (a) :wat::core::Fault/of "boom" type-checks as :wat::core::Error (satisfies the surface).
;; (b) The raise in a spawned child is caught over the wire as Lost[Panic]; the Panic
;;     message carries the raised Fault's message "boom".
;; (c) Passing a Fault to [e <- :wat::core::Error] param type-checks.

(:wat::core::defn :probe::accept-error [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; (c) Fault/of satisfies :wat::core::Error structurally.
     msg  (:probe::accept-error (:wat::core::Fault/of "boom"))
     ;; (b) A child that raises is caught over the primed wire as Lost[Panic].
     p    (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::kernel::raise! (:wat::core::Fault/of "boom")))))
     ;; raise-msg: the Panic message if caught as Lost[Panic]; a sentinel otherwise.
     raise-msg (:wat::core::match (:wat::kernel::recv p)
                 ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
                 ((:wat::kernel::RecvOutcome::Lost cause)
                   (:wat::core::match cause
                     ((:wat::kernel::LociDiedError::Panic message _failure) message)
                     (_ "LOST-NON-PANIC")))
                 (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
                 (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      ;; Verify the error message round-trips through accept-error.
      (:wat::test::assert-eq msg "boom")
      ;; Verify the sandboxed raise was caught over the wire, carrying "boom".
      (:wat::test::assert-eq raise-msg "boom"))))
