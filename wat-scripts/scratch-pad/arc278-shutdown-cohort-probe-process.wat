;; Disconfirming probe for BRIEF-the-shutdown-cohort-moves-to-children.md (Shape A, process tier).
;;
;; Parks the MAIN peer's recv on a PROCESS-tier peer (forked child) that never sends anything,
;; then prints what the outcome actually is when this process is SIGTERM'd. If it is not
;; `RecvOutcome::Lost[LociDiedError::Stopped]`, Shape A does not hold for the process tier and
;; the pipefd file needs Shape B instead.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             ;; The child parks forever on its OWN recv (parent never sends) — it must
             ;; never return, or its close would give the parent Closed instead of the
             ;; Stopped outcome under test.
             (:wat::core::match (:wat::kernel::recv (:wat::program::self-peer :wat::core::i64 :wat::core::i64))
               ((:wat::kernel::RecvOutcome::Message _m) nil)
               ((:wat::kernel::RecvOutcome::Lost _c) nil)
               (:wat::kernel::RecvOutcome::Closed nil)))))]
    (:wat::core::do
      (:wat::kernel::println "READY")
      (:wat::core::match (:wat::kernel::recv w)
        ((:wat::kernel::RecvOutcome::Message _m) (:wat::kernel::println "OUTCOME:Message"))
        ((:wat::kernel::RecvOutcome::Lost cause)
          (:wat::core::match cause
            (:wat::kernel::LociDiedError::Stopped (:wat::kernel::println "OUTCOME:Lost:Stopped"))
            (_ (:wat::kernel::println (:wat::core::string::concat "OUTCOME:Lost:Other:" (:wat::kernel::LociDiedError/message cause))))))
        (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "OUTCOME:Closed"))))))
