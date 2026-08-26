;; wat-tests/spawn/recv-budget-override.wat — THE recv-tunable frame-budget proof.
;;
;; Proves at the wat surface that a per-Receiver message budget set at spawn is
;; HONORED: a process peer spawned with a TINY `:max-message-bytes` rejects a
;; COMPLETE (terminated) message that the default 512 KiB budget would deliver.
;;
;; The child `println`s a 256-char String — a COMPLETE, '\n'-terminated frame
;; (~258 bytes on the wire). The parent spawned the child with a 64-byte budget.
;; Under the frame-budget contract (semantics B — max MESSAGE size, not merely
;; un-terminated accumulation), `next_complete_frame` rejects this complete frame
;; (258 > 64) → `FrameTooLarge` → lockstep teardown.
;;
;; Arc 278 recv'-wall: `recv'` returns a matchable RecvOutcome VALUE, never a raise
;; (a raise unwinds past the reader — the mask this arc kills). The over-budget
;; rejection surfaces as `RecvOutcome::Lost` carrying the frame-cap reason
;; ("frame exceeded cap (message larger than the receiver's max-message-bytes
;; budget)"). We MATCH the outcome and ASSERT it is ::Lost with the cap reason —
;; the deftest' verdict: a clean return = PASS, a failing assertion = FAIL.
;;
;; The 256-byte sizing (between the 64 budget and the 512 KiB default) FORCES the
;; override to be genuinely honored: were the budget ignored, 258 < 512 KiB default
;; → the complete frame is DELIVERED as ::Message → the assertion fires FAIL.
;;
;; Model: wat-tests/spawn/overcap-flood-no-deadlock.wat + the recv'-wall value
;; contract (probe_arc278_recv_over_budget_reason). PRIMED ONLY.
(:wat::test::deftest :wat-tests::recv-budget::tiny-budget-rejects-oversized-message
  
  (:wat::core::let
    [child (:wat::test::spawn-peer (:wat::spawn::process/max-message-bytes 64)
             (:wat::core::forms
               ;; double "x" 8× → 2^8 = 256-char String; println'd it is a
               ;; COMPLETE ('\n'-terminated) frame of ~258 bytes on the wire.
               (:wat::core::defn :my::rep [s <- :wat::core::String n <- :wat::core::i64] -> :wat::core::String
                 (:wat::core::if (:wat::core::= n 0)
                     s
                     (:my::rep (:wat::core::String/concat s s) (:wat::i64::- n 1))))
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 (:wat::kernel::println (:my::rep "x" 8)))))]
    ;; The 64-byte budget must reject the 258-byte complete message as ::Lost with
    ;; the frame-cap reason. ::Message = the budget was ignored (delivered); ::Closed
    ;; = a bare EOF with no reason — both are the failure. The global per-test
    ;; time-limit catches any deadlock regression.
    (:wat::core::match (:wat::kernel::recv child)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "tiny budget ignored: the oversized frame was DELIVERED, not rejected"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::test::assert-contains (:wat::kernel::LociDiedError/message cause) "frame exceeded cap"))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "expected the over-budget frame to surface as ::Lost with the cap reason, got a ::Stopped — the child was ALIVE"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected the over-budget frame to surface as ::Lost with the cap reason, got a bare ::Closed"
          :wat::core::None :wat::core::None)))))
