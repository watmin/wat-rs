;; Gate 3 — a runner that crashes in the child must surface the REAL cause in the
;; collect-loop assertion message ("runner {idx} crashed: {cause}"), not a blind
;; "runner crashed". The work-fn divides by zero → the child panics → Lost{idx,cause}.
;;
;; DISPOSITION (255.75) — negative, by design: the claim IS that this crashes, with the
;; real cause named in the message. There is no catch form in wat that lets :user::main
;; face the bracket's own collect-loop AssertionFailure as a value, so the probe cannot be
;; made to exit 0 while still proving the crash path.
;;
;; AMEND: this file START UP CLEAN (`startup_from_file` succeeds — the DivisionByZero only
;; fires once the bracket actually RUNS a worker), and `tests/lint/every_wat_bad_fixture_actually_fails.rs`
;; forbids `.wat.bad` for a file that starts up clean. Per that gate's own remedy #1 ("the file
;; is a valid program and the NAME is wrong"), this stays a plain `.wat`, moved out of
;; `wat-scripts/probes/` (so the "every probe runs, exit 0" gate does not require it) to
;; `tests/process/fixtures/`, invoked and asserted by
;; `tests/process/probe_arc255_75_negative_probes.rs`, which asserts the raised message names
;; both "runner N crashed" (N is whichever worker's division-by-zero the bracket's collect-loop
;; observed first — not pinned to a specific index, since 3 workers race under real scheduling)
;; and "DivisionByZero" (the real cause, not a blind "runner crashed").
(wat.core/defn probe/boom [n :- wat.type/i64] :- wat.type/i64
  (wat.i64// n 0))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
    (wat.bracket/map (wat.spawn/process)
      (wat.type/Vector :- [wat.type/i64] 1 2 3)
      probe/boom)))
