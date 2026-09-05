;; wat-tests/process/signal-reset-sigusr1-is-a-transition.wat
;;
;; P3 (DESIGN-STONE-process-signal-owner-to-child.md strike order) — replaces
;; `reset_sigusr1_flips_flag_false` (src/runtime.rs, deleted). The old test
;; set `KERNEL_SIGUSR1` directly, reset it, and read the flag back in the
;; harness's own process — no signal was ever delivered. This signals a REAL
;; child, and the child reports BOTH observations — before the reset and
;; after — in ONE reply, so what is asserted is the true->false FLIP itself,
;; not merely the false endpoint (which a process that was never signalled
;; at all would also report).
;;
;; Model: the P2 evidence fixture `signal_user1_delivers_child_observes_flag.wat`
;; (spawn/signal/send/recv shape, deleted with this commit) +
;; wat-tests/spawn/recv-budget-override.wat (the deftest + spawn-peer + assert idiom).
(:wat::test::deftest :wat-tests::process::signal-reset-sigusr1-is-a-transition
  (:wat::test::assert-eq
    (:wat::core::let
      [child (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil
                   (:wat::core::let
                     [n (:wat::core::match (:wat::kernel::readln)
                          ((:wat::kernel::ReadlnOutcome::Datum d) d)
                          (:wat::kernel::ReadlnOutcome::Eof
                            (:wat::kernel::assertion-failed! "unexpected eof" :wat::core::None :wat::core::None))
                          (:wat::kernel::ReadlnOutcome::Stopped
                            (:wat::kernel::assertion-failed! "unexpected stop" :wat::core::None :wat::core::None)))
                      before      (:wat::kernel::sigusr1?)
                      reset-done  (:wat::kernel::reset-sigusr1!)
                      after       (:wat::kernel::sigusr1?)]
                     (:wat::kernel::println (:wat::core::Vector :wat::core::bool before after))))))]
      (:wat::core::match (:wat::kernel::signal child :wat::kernel::Signal::User1)
        (:wat::kernel::SignalOutcome::Delivered
          (:wat::core::match (:wat::kernel::send child 0)
            (:wat::kernel::SendOutcome::Sent
              (:wat::core::match (:wat::kernel::recv child)
                ((:wat::kernel::RecvOutcome::Message m) m)
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::kernel::assertion-failed! "recv: stopped — the substrate was asked to stop; the child was ALIVE and the channel open" :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed! "recv: child closed unexpectedly" :wat::core::None :wat::core::None))))
            (:wat::kernel::SendOutcome::Closed
              (:wat::kernel::assertion-failed! "send: child closed unexpectedly" :wat::core::None :wat::core::None))
            (:wat::kernel::SendOutcome::Stopped
              (:wat::kernel::assertion-failed! "send: stopped — the substrate was asked to stop; the child was ALIVE and the channel open" :wat::core::None :wat::core::None))
            ((:wat::kernel::SendOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))))
        ((:wat::kernel::SignalOutcome::Failed cause)
          (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))))
    (:wat::core::Vector :wat::core::bool true false)))
