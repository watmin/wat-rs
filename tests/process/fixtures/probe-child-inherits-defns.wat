;; probe-child-inherits-defns.wat — DECISIVE probe: does a not-shared (process) child
;; inherit the PARENT's named defns, or is it a fresh universe that needs source shipped?
;;
;; :probe::dbl is defined in the PARENT ONLY. The shipped (forms ...) does NOT include it —
;; the child's runner references :probe::dbl BY NAME. If the child inherits the parent's
;; defns, this streams "6 10". If the child is a fresh universe, it fails "unknown :probe::dbl".
;;
;; This decides the bracket's not-shared delivery: reference-by-name (inherit) vs ship-source.
;;
;; DISPOSITION (255.75) — negative, and decisive: today's measured answer IS the second branch —
;; the child is a fresh universe; it fails `UnresolvedReferences` naming `:probe::dbl`.
;;
;; AMEND: this file starts up clean (`startup_from_file` succeeds — the UnresolvedReferences only
;; surfaces once the CHILD process itself starts, not the parent file), so
;; `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids `.wat.bad`; moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` as a plain `.wat` instead, driven by
;; `tests/process/probe_arc255_75_negative_probes.rs::child_inherits_defns_is_refused`, which
;; asserts that error. The `rune:lint(nested-program, expected)` comment right below is load-bearing
;; for `tests/lint/nested_program_starts.rs`'s OWN gate (measured directly: deleting it turns that
;; gate RED — "1 nested program(s) failed child startup" — on this exact form; restoring it turns
;; that gate green again), even though the test name it carries,
;; `deftest_wat_tests_process_child_is_fresh_universe`, is not a test anywhere in this tree
;; (grepped) — left exactly as committed, not touched by this stone.

;; defined in the PARENT universe only
(wat.core/defn probe/dbl [x :- wat.type/i64] :- wat.type/i64
  (wat.i64/* x 2))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [w (wat.test/spawn-peer (wat.spawn/process)
         ;; rune:lint(nested-program, expected) — test(deftest_wat_tests_process_child_is_fresh_universe)
         (wat.core/forms
           ;; NOTE: :probe::dbl is NOT redefined here — the child references it BY NAME.
           (wat.core/defn probe/runner
             [self :- (wat.kernel/Peer :- [wat.type/i64 wat.type/i64])] :- wat.type/nil
             (wat.core/let
               [item (wat.core/match (wat.kernel/recv self) [wat.kernel/RecvOutcome.Message {:msg __d} __d] [wat.kernel/RecvOutcome.Lost {:cause __c} (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message __c))] [wat.kernel/RecvOutcome.Stopped {} (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [wat.kernel/RecvOutcome.Closed {} (wat.kernel/assertion-failed! :message "recv': peer closed")])
                _    (wat.core/match (wat.kernel/send self (probe/dbl item)) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil] [wat.kernel/SendOutcome.Stopped {} nil])]
               (probe/runner self)))
           (wat.core/defn user/main [] :- wat.type/nil
             (probe/runner (wat.program/self-peer wat.type/i64 wat.type/i64)))))
     ;; arc 278 #73 — a stop here is terminal like Lost/Closed for this discard-only send; the
     ;; recv's below face the stop as its own outcome.
     _ (wat.core/match (wat.kernel/send w 3) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])
     _ (wat.core/match (wat.kernel/send w 5) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])
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
    (wat.kernel/println
      (wat.string/concat
        (wat.i64/to-string a)
        (wat.string/concat " " (wat.i64/to-string b))))))
