;; tests/diagnostics/probe_runtime_err_stderr_visibility.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
;; (fork + OS-pipe scrape → :wat::kernel::RunResult{stdout,stderr,failure}) onto the
;; PRIMED peer wire — a direct `(:wat::test::spawn-peer (:wat::spawn::process)
;; (:wat::core::forms …))` child + `(:wat::kernel::recv' p)`. `RunResult` is GONE.
;;
;; ORIGINAL PURPOSE (now obsolete — see the .rs header): this probe surfaced
;; RunResult.stderr to expose the retired harness's lossiness (it DROPPED the drained
;; stderr-lines Vec, reporting only "forked program exited N"). The primed wire has NO
;; OS-stderr side-channel at all: a crashed child's reason crosses the wire STRUCTURALLY
;; as the recv' Lost cause (a LociDiedError), so the "drop the stderr Vec" lossiness the
;; probe hunted cannot exist. The surviving contract is that the assertion death arrives
;; as a STRUCTURED LociDiedError::Panic carrying its message — not raw dropped text.
;;
;; Body: assert-eq with mismatched values → AssertionPayload panic → recv' → Lost[Panic];
;; the assertion message rides Panic.message. We return it as a plain String.
(:wat::core::defn :probe::structured [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::test::assert-eq "intentional" "different"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          (_ "LOST-NON-PANIC")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
