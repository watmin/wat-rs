;; probe-s1-impure-gate.wat — does `fn-forms` refuse to reify a closure that impurely captures a
;; live Process handle `p` (rather than silently serializing it)? Success is that it RAISES, never
;; reaching the "LEAK" println below.
;;
;; DISPOSITION (255.75) — negative, by design (the "LEAK:" println text names the failure
;; condition): today's refusal is `src/closure_extract.rs`'s general "arms slice 1 doesn't yet
;; encode" wall (a RustOpaque-backed Process is in the same not-yet-implemented bucket as `fn`,
;; `Vector`, `Instant`, …, per that file's own comment: "surface as Internal so a surfacing test
;; reveals the gap" — this probe IS that surfacing test), not a bespoke "impure" check — but the
;; probe's claim (no leak) holds either way. a plain `.wat` (AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it), driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting the error names
;; "closure-extract", "not implemented", and ":wat::kernel::Process", and that "LEAK" never
;; prints.
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [p  (wat.test/spawn-peer (wat.spawn/process)
          (wat.core/forms (wat.core/defn user/main [] :- wat.type/nil nil)))
     f  (wat.core/fn [x :- wat.type/i64] :- wat.type/i64
          (wat.core/let [_ (wat.core/match (wat.kernel/send p x) [wat.kernel/SendOutcome.Sent {} nil] [wat.kernel/SendOutcome.HandleClosed {} nil] [wat.kernel/SendOutcome.Stopped {} nil] [wat.kernel/SendOutcome.Closed {:cause _c} nil] [wat.kernel/SendOutcome.Failed {:cause _c} nil])] x))
     wf (wat.kernel/fn-forms f probe/work)]
    (wat.kernel/println "LEAK: impure capture was reified without error")))
