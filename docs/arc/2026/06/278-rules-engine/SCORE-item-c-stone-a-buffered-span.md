# SCORE — item (c) stone A: the buffered span

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me.

```
Summary [ 319.271s] 5140 tests run: 5140 passed (3 slow), 15 skipped
FLOOR=0
```

| # | what | result |
|---|---|---|
| 1 | ★ **no double-count** | ✅ `incr_flush_incr_close_sums_to_exactly_five` |
| 2 | logs batched | ✅ `log` **conj**s; the only `write-logs` sits behind the size trigger |
| 3 | logs survive the flush | ✅ `buffered_logs_all_land_on_close` |
| 4 | duration aggregate unchanged | ✅ `<name>/count` + `<name>/duration` |
| 5 | duration fidelity added | ✅ `duration_emits_count_sum_and_one_sample_per_sample` |
| 6 | reset resets | ✅ `second_flush_and_empty_close_emit_nothing` |
| 7 | `close` is the remainder | ✅ same gate |
| 8 | threshold from the contract | ✅ `WRITE-{LOGS,METRICS}-MAX-REQUEST-BYTES`, no literal |
| 9 | no timer | ✅ grep empty |
| 10 | blast radius | ✅ `wat/telemetry{,/span}.wat` + gates |
| 11 | existing gates hold | ✅ all 8 span probes |
| 12 | floor | ✅ 5140/5140, my own run |

**One path, verified structurally:** `flush-accumulators` is defined once (`span.wat:322`) and called
from four sites — both size triggers, the `flush` op, and `close`. There is no second emission path
for the double-count to hide in.

## The delta, and it is MY specification error

The strike added `Span/flush` — a **new surface op**, which my DESIGN listed under "Out of scope =
REJECTED: any new surface."

The strike is right and the brief was self-contradictory. Three of my own constraints cannot all
hold:

- EXPECTATIONS row 1 demands a **mid-life flush** (`incr ×3 → flush → incr ×2 → close`);
- STOP-2 forbids a **timer** in this stone;
- the DESIGN forbids a **new surface op**.

With no timer and no op, the only way to trigger a mid-life flush is to cross the size threshold —
~10 MiB of metrics. My own acceptance criterion required the thing my own scope forbade. The strike
picked the right constraint to break and reported it plainly.

And `flush` earns its place independently: "close is the remainder" is a claim that only means
something if a mid-life flush exists, and every agent in this class (statsd, the CloudWatch agent)
exposes a forced flush.

## ★ What I found while grading, which the green floor was hiding

`Span` now declares **five** ops. `probe_arc278_span_surface` — whose entire job is *"every declared
op is reachable and replies"* — was driving **four**, and passing. Its toy satisfier declares
`:satisfies :wat::telemetry::Span` and implements four arms; **that compiles**, because
`serve-op-arms` folds over `:impls` and an unimplemented surface op simply gets no arm. Nothing
compares `:impls` to the surface's `:features`.

The tell was the op COUNT in the test's own name — `..._all_four_ops_reply` — which a green run can
never contradict.

Fixed here: the toy satisfier gained its `flush` arm, the probe drives it, and the test is renamed
off the count (`..._every_declared_op_replies`), because a name carrying a number goes stale silently
the moment the surface grows.

**The underlying guard is NOT built**, and it is the sibling of excursus 001 stone 5's `:messages`
completeness guard — same failure geometry, one clause over. Recorded in
`NOTE-impls-completeness-is-unenforced.md` rather than chased.

## Left open, deliberately (STOP-4)

`LogResponse::Ok` now means **"buffered"**, not "written". No new variant was invented to paper over
it. Whether a buffered write's failure should reach the pusher, surface at `close`, or be accepted as
lossy-on-crash is a contract question and it is the builder's to rule on.

## Next

Stone B — two internal ops at independent cadences (`-flush-logs` fast, `-flush-counters` at 30s).
Pure scheduling on the flush path this stone built; `Alarm` already carries its own `after` AND its
own `op`, so nothing new is needed for two clocks.
