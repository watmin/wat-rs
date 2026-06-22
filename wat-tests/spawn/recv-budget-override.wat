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
;; (258 > 64) → `FrameTooLarge` → lockstep teardown → `recv'` RAISES
;; "frame exceeded cap". `:should-panic` asserts that raise.
;;
;; RED at HEAD (disconfirms cleanly):
;;   - `(process/max-message-bytes …)` does not exist yet → the spawn errors with
;;     a DIFFERENT message than "frame exceeded cap" → :should-panic fails. AND
;;   - even were the builder present-but-ignored, 258 < 512 KiB default → the
;;     complete frame is DELIVERED (no raise) → :should-panic fails. The 256-byte
;;     sizing (between the 64 budget and the 512 KiB default) FORCES the override
;;     to be genuinely honored, not merely accepted.
;; GREEN after: the 64-byte budget is honored → the 258-byte message is rejected
;;   with the cap reason → :should-panic passes.
;;
;; Model: wat-tests/spawn/overcap-flood-no-deadlock.wat (deftest' + should-panic
;; + (:wat::spawn::process) + recv'). PRIMED ONLY.
(:wat::test::should-panic "frame exceeded cap")
(:wat::test::deftest' :wat-tests::recv-budget::tiny-budget-rejects-oversized-message
  ()
  (:wat::core::let
    [child (:wat::kernel::spawn-program' (:wat::spawn::process/max-message-bytes 64)
             (:wat::core::forms
               ;; double "x" 8× → 2^8 = 256-char String; println'd it is a
               ;; COMPLETE ('\n'-terminated) frame of ~258 bytes on the wire.
               (:wat::core::defn :my::rep [s <- :wat::core::String n <- :wat::core::i64] -> :wat::core::String
                 (:wat::core::if (:wat::core::= n 0) -> :wat::core::String
                     s
                     (:my::rep (:wat::core::String/concat s s) (:wat::core::i64::- n 1))))
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 (:wat::kernel::println (:my::rep "x" 8)))))
     ;; recv' must RAISE "frame exceeded cap" — the 64-byte budget rejects the
     ;; 258-byte complete message. :should-panic catches the raise; the global
     ;; per-test time-limit catches any deadlock regression.
     _ (:wat::kernel::recv' child)]
    nil))
