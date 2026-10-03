;; Does a shipped GENERIC pool-runner resolve its types by inference from the concrete __work?
;;
;; DISPOSITION (255.75) — negative, and decisively answered NO: the child's own startup
;; type-checker originally reported 5 errors (surfacing as a StartupError inside
;; `:probe::drain`, since `--check` on the OUTER file passes clean — `pool-runner` is DATA, a
;; quoted `(forms ...)` literal, until the child spawns and type-checks its own received world).
;; Three were the `RecvOutcome` bare-bind + missing `SendOutcome.Stopped` class 255.73 repaired
;; elsewhere (`pool-runner` bare-bound `(recv self)` as `pair`, so `first`/`second pair` saw a
;; `RecvOutcome`, not the tuple; `send` was missing the `Stopped` arm) — REPAIRED here the same
;; way (match on `recv`, add the arm), which also un-masks a 3rd generic-class error that the
;; bare-bind's own type-error had been hiding (`:bracket::__work`'s own parameter check, below).
;;
;; The remaining 3 type-check errors are load-bearing and NOT that class: `pool-runner` is
;; declared generic over `[A B]`, and its body calls the MONOMORPHIC `:bracket::__work` (reified
;; via `fn-forms` from a concrete `i64->i64` fn) as if its argument/result had the abstract type
;; `B` — unsound unless B is unified to i64 specifically, and nothing in the declared signature
;; tells the checker that (a generic function's body is checked once against its OWN abstract
;; parameters, for every possible instantiation, never against one call site's concrete
;; substitution — ordinary, correct parametric-polymorphism checking, confirmed by hand-testing
;; both the keyword `:B` and bare `B` spellings: identical error either way). `probe-s3-process-runner.wat`
;; and `probe-s3c-rendezvous.wat` show the two approaches that DO work (a concretely-typed
;; runner, or a work-fn threaded through as a VALUE parameter) — a shipped runner cannot be
;; declared generic and secretly monomorphized by what it happens to call.
;;
;; AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it, driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting exactly the 3 remaining
;; TypeMismatch errors.
(wat.core/defn probe/drain
  [w :- (wat.kernel/Process :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
  :- wat.type/nil
  (wat.core/let
    [;; arc 278 #73 — a stop here is terminal like Lost/Closed for this discard-only send; the
     ;; recv' below faces the stop as its own outcome.
     _ (wat.core/match (wat.kernel/send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 0 3)) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])
     ra (wat.kernel/recv w)
     a  (wat.core/match ra
          [wat.kernel/RecvOutcome.Message {:msg m} m]
          [wat.kernel/RecvOutcome.Lost {:cause cause}
            (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message cause))]
          [wat.kernel/RecvOutcome.Stopped {}
            (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [wat.kernel/RecvOutcome.Closed {}
            (wat.kernel/assertion-failed! :message "recv': w closed unexpectedly")])]
    (wat.kernel/println (wat.i64/to-string (wat.core/second a)))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [work (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/* x 2))
     w (wat.test/spawn-peer (wat.spawn/process)
         (wat.core/concat
           (wat.kernel/fn-forms work bracket/__work)
           (wat.core/forms
             (wat.core/defn bracket/pool-runner :- [A B]
               [self :- (wat.kernel/Peer :- [(wat.type/Tuple :- [wat.type/i64 A]) (wat.type/Tuple :- [wat.type/i64 B])])]
               :- wat.type/nil
               (wat.core/match (wat.kernel/recv self)
                 [wat.kernel/RecvOutcome.Message {:msg pair}
                   (wat.core/let
                     [out (wat.type/Tuple :- [wat.type/i64 :B] (wat.core/first pair)
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
