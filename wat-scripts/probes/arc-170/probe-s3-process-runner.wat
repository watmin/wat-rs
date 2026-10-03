;; probe-s3-process-runner.wat — the NOT-SHARED runner the widened bracket (S3) will use.
;;
;; Proves: only the user WORK-FN is reified via fn-forms; the pool-runner (recv (i,item) →
;; send (i, work item) → loop) is a NAMED defn shipped as source (like defservice's serve) —
;; no recursive-closure reification. The parent-side Process' type is pinned by a typed context
;; (a fn param here; the bracket's generic peers-vector element type in real use — same as how
;; defservice pins its Process' through (Launched :- [S R])). Index-carrying (idx,value) pairs.
;;
;; EXPECT "6 10".
;;
;; DISPOSITION (255.75) — repair: `bracket::pool-runner` bare-bound `(recv self)` as `pair`
;; instead of matching the `RecvOutcome`, and its `send` match was missing
;; `SendOutcome.Stopped` — the same defect class 255.73's `probe-s3b-astsplice.wat` site-2
;; repaired. Repaired the same way. Also now asserts its own claim.

;; typed drain: the param pins the Process' I/O (parent sends (idx,I), recvs (idx,O)); I=O=i64.
(wat.core/defn probe/drain
  [w :- (wat.kernel/Process :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
  :- wat.type/nil
  (wat.core/let
    [_ (wat.core/match (wat.kernel/send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 0 3)) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])
     _ (wat.core/match (wat.kernel/send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 5)) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])
     ra (wat.kernel/recv w)
     a  (wat.core/match ra
          [wat.kernel/RecvOutcome.Message {:msg m} m]
          [wat.kernel/RecvOutcome.Lost {:cause cause}
            (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message cause))]
          [wat.kernel/RecvOutcome.Stopped {}
            (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [wat.kernel/RecvOutcome.Closed {}
            (wat.kernel/assertion-failed! :message "recv': w closed unexpectedly")])
     rb (wat.kernel/recv w)
     b  (wat.core/match rb
          [wat.kernel/RecvOutcome.Message {:msg m} m]
          [wat.kernel/RecvOutcome.Lost {:cause cause}
            (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message cause))]
          [wat.kernel/RecvOutcome.Stopped {}
            (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [wat.kernel/RecvOutcome.Closed {}
            (wat.kernel/assertion-failed! :message "recv': w closed unexpectedly")])]
    (wat.core/do
      (wat.kernel/println
        (wat.string/concat
          (wat.i64/to-string (wat.core/second a))
          (wat.string/concat " " (wat.i64/to-string (wat.core/second b)))))
      (wat.test/assert-eq (wat.core/second a) 6)
      (wat.test/assert-eq (wat.core/second b) 10))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [work (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/* x 2))
     w (wat.test/spawn-peer (wat.spawn/process)
         (wat.core/concat
           (wat.kernel/fn-forms work bracket/__work)
           (wat.core/forms
             (wat.core/defn bracket/pool-runner
               [self :- (wat.kernel/Peer :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
               :- wat.type/nil
               (wat.core/match (wat.kernel/recv self)
                 [wat.kernel/RecvOutcome.Message {:msg pair}
                   (wat.core/let
                     [out (wat.type/Tuple :- [wat.type/i64 wat.type/i64] (wat.core/first pair)
                                            (bracket/__work (wat.core/second pair)))
                      _   (wat.core/match (wat.kernel/send self out) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])]
                     (bracket/pool-runner self))]
                 [wat.kernel/RecvOutcome.Lost {:cause cause}
                   (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message cause))]
                 [wat.kernel/RecvOutcome.Stopped {} nil]
                 [wat.kernel/RecvOutcome.Closed {} nil]))
             (wat.core/defn user/main [] :- wat.type/nil
               (bracket/pool-runner
                 (wat.program/self-peer (wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])))))))]
    (probe/drain w)))
