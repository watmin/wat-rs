;; probe-s3c-rendezvous.wat — the redesign's load-bearing composition (259 S3c).
;;
;; The current process arm SHIPS the runner and lets it reference the work-fn BY NAME
;; (a closure over a shipped symbol). To BAKE the runner into :wat::bracket:: (reserved,
;; privileged, zero user input) it CANNOT reference a :user::bracket:: name — that would be
;; a stdlib -> user-data forward reference the resolver rejects. So the runner must take
;; the work-fn as a VALUE argument and thread it through its recursion (like the thread
;; runner-loop already does), and :user::main passes it from the RENDEZVOUS coordinate.
;;
;; This probe proves that composition with the runner still SHIPPED (baking it is then pure
;; relocation into stdlib). It proves:
;;   (1) the runner takes work-fn as a VALUE (so it can be baked/generic — no by-name ref),
;;   (2) the work-fn lives at the rendezvous coordinate :user::bracket::work-fn (non-reserved,
;;       ships clean — no reserved gate, no underscores/"internal" markers),
;;   (3) :user::main looks up that coordinate and drives the runner.
;;
;; EXPECT "6 10".
;;
;; DISPOSITION (255.75) — repair: `bracket::pool-runner` bare-bound `(recv self)` as `pair`
;; instead of matching the `RecvOutcome`, and its `send` match was missing
;; `SendOutcome.Stopped` — the same defect class 255.73's `probe-s3b-astsplice.wat` site-2
;; repaired. Repaired the same way. Also now asserts its own claim.

;; typed drain: pins the Process' I/O (parent sends (idx,I), recvs (idx,O)); I=O=i64.
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
           ;; the user's work-fn, reified to the RENDEZVOUS coordinate (non-reserved, clean name)
           (wat.kernel/fn-forms work user.bracket/work-fn)
           (wat.core/forms
             ;; the GENERIC runner (baked into :wat::bracket:: in the real strike) — takes the
             ;; work-fn as a VALUE and threads it through the recursion; NO by-name reference.
             (wat.core/defn bracket/pool-runner
               [self    :- (wat.kernel/Peer :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])
                work-fn :- [wat.type/i64 :-> wat.type/i64]]
               :- wat.type/nil
               (wat.core/match (wat.kernel/recv self)
                 [wat.kernel/RecvOutcome.Message {:msg pair}
                   (wat.core/let
                     [out (wat.type/Tuple :- [wat.type/i64 wat.type/i64] (wat.core/first pair)
                                            (work-fn (wat.core/second pair)))
                      _   (wat.core/match (wat.kernel/send self out) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])]
                     (bracket/pool-runner self work-fn))]
                 [wat.kernel/RecvOutcome.Lost {:cause cause}
                   (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message cause))]
                 [wat.kernel/RecvOutcome.Stopped {} nil]
                 [wat.kernel/RecvOutcome.Closed {} nil]))
             ;; :user::main looks up the rendezvous coordinate and PASSES the work-fn value.
             (wat.core/defn user/main [] :- wat.type/nil
               (bracket/pool-runner
                 (wat.program/self-peer (wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64]))
                 user.bracket/work-fn)))))]
    (probe/drain w)))
