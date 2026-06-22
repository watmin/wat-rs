;; wat-tests/overcap-flood-no-deadlock.wat — THE wat-direct proof (round two).
;;
;; Proves at the wat surface that an over-cap flood is REJECTED, never deadlocks.
;; A process child `print-raw'`s a ~1 MiB UN-TERMINATED frame (no newline); the
;; parent `recv'`s it. The fix (`RecvError::FrameTooLarge` → immediate lockstep
;; teardown, NO blocking `err.recv()`) makes `recv'` RAISE "process channel
;; disconnected" promptly. Before the fix this DEADLOCKED (parent waits on
;; err.recv() ↔ child blocked in write_all) — the global per-test time-limit
;; would fire and the test would fail as a timeout.
;;
;; `:should-panic` asserts the rejection raise; the global time-limit guards the
;; no-deadlock invariant (a regression to deadlock = a timeout failure).
;;
;; The Rust bench (tests/probe_overcap_no_deadlock.rs) is the lab bench; THIS is
;; the proof — the supervisor pattern in wat directly.

;; The SPECIFIC cap reason propagates all the way out: the inner process `recv'`
;; raises "frame exceeded cap …", the thread crash channel carries it through the
;; deftest' harness (arc 259 crash-reason parity — the thread tier now sends a
;; body RuntimeError on its crash channel, like the process tier's fd 2), and the
;; outer `recv'` re-raises it. So this asserts the EXACT cause, not a generic
;; disconnect. What THIS proves at the wat surface: the flood is REJECTED with the
;; cap reason AND does not deadlock (it completes far inside the global per-test
;; time-limit; a regression to deadlock would fail as a timeout).
(:wat::test::should-panic "frame exceeded cap")
(:wat::test::deftest' :wat-tests::overcap::flood-is-rejected-not-deadlocked
  ()
  (:wat::core::let
    [child (:wat::kernel::spawn-program' (:wat::spawn::process)
             (:wat::core::forms
               ;; double "x" 20× → ~1 MiB, then print-raw' it with NO newline → un-terminated frame.
               (:wat::core::defn :my::flood [s <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
                 (:wat::core::if (:wat::core::= n 0) -> :wat::core::nil
                     (:wat::kernel::print-raw' s)
                     (:my::flood (:wat::core::String/concat s s) (:wat::core::i64::- n 1))))
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 (:my::flood "x" 20))))
     ;; recv' must RAISE "process channel disconnected" (FrameTooLarge → teardown),
     ;; promptly — not hang. :should-panic catches the raise; the global time-limit
     ;; catches a deadlock regression.
     _ (:wat::kernel::recv' child)]
    nil))
