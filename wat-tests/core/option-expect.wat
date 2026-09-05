;; wat-tests/core/option-expect.wat — arc 108 unit tests for
;; `:wat::core::Option/expect`.
;;
;; Form: (:wat::core::Option/expect -> :T <opt> <msg>) — type
;; declared at HEAD position before any value producer (parallels
;; `match`'s `-> :T` placement, but the VALUE-producing role of the
;; opt-expr puts the type ahead of it). On `(Some v)` returns `v`;
;; on `:None` panics with the msg.
;;
;; Pass cases: deftests that exercise the Some-arm.
;; Fail cases: run the panic path inside `:wat::test::run-ast` so
;; the surrounding catch_unwind surfaces the AssertionPayload as a
;; `Failure` on the inner RunResult; the outer deftest matches on it.


;; ─── Some happy path — i64 ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-i64
  
  (:wat::core::let
    [opt (:wat::core::Some 42)
     v
      (:wat::core::Option/expect  
        opt
        "should be Some")]
    (:wat::test::assert-eq v 42)))


;; ─── Some happy path — String ─────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-string
  
  (:wat::core::let
    [opt (:wat::core::Some "hello")
     v
      (:wat::core::Option/expect  
        opt
        "should be Some")]
    (:wat::test::assert-eq v "hello")))


;; ─── Some happy path — nested (:wat::core::Option :- [(:wat::core::Option :- [:wat::core::i64])]) ────────────────────

(:wat::test::deftest :wat-tests::core::option-expect::some-nested-option
  
  (:wat::core::let
    [opt (:wat::core::Some (:wat::core::Some 7))
     inner
      (:wat::core::Option/expect  
        opt
        "outer should be Some")
     v
      (:wat::core::Option/expect  
        inner
        "inner should be Some")]
    (:wat::test::assert-eq v 7)))


;; ─── :None panics with the supplied message ──────────────────────────


(:wat::test::deftest :wat-tests::core::option-expect::none-panics-with-message
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; Option/expect on :None panics; the crash reaches the parent's recv'
           ;; as Lost (carrying the LociDiedError) BEFORE the completion send'.
           (:wat::core::do
             (:wat::core::let
               [opt :wat::core::None
                _v
                 (:wat::core::Option/expect
                   opt
                   "broker disconnected")]
               nil)
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the :None expect above already panicked
               ;; before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected panic on :None expect, got clean completion"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::test::assert-eq
          (:wat::kernel::LociDiedError/message cause)
          "broker disconnected"))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected panic on :None expect, got clean close"
          :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
