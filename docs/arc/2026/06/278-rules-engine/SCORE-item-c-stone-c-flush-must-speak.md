# SCORE — item (c) stone C: a size-triggered flush must speak

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 321.419s] 5150 tests run: 5150 passed (3 slow), 15 skipped
FLOOR=0
```

5150 = 5145 + five `probe_arc278_span_speak` tests. **All eleven rows pass.**

| # | what | result |
|---|---|---|
| 1 | ★ the failure reaches the caller | ✅ `logs_size_flush_failure_reaches_the_caller` |
| 2 | ★ the arriving item survives | ✅ `logs_arriving_item_survives_failed_flush` (+ incr/timed twins) |
| 3 | same for metrics | ✅ both |
| 4 | `Ok` still means accepted | ✅ `ok_still_means_accepted` |
| 5 | no `_` wildcard | ✅ zero added |
| 6 | flush fns untouched | ✅ |
| 7 | no second success value | ✅ no `Buffered` |
| 8 | pass-through vocabulary | ✅ `:wat::query::` variants copied from `CloseResponse` |
| 9 | no runtime change | ✅ empty |
| 10 | stones A/B hold, assertions unedited | ✅ 13 prior gates pass, their diffs empty |
| 11 | floor | ✅ 5150/5150, my own run |

## Delta — my brief asserted a shape I had not read

The BRIEF said *"`incr` and `timed` have the identical shape against `flush-metrics`."* **`incr` had
no size trigger at all** — verified against the parent commit: zero `cap`/`flush-metrics` references
in that arm. I had read the `timed` arm and generalised to its sibling without looking.

Left alone, `IncrResponse` would have gained three variants that nothing could ever produce — a
surface that advertises failures it cannot report, which is the `experiri` defect exactly. The strike
gave `incr` the trigger it was missing, then taught all three arms to read the second element. That
is scope beyond the brief and it is correct: without it this stone half-fires.

## ★ Row 2 was impossible as written, and what that exposes is bigger than the row

I wrote *"after that failed flush, flush again against a working sink; every log lands."* It cannot
work as one write, and the reason is a real defect:

- the span triggers at **`>=`** — `span.wat:75, 123, 172`
- the server rejects at **`>`** — `service.wat:1779, 2132`

The trigger fires when `would >= cap`. Keeping the arriving item (which STOP-1 correctly demands —
dropping it would be silent data loss) leaves the buffer holding `would`. When `would > cap`, that
buffer **can never be flushed as a single write**: every subsequent flush is refused
`RequestTooLarge`, and each further log makes it larger. **A permanently unflushable, growing
buffer.**

The strike gated survival by counting the durable buffer through `span/stop` instead — sound, and it
proves what row 2 was for. But the thing it routed around is the finding.

★ **This is not stone C's doing.** It exists the moment a failed flush retains its buffer, which is
stone A's (correct) reset-only-on-success. Stone C only makes it reachable, because before this the
failure was invisible.

**And the arc already owns the fix.** `DESIGN-service-io-budgets.md` item **(b)** — `write-logs-batched`
/ `write-logs-stream`, *"writers FRAGMENT an oversized batch into ≤budget submissions"*. An
over-cap buffer is exactly what fragmentation exists for. Item (b) is **NOT built** — the same status
line that said item (c) was not built.

So the ordering that has been implicit all along is now explicit: **item (b) is not optional polish
downstream of (c); it is what makes (c)'s failure path terminate.** Recorded rather than chased.

## Still the builder's

`LogResponse::Ok` means "accepted", unchanged and now honest — a failure is matchable. Backpressure,
bounded buffers and drop policy remain unruled, and the unflushable-buffer finding above is the
sharpest argument yet that they need a ruling: today an over-cap buffer grows without bound and
nothing sheds it.
