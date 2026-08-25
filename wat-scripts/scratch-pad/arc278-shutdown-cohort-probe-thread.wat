;; Disconfirming probe for BRIEF-the-shutdown-cohort-moves-to-children.md (Shape A, thread tier).
;;
;; Parks the MAIN peer's recv on a THREAD-tier peer that never sends anything, then prints
;; what the outcome actually is when this process is SIGTERM'd. If it is not
;; `RecvOutcome::Lost[LociDiedError::Stopped]`, Shape A does not hold for the thread tier and
;; the memory file needs Shape B instead.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [w (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; The child parks forever on its OWN recv (parent never sends) — it must
           ;; never return, or its send-half closing would give the parent Closed
           ;; instead of the Stopped outcome under test.
           (:wat::core::match (:wat::kernel::recv self)
             ((:wat::kernel::RecvOutcome::Message _m) nil)
             ((:wat::kernel::RecvOutcome::Lost _c) nil)
             ;; same discard as its siblings — this child's body never inspects the
             ;; outcome of its own parked recv either way.
             (:wat::kernel::RecvOutcome::Stopped nil)
             (:wat::kernel::RecvOutcome::Closed nil))))]
    (:wat::core::do
      (:wat::kernel::println "READY")
      (:wat::core::match (:wat::kernel::recv w)
        ((:wat::kernel::RecvOutcome::Message _m) (:wat::kernel::println "OUTCOME:Message"))
        ((:wat::kernel::RecvOutcome::Lost cause)
          (:wat::core::match cause
            (:wat::kernel::LociDiedError::Stopped (:wat::kernel::println "OUTCOME:Lost:Stopped"))
            (_ (:wat::kernel::println (:wat::string::concat "OUTCOME:Lost:Other:" (:wat::kernel::LociDiedError/message cause))))))
        ;; arc 278 #73 — this is the exact top-level arm the shutdown-cohort question
        ;; was probing for: post-migration, `recv_outcome_shutdown()` builds this
        ;; variant directly rather than `Lost[LociDiedError::Stopped]`, so a live run
        ;; should now print HERE, not through the nested Lost:Stopped dig above. Left
        ;; that dig in place (not collapsed) so this diagnostic can still tell the two
        ;; shapes apart if the mechanism ever regresses — reported, not decided.
        (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "OUTCOME:Stopped"))
        (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "OUTCOME:Closed"))))))
