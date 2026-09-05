;; wat-tests/process/signal-terminate-kills-the-child-and-the-read-sees-it.wat
;;
;; P4 (DESIGN-STONE-process-signal-owner-to-child.md strike order) — replaces
;; tests/process/shutdown_cascade_memory.rs and shutdown_cascade_pipefd.rs,
;; both deleted with this commit.
;;
;; THE TESTS THIS REPLACES RAISED SIGTERM AT THEMSELVES. In their own process,
;; to unblock their own thread, to reach a function they could have called
;; directly. The last link of the chain they were driving —
;; `wat::runtime::trigger_shutdown()` — is a plain public fn; the signal, the
;; handler, the wake pipe and the worker were all theatre in front of it. And
;; the value they asserted (`channel::RecvOutcome::Shutdown`) is internal: no
;; wat program can observe it, so nothing a user can write was covered.
;;
;; THIS IS THE TEST THAT WAS WANTED, and it is the ordinary one:
;;
;;   spawn a child in a process locus
;;   read from it            -> it speaks, so it is alive
;;   signal it               -> from the PARENT, the way a signal is actually sent
;;   read from it again      -> it is gone
;;
;; No self-signal, because a parent signalling its child is what a signal IS.
;; No thread anywhere, because nothing here is about threads.
;;
;; `RecvOutcome::Closed` is the honest assertion for the second read: the child
;; caught SIGTERM, its blocked `readln` returned `ReadlnOutcome::Stopped`,
;; `:user::main` returned, the process exited, and the parent's pipe read hit
;; EOF. The child is genuinely closed — nothing is being papered over. (A stop
;; observed on a peer that is STILL ALIVE is a different question and wants a
;; variant this enum does not have; it is not this test's subject.)
(:wat::test::deftest :wat-tests::process::signal-terminate-kills-the-child-and-the-read-sees-it
  (:wat::test::assert-eq
    (:wat::core::let
      [child (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil
                   (:wat::core::do
                     ;; Speak first, so the parent can prove we are alive.
                     (:wat::kernel::println "alive")
                     ;; Then park. Nothing will ever be sent; the only thing
                     ;; that ends this wait is the parent's SIGTERM, which
                     ;; surfaces as ReadlnOutcome::Stopped and lets main return
                     ;; normally so the process exits and the pipe closes.
                     (:wat::core::match (:wat::kernel::readln)
                       ((:wat::kernel::ReadlnOutcome::Datum d) nil)
                       (:wat::kernel::ReadlnOutcome::Eof nil)
                       (:wat::kernel::ReadlnOutcome::Stopped nil))))))]
      ;; 1 — the child speaks: it is alive and past its own startup.
      (:wat::core::match (:wat::kernel::recv child)
        ((:wat::kernel::RecvOutcome::Message m)
          ;; 2 — kill it, from the parent, like a signal is normally sent.
          (:wat::core::match (:wat::kernel::signal child :wat::kernel::Signal::Terminate)
            (:wat::kernel::SignalOutcome::Delivered
              ;; 3 — the child ANNOUNCES its stop. Arc 170 "stopping is a
              ;; protocol": on SIGTERM the child asks each held stdio service
              ;; to stop and emits one `#wat.kernel/StopAccepted {:services …}`
              ;; on stdout before tearing down. Seeing it here is the proof
              ;; that the signal arrived AND the protocol ran — strictly more
              ;; than "the pipe went quiet".
              (:wat::core::match (:wat::kernel::recv child)
                ((:wat::kernel::RecvOutcome::Message announced)
                  ;; 4 — read once more. Now it is gone.
                  (:wat::core::match (:wat::kernel::recv child)
                    ((:wat::kernel::RecvOutcome::Message extra)
                      (:wat::kernel::assertion-failed!
                        "the child kept talking after it announced its stop"
                        :wat::core::None :wat::core::None))
                    ((:wat::kernel::RecvOutcome::Lost cause)
                      (:wat::kernel::assertion-failed!
                        (:wat::kernel::LociDiedError/message cause)
                        :wat::core::None :wat::core::None))
                    ;; arc 278 #73 — the case this file's header once had no variant
                    ;; for: a STOP observed on a peer that is still ALIVE. That is
                    ;; NOT this test's subject (a SIGTERM-driven clean process exit,
                    ;; asserted as Closed below) — never conflate the two, so this
                    ;; arm reports the stop distinctly rather than folding into
                    ;; Closed's `true`.
                    (:wat::kernel::RecvOutcome::Stopped
                      (:wat::kernel::assertion-failed!
                        "recv: stopped — the substrate was asked to stop; the child was ALIVE (not the SIGTERM-close this test proves)"
                        :wat::core::None :wat::core::None))
                    (:wat::kernel::RecvOutcome::Closed true) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::kernel::assertion-failed!
                    (:wat::kernel::LociDiedError/message cause)
                    :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::kernel::assertion-failed!
                    "recv: stopped — the substrate was asked to stop; the child was ALIVE and the channel open"
                    :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed!
                    "the child died without announcing a stop — the signal did not run the protocol"
                    :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
            ((:wat::kernel::SignalOutcome::Failed cause)
              (:wat::kernel::assertion-failed!
                (:wat::kernel::Failure/message cause)
                :wat::core::None :wat::core::None))))
        ((:wat::kernel::RecvOutcome::Lost cause)
          (:wat::kernel::assertion-failed!
            (:wat::kernel::LociDiedError/message cause)
            :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Stopped
          (:wat::kernel::assertion-failed!
            "recv: stopped — the substrate was asked to stop; the child was ALIVE and the channel open"
            :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Closed
          (:wat::kernel::assertion-failed!
            "the child closed before we ever signalled it"
            :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))
    true))
