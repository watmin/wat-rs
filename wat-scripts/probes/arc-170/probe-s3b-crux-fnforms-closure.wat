;; probe-s3b-crux-fnforms-closure.wat — the S3b crux: fn-forms of the index-wrapping
;; `wf` CLOSURE that captures the work-fn, shipped to a PROCESS runner.
;;
;; The bracket's process arm must fn-forms `wf` = (fn [pair] (Tuple (first pair) (work-fn (second pair)))),
;; a closure that CAPTURES work-fn. probe-s3-process-runner.wat proved fn-forms of a BARE work fn;
;; this proves fn-forms of a closure that captures another fn (the real bracket shape). If the capture
;; can't be reified, this RED-reports the exact gap (S1's ImpureCapture/portability boundary) before S3b.
;;
;; EXPECT "6 10".

;; parent-side drain: pins the Process' I/O (parent sends (idx,i64), recvs (idx,i64))
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
      (:wat::core::string::concat
        (:wat::core::i64::to-string (:wat::core::second a))
        (:wat::core::string::concat " " (:wat::core::i64::to-string (:wat::core::second b)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-fn (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
     ;; wf — the index-wrapping closure that CAPTURES work-fn (the exact bracket shape)
     wf (:wat::core::fn [pair <- :(wat::core::i64,wat::core::i64)] -> :(wat::core::i64,wat::core::i64)
          (:wat::core::Tuple (:wat::core::first pair)
                             (work-fn (:wat::core::second pair))))
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat
           (:wat::kernel::fn-forms wf :bracket::__pool-work)     ;; reify wf + its captured work-fn
           (:wat::core::forms
             (:wat::core::defn :bracket::__pool-runner
               [self <- (:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
               -> :wat::core::nil
               (:wat::core::let
                 [pair (:wat::kernel::recv self)
                  _    (:wat::core::match (:wat::kernel::send self (:bracket::__pool-work pair)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]  ;; apply the reified wf to the pair
                 (:bracket::__pool-runner self)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:bracket::__pool-runner
                 (:wat::program::self-peer :(wat::core::i64,wat::core::i64) :(wat::core::i64,wat::core::i64)))))))]
    (:probe::drain w)))
