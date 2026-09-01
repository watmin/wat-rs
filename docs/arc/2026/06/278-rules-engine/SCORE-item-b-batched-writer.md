# SCORE — item (b): the batched writer

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 324.026s] 5155 tests run: 5155 passed (3 slow), 15 skipped
FLOOR=0
```

5155 = 5150 + five `probe_arc278_span_batched` tests. **All twelve rows pass, no deltas.**

| # | what | result |
|---|---|---|
| 1 | ★ an over-cap buffer drains | ✅ `overcap_buffer_drains` — **RED this morning** |
| 2 | ★ partial progress is exact | ✅ `partial_progress_is_exact` |
| 3 | one item over the cap | ✅ `one_item_over_cap_is_request_too_large` |
| 4 | cut at `>` not `>=` | ✅ `exact_cap_chunk_is_sent`; `>` at `telemetry.wat:386, 405` |
| 5 | under-cap path unchanged | ✅ `undercap_is_one_write` |
| 6 | stone C still speaks | ✅ all four `_speak` gates |
| 7 | stones A/B hold | ✅ 23 span gates pass, and `git diff tests/` is **empty** — no prior gate touched at all, not even mechanically |
| 8 | no `Stream`, no `WriteResult` | ✅ the one hit is a comment recording the ruling |
| 9 | cap from the contract | ✅ four `MAX-REQUEST-BYTES` uses, no literal |
| 10 | no new surface op | ✅ Span 5, Journal 6 |
| 11 | no runtime change | ✅ empty |
| 12 | floor | ✅ 5155/5155, my own run |

**First stone in this campaign with no delta to report.** Every row landed as drawn, and the brief
needed no correction — the difference being that its scope ruling and its contract decision were both
made against measurements taken first (`Stream`/`WriteResult`/chunker all absent; the `>=` vs `>`
asymmetry read on both sides) rather than from the design doc's prose.

## What the two load-bearing rows actually prove

`overcap_buffer_drains` closes stone C's finding: a buffer past the cap now empties across multiple
submissions instead of sticking forever and growing with every log.

`partial_progress_is_exact` is the one that could not be caught anywhere else. A sink that accepts
chunk 1 and refuses chunk 2 leaves **exactly** the un-written suffix — a later drain lands `n`, not
`n+1` (duplicate logs) or `n-1` (lost logs). An off-by-one in the written count is invisible to every
other row in the table, in both directions.

## Where item (c) stands now

Its failure path terminates. The chain, all measured:

```
log/incr/timed  → buffer, arm on empty→non-empty         (stone A, B)
size cap        → flush, and SPEAK if it fails            (stone C)
over-cap buffer → fragment into ≤cap submissions, drain   (item b)
failed chunk    → keep exactly the un-written suffix      (item b)
```

## Still the builder's, and now genuinely bounded

`LogResponse::Ok` means "accepted"; failures are matchable. Backpressure, bounded buffers and drop
policy remain unruled — but the **unbounded** growth that made them urgent is gone: a buffer that can
always drain is a buffer whose size is bounded by its producer's rate, not by its own failure to
flush. That is a different and much smaller decision than the one this campaign started with.

Item (a) — `write-*-stream` over a lazy `Stream` — remains undrawn by ruling, not by omission: it
waits for a consumer that actually streams.
