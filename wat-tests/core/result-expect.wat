;; wat-tests/core/result-expect.wat — arc 108 unit tests for
;; `:wat::core::Result/expect`.
;;
;; Form: (:wat::core::Result/expect -> :T <res> <msg>). On
;; `(Ok v)` returns `v`; on `(Err _)` panics with the msg (the
;; carried Err value is discarded — the message names the
;; contract).


;; ─── Ok happy path — i64 ──────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-i64
  
  (:wat::core::let
    [res (:wat::core::Ok 99)
     v
      (:wat::core::Result/expect  
        res
        "should be Ok")]
    (:wat::test::assert-eq v 99)))


;; ─── Ok happy path — String ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::result-expect::ok-string
  
  (:wat::core::let
    [res (:wat::core::Ok "yes")
     v
      (:wat::core::Result/expect  
        res
        "should be Ok")]
    (:wat::test::assert-eq v "yes")))


;; ─── Err panics with the supplied message ────────────────────────────


(:wat::test::deftest :wat-tests::core::result-expect::err-panics-with-message
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; Result/expect on Err panics; the crash reaches the parent's recv'
           ;; as Lost (carrying the LociDiedError) BEFORE the completion send'.
           (:wat::core::do
             (:wat::core::let
               [res (:wat::core::Err "rundb crashed")
                _v
                 (:wat::core::Result/expect
                   res
                   "expected Ok value")]
               nil)
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the Err expect above already panicked
               ;; before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected panic on Err expect, got clean completion"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::test::assert-eq
          (:wat::kernel::LociDiedError/message cause)
          "expected Ok value"))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected panic on Err expect, got clean close"
          :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
