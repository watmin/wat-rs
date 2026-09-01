# SCORE — item (c) stone B: two clocks

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 325.306s] 5145 tests run: 5145 passed (3 slow), 15 skipped
FLOOR=0
```

5145 = 5140 + five `probe_arc278_span_clocks` tests.

| # | what | result |
|---|---|---|
| 1 | ★ only-logs flushes ZERO metrics | ✅ `only_logs_flushes_logs_and_zero_metrics` |
| 2 | ★ only-counts flushes ZERO logs | ✅ `only_counts_flushes_metrics_and_zero_logs` |
| 3 | the tick re-arms | ✅ `tick_rearms_without_client_flush` |
| 4 | an idle span is silent | ✅ `idle_span_is_silent` |
| 5 | no double-count survives the split | ✅ stone A's `incr_flush_incr_close_sums_to_exactly_five` |
| 6 | one builder per accumulator | ✅ see below |
| 7 | size triggers decoupled | ✅ `:99` metrics-only, `:147` logs-only |
| 8 | stone A's gates unedited | ✅ **in the sense that matters** — see the delta |
| 9 | no armed flag | ✅ zero hits; `wat/telemetry.wat` diff empty |
| 10 | cadence configurable | ✅ `non_default_cadence_is_honoured` (20ms vs 2000ms) |
| 11 | surface still five ops | ✅ 5 |
| 12 | no runtime change | ✅ empty diff |
| 13 | time is I/O | ✅ `select'` on a one-shot `after`; the only "sleep" in the file is prose forbidding one |
| 14 | floor | ✅ 5145/5145, my own run |

**Row 6, verified structurally.** `flush-logs` (`:361`) and `flush-metrics` (`:388`) are each defined
ONCE. `flush-logs` is called by the logs size trigger (`:147`), its own timer (`:186`), and the
composition; `flush-metrics` by the metrics size trigger (`:99`), its own timer (`:191`), and the
composition. `flush-accumulators` (`:415`) is now *only* composition, called by `Span/flush` and
`close`. That is the invariant exactly: one emit-and-reset path per accumulator, shared by its
timer, its size trigger, and `close`.

## ★ Row 8 was my row, and it was the wrong test

I wrote *"`git diff tests/…/probe_arc278_span_buffered.*` → empty."* It is not empty, and the strike
reported that plainly rather than quietly.

The diff is **four Record constructions gaining the two cadence fields, and nothing else** — kwargs
construction requires every field, so a Record that grows forces every construction site to follow or
the freeze fails. Not one assertion, not one line of test logic. The `.rs` is untouched. It is the
same class of edit stone A made when `:logs` joined the Record.

My row conflated *"the gate was not weakened"* with *"the file did not change."* The first is the
thing I cared about; the second is a proxy that a required-field Record makes false for purely
mechanical reasons. **The right test is "assertions unedited", and I should have written that.** A
future row on a Record-bearing gate should say so.

## Delta — the BRIEF said "re-arm itself", the DESIGN said otherwise

BRIEF step 2 said each timer *"re-arms itself at its own interval — unless its accumulator is
empty."* The strike deferred to the DESIGN, which is the authority, and the two are the same fact:
**after a flush the accumulator is ALWAYS empty**, so the timer never re-arms. The next accumulation
re-arms via the empty→non-empty transition. `-flush-logs`/`-flush-metrics` return `NoReply`.

The BRIEF's phrasing was redundant in a way that reads as a second mechanism. It is not; a timer that
re-armed itself here would tick forever on an empty span, which is exactly what row 4 forbids.

## Deltas the strike named

- Cadence lives on the defservice Record in `span.wat`, not `telemetry.wat` — which is why row 9's
  `wat/telemetry.wat` diff is empty. Correct: the Record is the service's, not the surface's.
- Defaults `logs-flush-after-ms` 1000, `metrics-flush-after-ms` 30000.

## Item (c) is now built

`DESIGN-service-io-budgets.md` has recorded item (c) as *"NOT built (nothing on the disk)"* since
2026-07-21. It is built: the span buffers, emits deltas from one path per accumulator, flushes on
size from the declared contract and on two independent clocks, and emits both aggregate and fidelity
for durations. That file's status line should be updated by whoever next touches it.

## Still open, and still the builder's

`LogResponse::Ok` means **"buffered"**, not "written" — unchanged from stone A, no variant invented.
Backpressure, bounded buffers, and a drop policy when a flush fails are all downstream of that one
ruling. Jitter/backoff on the cadence remains rejected until there is a measured need.
