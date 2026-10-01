;; probe-s3b-crux-fnforms-closure.wat — the S3b crux: fn-forms of the index-wrapping
;; `wf` CLOSURE that captures the work-fn, shipped to a PROCESS runner.
;;
;; The bracket's process arm must fn-forms `wf` = (fn [pair] (Tuple (first pair) (work-fn (second pair)))),
;; a closure that CAPTURES work-fn. probe-s3-process-runner.wat proved fn-forms of a BARE work fn;
;; this proves fn-forms of a closure that captures another fn (the real bracket shape). If the capture
;; can't be reified, this RED-reports the exact gap (S1's ImpureCapture/portability boundary) before S3b.
;;
;; EXPECT "6 10".
;;
;; DISPOSITION (255.75) — negative, and the header's "EXPECT" above is STALE:
;; `docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md` ("Why raw, not `wf`")
;; names this exact probe RED BY DESIGN — "disconfirming probe … RED, proved it": `fn-forms`
;; (closure_extract slice-1) cannot reify a closure that captures a fn value
;; (`closure_extract.rs:2025`, a deliberate slice-1 limit, same bucket as `fn`/`Stream`), which is
;; why S3b ships the RAW work-fn via `fn-forms` and index-wraps separately on the process side
;; (`probe-s3-process-runner.wat`, green standalone) instead of `fn-forms`-ing the index-wrapping
;; closure this probe builds. a plain `.wat` (AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it), driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting the error names
;; "closure-extract" and "not implemented" for a captured `wat::core::fn`.

;; parent-side drain: pins the Process' I/O (parent sends (idx,i64), recvs (idx,i64))
(:wat::core::defn :probe::drain
  [w <- (:wat::kernel::Process :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
  -> wat.type/nil
  (:wat::core::let
    [_ (:wat::core::match (:wat::kernel::send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 0 3)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     _ (:wat::core::match (:wat::kernel::send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 5)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::i64::to-string (:wat::core::second a))
        (:wat::string::concat " " (:wat::i64::to-string (:wat::core::second b)))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work-fn (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
     ;; wf — the index-wrapping closure that CAPTURES work-fn (the exact bracket shape)
     wf (:wat::core::fn [pair <- (wat.type/Tuple :- [wat.type/i64 wat.type/i64])] -> (wat.type/Tuple :- [wat.type/i64 wat.type/i64])
          (wat.type/Tuple :- [wat.type/i64 wat.type/i64] (:wat::core::first pair)
                             (work-fn (:wat::core::second pair))))
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat
           (:wat::kernel::fn-forms wf :bracket::__pool-work)     ;; reify wf + its captured work-fn
           (:wat::core::forms
             (:wat::core::defn :bracket::__pool-runner
               [self <- (:wat::kernel::Peer :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
               -> wat.type/nil
               (:wat::core::let
                 [pair (:wat::kernel::recv self)
                  _    (:wat::core::match (:wat::kernel::send self (:bracket::__pool-work pair)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])]  ;; apply the reified wf to the pair
                 (:bracket::__pool-runner self)))
             (:wat::core::defn :user::main [] -> wat.type/nil
               (:bracket::__pool-runner
                 (:wat::program::self-peer (wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])))))))]
    (:probe::drain w)))
