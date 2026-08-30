;; wat-tests/process/signal-user2-and-hangup-independent.wat
;;
;; P3 (DESIGN-STONE-process-signal-owner-to-child.md strike order) — replaces
;; `sigusr2_and_sighup_independent` (src/runtime.rs, deleted). The old test
;; set `KERNEL_SIGUSR2` directly and read both flags back in the harness's
;; own process — no signal was ever delivered. This sends REAL
;; `Signal::User2` to a real child and asserts BOTH observations — `(sigusr2?)`
;; true AND `(sighup?)` false — off ONE reply from a process that actually
;; received exactly one signal.
;;
;; `(sighup?)` false is trivially true of ANY never-signalled process; it only
;; demonstrates independence paired WITH `(sigusr2?)` true in the SAME reply
;; from the SAME child — which is why both ride home in one
;; `:wat::core::Vector` rather than two separate round trips (two round trips
;; would let a fresh, untouched process satisfy the Hangup half for free).
;;
;; Model: tests/process/signal_user1_delivers_child_observes_flag.wat (spawn/
;; signal/send/recv shape) + wat-tests/spawn/recv-budget-override.wat (the
;; deftest + spawn-peer + assert idiom).
(:wat::test::deftest :wat-tests::process::signal-user2-and-hangup-independent
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
                            (:wat::kernel::assertion-failed! "unexpected stop" :wat::core::None :wat::core::None)))]
                     (:wat::kernel::println
                       (:wat::core::Vector :- [:wat::core::bool] (:wat::kernel::sigusr2?) (:wat::kernel::sighup?)))))))]
      (:wat::core::match (:wat::kernel::signal child :wat::kernel::Signal::User2)
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
    (:wat::core::Vector :- [:wat::core::bool] true false)))
