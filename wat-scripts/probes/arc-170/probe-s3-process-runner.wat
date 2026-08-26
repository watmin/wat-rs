;; probe-s3-process-runner.wat — the NOT-SHARED runner the widened bracket (S3) will use.
;;
;; Proves: only the user WORK-FN is reified via fn-forms; the pool-runner (recv (i,item) →
;; send (i, work item) → loop) is a NAMED defn shipped as source (like defservice's serve) —
;; no recursive-closure reification. The parent-side Process' type is pinned by a typed context
;; (a fn param here; the bracket's generic peers-vector element type in real use — same as how
;; defservice pins its Process' through (Launched :- [S R])). Index-carrying (idx,value) pairs.
;;
;; EXPECT "6 10".

;; typed drain: the param pins the Process' I/O (parent sends (idx,I), recvs (idx,O)); I=O=i64.
(:wat::core::defn :probe::drain
  [w <- (:wat::kernel::Process :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::core::match (:wat::kernel::send w (:wat::core::Tuple 0 3)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     _ (:wat::core::match (:wat::kernel::send w (:wat::core::Tuple 1 5)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::i64::to-string (:wat::core::second a))
        (:wat::string::concat " " (:wat::i64::to-string (:wat::core::second b)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat
           (:wat::kernel::fn-forms work :bracket::__work)
           (:wat::core::forms
             (:wat::core::defn :bracket::pool-runner
               [self <- (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
               -> :wat::core::nil
               (:wat::core::let
                 [pair (:wat::kernel::recv self)
                  out  (:wat::core::Tuple (:wat::core::first pair)
                                          (:bracket::__work (:wat::core::second pair)))
                  _    (:wat::core::match (:wat::kernel::send self out) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                 (:bracket::pool-runner self)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:bracket::pool-runner
                 (:wat::program::self-peer (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])))))))]
    (:probe::drain w)))
