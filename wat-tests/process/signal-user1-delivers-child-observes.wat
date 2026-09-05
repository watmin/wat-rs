;; wat-tests/process/signal-user1-delivers-child-observes.wat
;;
;; P3 (DESIGN-STONE-process-signal-owner-to-child.md strike order) — replaces
;; `sigusr1_query_reflects_flag_state` (src/runtime.rs, deleted). The old test
;; called a setter then a getter in the HARNESS's own process — no signal was
;; ever delivered, no handler ever ran. This deftest spawns a REAL child,
;; signals it with `:wat::kernel::Signal::User1` through the P2 verb, and
;; asserts on what the CHILD reports observing — the only honest evidence
;; that delivery happened. Asserting the parent's `SignalOutcome::Delivered`
;; alone would prove only that the kernel accepted the signal, not that a
;; handler ran; the assertion below is on the child's own reply.
;;
;; Model: `signal_user1_delivers_child_observes_flag.wat` (P2's evidence
;; fixture — spawn/signal/send/recv shape, deleted with this commit) +
;; wat-tests/spawn/recv-budget-override.wat (the deftest + spawn-peer + assert
;; idiom in its proper home).
;;
;; The child blocks in `readln` until the parent's `send` unblocks it —
;; proving it is alive, and ordering the child's flag read strictly after the
;; kernel already reported the signal ::Delivered. It replies with the RAW
;; bool `(:wat::kernel::sigusr1?)`; `recv` decodes the child's EDN wire back
;; into an actual `:wat::core::bool` Value, so `assert-eq` below compares the
;; child's own observation structurally, not a rendered string.
(:wat::test::deftest :wat-tests::process::signal-user1-delivers-child-observes
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
                     (:wat::kernel::println (:wat::kernel::sigusr1?))))))]
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
    true))
