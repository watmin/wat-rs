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
  [w <- :wat::kernel::Process'<(wat::core::i64,wat::core::i64),(wat::core::i64,wat::core::i64)>]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::send' w (:wat::core::Tuple 0 3))
     _ (:wat::kernel::send' w (:wat::core::Tuple 1 5))
     a (:wat::kernel::recv' w)
     b (:wat::kernel::recv' w)]
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
     w (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::concat
           (:wat::kernel::fn-forms wf :bracket::__pool-work)     ;; reify wf + its captured work-fn
           (:wat::core::forms
             (:wat::core::defn :bracket::__pool-runner
               [self <- :wat::kernel::Peer'<(wat::core::i64,wat::core::i64),(wat::core::i64,wat::core::i64)>]
               -> :wat::core::nil
               (:wat::core::let
                 [pair (:wat::kernel::recv' self)
                  _    (:wat::kernel::send' self (:bracket::__pool-work pair))]  ;; apply the reified wf to the pair
                 (:bracket::__pool-runner self)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:bracket::__pool-runner
                 (:wat::program::self-peer :(wat::core::i64,wat::core::i64) :(wat::core::i64,wat::core::i64)))))))]
    (:probe::drain w)))
