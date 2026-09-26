# WEIGH — STONE 255.44: the span reports what the sink said — ACCEPTED

**Executor: grok via pulsare, commit `145d2434a`.** Weighed by the orchestrator against disk on 2026-09-26.

## Re-measured

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-26T00-51-04Z` | `6141 tests run: 6141 passed (10 slow), 22 skipped` |
| the four rows | the floor log | all PASS: `a_successful_write_is_ok`, `a_fatal_write_is_fatal`, `a_constraint_write_is_constraint`, `a_transient_write_is_transient` |
| `_w` in `wat/telemetry/span.wat` | grep | **gone** |
| the IDE's E0716 "temporary dropped while borrowed" (`probe_arc255_44_…rs:56/63/70/80`) | read line 56 | **stale**: the file binds `let got = log_through(…)` first; the tests compiled and passed |

Taken from the SCORE: pre-stone the three failure rows failed with `left: "Ok"` against `right: "Fatal"` /
`"Constraint"` / `"Transient"`, and the success row passed; clippy 0; census `no STOP-8`; delta NEW 2 /
RECOVERY 0; ledger 198.

## What landed

- **`Span::LogResponse` mirrors `CloseResponse`** (`:Constraint`/`:Transient`/`:Fatal` carrying `err`, with
  the same field names and types).
- **The span matches the journal write and replies the honest arm.** The wire-breach and gone-journal arms
  copy the close producer's own handling (a precedent existed for every arm, so there was no stop), so the
  siblings cannot disagree.
- **No exhaustive `LogResponse` consumer existed** besides the producer. The `:wat::telemetry::log` macro
  hands its caller the `RecvOutcome`.

**With 255.43, the telemetry stdlib no longer swallows its sink's failures.**
