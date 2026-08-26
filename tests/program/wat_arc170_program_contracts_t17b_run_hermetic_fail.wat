;; tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat — the FAILURE-path
;; hermetic run (arc 278 IPC de-prime). Sibling of t17_run_hermetic: SAME primed wire
;; (`spawn-program' (process)` + `recv'`), but the child's `assert-eq` FAILS, so the child
;; DIES before it can send anything.
;;
;; recv' surfaces that death honestly as `RecvOutcome::Lost cause`, where `cause` is a
;; `:wat::kernel::LociDiedError`. An assert-eq failure is an AssertionPayload panic, so `cause`
;; is the `LociDiedError::Panic` variant, whose `failure` field is `Some(Failure)` carrying the
;; structured assert-eq diagnostic (same shape proven in
;; tests/comms/probe_arc278_failure_carries_structured_error.wat). The death is SURFACED (this
;; defn returns the raw LociDiedError), NEVER swallowed.
;;
;; (The old form drove `:wat::test::run-hermetic` and inspected a `RunResult.failure` slot; the
;; failure is now the peer's own Lost cause, read straight off recv'.)
(:wat::core::defn :my::test::one-neq-two [] -> :wat::kernel::LociDiedError
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             ;; assert-eq: 1+0=1 vs expected=2 — this fails, child panics before sending.
             (:wat::test::assert-eq (:wat::i64::+ 1 0) 2))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Lost cause) cause)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed! "one-neq-two: expected the child to die on assert-eq, but it sent a value" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "one-neq-two: expected the child to die on assert-eq, but a stop was requested instead — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "one-neq-two: expected LociDiedError::Panic, got a clean close" :wat::core::None :wat::core::None)))))
