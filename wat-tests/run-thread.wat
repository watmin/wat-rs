;; wat-tests/run-thread.wat — Layer 1 verification for :wat::test::run-thread.
;;
;; Arc 170 slice 4a-α (task #308). Cheap-thread counterpart to the
;; existing :wat::test::run-hermetic. The macro spawns a thread via
;; :wat::kernel::spawn-thread, joins via :wat::kernel::Thread/join-result,
;; and surfaces panics as a structured Failure in RunResult.failure.
;;
;; Two paths exercised:
;;
;;   Ok-path  — body runs a passing assertion; outer asserts
;;              RunResult.failure is :None.
;;
;;   Err-path — body runs a FAILING assertion; outer asserts
;;              RunResult.failure is :Some(_). This is the load-bearing
;;              proof that ThreadDiedError -> Failure conversion works
;;              through Thread/join-result's chain branch. Without this,
;;              the next stone (4a-β sweep) can't trust panic
;;              propagation through the new macro.
;;
;; Each deftest body executes inside its own deftest sandbox (currently
;; run-hermetic per test.wat:294-303). The INNER program is what
;; exercises run-thread.

;; ─── Ok-path: passing assertion inside run-thread ─────────────────────


(:wat::test::deftest :wat-tests::test::run-thread-ok-path
  
  ;; arc 278 IPC de-prime: run-thread → primed peer wire (spawn-program' :thread + recv').
  ;; The PASSING assertion lets the self-peer reach its send' → recv' Message → clean run
  ;; (the old RunResult/failure :None). Lost/Closed would mean the pass was misclassified.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::do
             (:wat::test::assert-eq 4 (:wat::i64::+ 2 2))
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless (never a `_`-swallow).
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))
     fail (:wat::core::match (:wat::kernel::recv p)
            ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause)))
            ;; arc 278 #73 — a stop is neither a clean pass nor the assertion failure
            ;; this file exists to distinguish; assert it distinctly rather than fold
            ;; it into either :None (Closed's meaning: thread finished quietly) or
            ;; :Some (Lost's meaning: the thread died).
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed!
                "run-thread: stopped — the substrate was asked to stop; the thread was ALIVE and the channel open"
                :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed :wat::core::None) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::core::match fail

      (:wat::core::None nil)
      ((:wat::core::Some _f)
       (:wat::kernel::assertion-failed!
         "Ok-path: expected :None but got :Some — passing assertion was misclassified as failure"
         :wat::core::None :wat::core::None)))))

;; ─── Err-path: failing assertion inside run-thread ────────────────────


(:wat::test::deftest :wat-tests::test::run-thread-err-path
  
  ;; arc 278 IPC de-prime: run-thread → primed peer wire (spawn-program' :thread + recv').
  ;; The FAILING assertion crashes the self-peer BEFORE its send' → recv' Lost[cause];
  ;; LociDiedError/to-failure rebuilds the (Option :- [Failure]) the old RunResult/failure gave
  ;; (:Some), so the downstream match on `fail` is unchanged.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::do
             (:wat::test::assert-eq 99 (:wat::i64::+ 2 2))
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless (never a `_`-swallow).
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))
     fail (:wat::core::match (:wat::kernel::recv p)
            ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause)))
            ;; arc 278 #73 — a stop is neither a clean pass nor the assertion failure
            ;; this file exists to distinguish; assert it distinctly rather than fold
            ;; it into either :None (Closed's meaning: thread finished quietly) or
            ;; :Some (Lost's meaning: the thread died).
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed!
                "run-thread: stopped — the substrate was asked to stop; the thread was ALIVE and the channel open"
                :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed :wat::core::None) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::core::match fail

      ((:wat::core::Some _f) nil)
      (:wat::core::None
       (:wat::kernel::assertion-failed!
         "Err-path: expected :Some failure but got :None — chain handling broken"
         :wat::core::None :wat::core::None)))))
