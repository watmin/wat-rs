;; tests/diagnostics/probe_no_default_rust_panic_noise_on_stderr.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
;; (fork + OS-pipe scrape → :wat::kernel::RunResult{stdout,stderr,failure}) onto the
;; PRIMED peer wire — a direct `(:wat::test::spawn-peer (:wat::spawn::process)
;; (:wat::core::forms …))` child + `(:wat::kernel::recv' p)`. `RunResult` is GONE.
;;
;; ORIGINAL CONTRACT (partially obsolete — see the .rs header): this probe inspected the
;; child's OS-stderr lines (RunResult.stderr) to prove the silent panic hook suppressed
;; Rust's default "thread '…' panicked / RUST_BACKTRACE" noise, leaving ONLY a structured
;; line. The primed wire captures NO child OS-stderr — a crashed child's reason crosses
;; the wire STRUCTURALLY as the recv' Lost cause (a LociDiedError), never as raw stderr
;; text. So the ABSENCE-of-raw-noise contract is not expressible over the wire; what
;; SURVIVES (and subsumes it) is that the death arrives as a matchable structured
;; LociDiedError::Panic carrying the assertion message — never a raw noise blob.
;;
;; Body: assert-eq with mismatched values → AssertionPayload panic → recv' → Lost[Panic];
;; the assertion message rides Panic.message. We return it as a plain String.
(:wat::core::defn :probe::hook-test [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::test::assert-eq "expected-value" "actual-value"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          (_ "LOST-NON-PANIC")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
