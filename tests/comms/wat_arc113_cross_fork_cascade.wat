;; tests/comms/wat_arc113_cross_fork_cascade.wat — co-located fixture for the cross-fork cascade probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Arc 278 IPC de-prime (MAP unit): the driver migrated off the retired non-prime
;; `:wat::test::run-hermetic` (fork + stderr-EDN scrape into a RunResult) onto the
;; PRIMED peer wire — a process peer (`spawn-program' (process)`) whose entry forms
;; define a `:user::main` that `assert-eq`s across the fork boundary. The child body
;; is UNCHANGED; only the driver flipped.
;;
;; An assert-eq failure is an AssertionPayload panic, so the child dies before it can
;; send. `recv'` surfaces that death honestly as `RecvOutcome::Lost cause`, where
;; `cause` is a `:wat::kernel::LociDiedError::Panic` whose `failure` field is
;; `Some(Failure)` carrying the structured assert-eq diagnostic — message, actual,
;; expected — reconstructed across the real fork boundary (same shape proven in
;; tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat and
;; tests/comms/probe_arc278_failure_carries_structured_error.wat). The death is read
;; STRUCTURALLY off the Failure record (Failure/message, Failure/actual,
;; Failure/expected) — no edn::read, no string re-parse — and NEVER swallowed.
;;
;; Returns [message actual-or-":None" expected-or-":None"] — the same triple the old
;; RunResult.failure path produced, now sourced from the peer's own Lost cause.

(:wat::core::defn :my::compute [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [p
      (:wat::test::spawn-peer (:wat::spawn::process)
        (:wat::core::forms
          (:wat::core::defn :user::main [] -> :wat::core::nil
            (:wat::test::assert-eq 1 2))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::core::Vector :- [:wat::core::String] "UNEXPECTED-MESSAGE"))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic _message failure)
            (:wat::core::match failure
              ((:wat::core::Some f)
               (:wat::core::Vector :- [:wat::core::String]
                 (:wat::kernel::Failure/message f)
                 (:wat::core::match (:wat::kernel::Failure/actual f)
                   ((:wat::core::Some a) a)
                   (:wat::core::None ":None"))
                 (:wat::core::match (:wat::kernel::Failure/expected f)
                   ((:wat::core::Some e) e)
                   (:wat::core::None ":None"))))
              (:wat::core::None
               (:wat::core::Vector :- [:wat::core::String] "NO-FAILURE-PAYLOAD"))))
          (_ (:wat::core::Vector :- [:wat::core::String] "LOST-NON-PANIC"))))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::core::Vector :- [:wat::core::String] "UNEXPECTED-STOPPED"))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::core::Vector :- [:wat::core::String] "UNEXPECTED-CLOSED")) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
