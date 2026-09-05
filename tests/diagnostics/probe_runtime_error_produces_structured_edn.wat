;; tests/diagnostics/probe_runtime_error_produces_structured_edn.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
;; (fork + OS-pipe scrape → :wat::kernel::RunResult) onto the PRIMED peer wire — a
;; direct `(:wat::test::spawn-peer (:wat::spawn::process) (:wat::core::forms …))`
;; child + `(:wat::kernel::recv' p)`. `RunResult` is GONE from this file.
;;
;; Body: integer division by zero → RuntimeError::DivisionByZero. Passes type-check;
;; fails at CHILD RUNTIME. apply_function returns Err(RuntimeError) — the Ok(Err(runtime))
;; arm of the forked child — so the death surfaces over the wire as recv' → Lost[cause]
;; with cause = LociDiedError::RuntimeError (NOT Panic; a runtime error is not a Rust
;; panic — same mapping wat_run_sandboxed's missing-main case grounds). The runtime
;; error text rides RuntimeError.message; we return it as a plain String for the driver.
(:wat::core::defn :probe::runtime-err [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             ;; Division by zero → RuntimeError::DivisionByZero.
             (:wat::core::let [_ (:wat::i64::/ 1 0)] nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ;; TRUE variant: integer div-by-zero → LociDiedError::RuntimeError (a runtime
          ;; error is NOT a Rust panic); the error text rides RuntimeError.message. Return it.
          ((:wat::kernel::LociDiedError::RuntimeError message) message)
          ;; LociDiedError is the no-hidden-failures enum — every OTHER death is named
          ;; EXPLICITLY (no `_` lump; verbosity is the shield). A distinct WRONG:<variant>
          ;; sentinel makes a RED name exactly which non-RuntimeError death surfaced.
          ((:wat::kernel::LociDiedError::Panic _pm _pf) "WRONG:Panic")
          (:wat::kernel::LociDiedError::Disconnected "WRONG:Disconnected")
          (:wat::kernel::LociDiedError::Stopped "WRONG:Stopped")
          (:wat::kernel::LociDiedError::Severed "WRONG:Severed")
          ((:wat::kernel::LociDiedError::StartupError _m) "WRONG:StartupError")
          ((:wat::kernel::LociDiedError::EntryFormFailure _m) "WRONG:EntryFormFailure")
          ((:wat::kernel::LociDiedError::MainSignature _m) "WRONG:MainSignature")
          ((:wat::kernel::LociDiedError::BadReturn _m) "WRONG:BadReturn")))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
