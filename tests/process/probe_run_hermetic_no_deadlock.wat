;; tests/process/probe_run_hermetic_no_deadlock.wat — co-located fixture for probe_run_hermetic_no_deadlock.rs
;;
;; Arc 278 IPC de-prime (MAP unit). A no-deadlock regression: historically drove
;; the non-prime `:wat::test::run-hermetic` + `run-hermetic-driver` drain-then-join
;; restructure. Migrated onto the PRIMED peer wire — `(:wat::test::spawn-peer
;; (:wat::spawn::process) (:wat::core::forms …))` child + `(:wat::kernel::recv' p)`.
;; The point is preserved: the primed wire ALSO does not deadlock — the parent's
;; `recv'` COMPLETES (rather than hanging) for both a clean child and a dying one.
;; Completing the recv' (and the test) IS the positive no-deadlock verification.
;;
;; Both probe defns coexist in the same world:
;;   :probe::test::clean-exit        (Probe 1 — clean child → recv' → Closed)
;;   :probe::test::intentional-panic (Probe 2 — dying child → recv' → Lost[cause])

;; Probe 1 — clean child returns nil, prints nothing, sends nothing → recv' → Closed.
;; (Mirrors the old failure=None clean-exit read; completing = no hang.)
(:wat::core::defn :probe::test::clean-exit [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil nil)))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost _cause) "UNEXPECTED-LOST")
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "closed") (:wat::kernel::RecvOutcome::TimedOut "UNEXPECTED-LOST"))))

;; Probe 2 — child calls assertion-failed! → the peer CRASHES before any send →
;; recv' → Lost[cause] (a LociDiedError carrying the diagnostic). Returns the
;; death message; completing = no hang even on the failure path. (Mirrors the old
;; failure=Some[non-empty message] read.)
(:wat::core::defn :probe::test::intentional-panic [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::assertion-failed!
               "intentional panic from probe_run_hermetic_no_deadlock"
               :wat::core::None
               :wat::core::None))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::LociDiedError/message cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
