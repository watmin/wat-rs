;; tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat — the FAILURE-path
;; bidirectional prime (arc 278 IPC de-prime). Sibling of t18_echo_doubled: SAME primed wire
;; (`spawn-program' (process)` + `send'` + `:wat::kernel::recv-all'`), but the child's `assert-eq`
;; FAILS mid-exchange, so the child DIES before it can `println`.
;;
;; recv-all' drains the peer honestly and returns a `Result<Vector<i64>, LociDiedError>`:
;;   Ok[outputs] -> the peer closed cleanly; the collected outputs.
;;   Err[cause]  -> the peer DIED; `cause` is a `:wat::kernel::LociDiedError` — here a
;;                  `LociDiedError::Panic` carrying the assertion failure (message + the
;;                  structured `Some Failure`), SURFACED in the Err, NEVER swallowed.
;;
;; This defn returns the RAW Result so the test measures the `Err[LociDiedError::Panic]` — the
;; child's death is the observable, matching the "recv_assert_fail" intent. (The old form drove
;; the non-prime `run-hermetic-with-io` and inspected a `RunResultIO.failure`; the death is now
;; the peer's own Lost cause, read straight off recv-all'.)

;; Spawn a process peer whose :user::main readln's an i64 then asserts it equals 3; send' 2 to
;; feed the child's readln → assert-eq 2 3 fails → child panics before its println → the peer
;; dies → recv-all' returns (Err (LociDiedError::Panic …)).
(:wat::core::defn :my::test::recv-assert-fail []
  -> :wat::core::Result<wat::core::Vector<wat::core::i64>,wat::kernel::LociDiedError>
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [n (:wat::kernel::readln )
                ;; assert-eq: n=2 vs expected=3 — this fails, child panics
                _ (:wat::test::assert-eq n 3)
                ;; println never reached (child already dead):
                _2 (:wat::kernel::println n)]
               nil))))
     _ (:wat::core::match (:wat::kernel::send' p 2)
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::kernel::recv-all' p)))
