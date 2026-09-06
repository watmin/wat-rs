# SCORE — what does publish actually cost

**STRUCK. Measurement only; no production change.** Executor: grok, 2026-09-05.
Tree safe, uncommitted. One new file: `wat-scripts/scratch-pad/probe-what-publish-costs.wat`.

```
Summary [ 377.024s] 5215 tests run: 5215 passed (5 slow), 22 skipped
```

`.floor/2026-09-05T23-58-56Z/`

The model does not close. The gap is the deliverable.

## THE FIVE UNITS — one run, quiet box

`ps` before the timing process: grok 5.3 %, claude 5.1 %, everything else < 1 %. Same box as the 37.3 s median.

```
rt_thread_us=144 rt_process_us=191 put_us=696 count_us=528 scan1_us=566
count_n=60 scan1_n=1
```

| unit | µs/op | ×8000 | share of 37.3 s |
|---|---|---|---|
| ★ **bare RT, thread** | **144** | 1.15 s | 3 % per call site |
| bare RT, process | 191 | 1.53 s | 4 % per call site |
| `Store/put` (process sqlite, depth ~60) | **696** | 5.57 s | **15 %** |
| `Store/count-index` | 528 | 4.22 s | 11 % |
| `Store/scan-index` limit 1 | 566 | 4.53 s | contrast |

count-index on process-locus sqlite is 528 µs vs last stone's 467 µs on thread. Same order. `count_n=60` / `scan1_n=1` — the fill held.

## ★ ROW 3 — the interpretation floor

**A bare thread-locus round trip is 144 µs.** That is the closest number this tree has to *"wat's interpretation + dispatch."* Every generated client method pays it. 8000 × 144 µs = **1.15 s**. Process adds 47 µs of IPC (191 − 144). Interpretation is **not** the leader; it is a few percent per call.

The ping arm returns a constant `Pong`. No format, no state, no extra alloc. STOP-1 did not fire.

## ★★ ROW 2 — the arithmetic, stated

DESIGN formula, as written:

```
predicted = 8000 × (put + count-index + 2 × round-trip-process)
          = 8000 × (696 + 528 + 2×191) µs
          = 12848 ms
actual    = 37300 ms
gap       = 24452 ms     ← 66 % of publish, UNATTRIBUTED
```

Honest model (STOP-3 — see below):

```
honest    = 8000 × (put + count-index)     because those already include a process RT
          = 8000 × (696 + 528) µs
          = 9792 ms
gap       = 27508 ms     ← 74 % unattributed
```

**The gap is large. The model is blind.** That is the finding.

## STOP-3 — the real call count

A `Queue/send` does **two** store client calls: `count-index` then `put` (`sqs.wat:319`, `:348`). Measuring `Store/put` and `Store/count-index` already includes one process-locus round trip each. Adding `2 × round-trip` on top **double-counts** dispatch.

The extra RPC per send is **one**: the `Queue/send` itself (topic-worker → queue).

Further: `publish=` times **2000** sequential `Topic/publish` from main (`circuit.wat:1399-1404`, n=2000), not 8000 sequential store ops. The 8000 queue-sends (n×m) run on topic workers, concurrent with that loop, gated by inbox cap 64.

## ★ ROW 7 — the next target

**Do not fire at `put`.** It is the largest *measured* primitive (15 %), and shrinking it cannot close a 74 % gap.

**The next stone should stamp the publish path in-band** (the DESIGN's named fallback, now earned): `Topic/publish` → inbox `Queue/send` → topic-worker fanout → m× `Queue/send`. Unit × count of the two store verbs does not explain the wall. The 37.3 s is main blocked on inbox capacity while workers drain; that wait is exactly the `outbox` histogram, and it is not in the five units.

Among isolated units, if a later stone *does* cut a measured verb: **put first** (5.6 s), then count-index (4.2 s). Neither is the 24 s.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ five unit costs, one run | ✅ 144 / 191 / 696 / 528 / 566 µs |
| 2 | ★★ the arithmetic | ✅ predicted 12848 ms, actual 37300, **gap 24452 ms** |
| 3 | ★ interpretation floor | ✅ **144 µs** thread-locus bare ping |
| 4 | ⛔ blast radius | ✅ untracked `wat-scripts/scratch-pad/probe-what-publish-costs.wat` only |
| 5 | ⛔ box quiet | ✅ `ps` before the run: grok 5.3, claude 5.1, else < 1 |
| 6 | floor untouched | ✅ `5215 passed`, 22 skipped |
| 7 | ★ next target | **in-band publish-path stamps** — the 24.5 s gap. Not put. |

## NOT TOUCHED

`wat/`. `src/`. `sqs.wat`. `circuit.wat`. No fix. No rebuild.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-06

**STRUCK.** All rows re-run by me; units within 4 % of the executor's.

```
mine   rt_thread=143  rt_process=179  put=675  count=517  scan1=573   gap_ms=24900
grok   rt_thread=144  rt_process=191  put=696  count=528  scan1=566   gap_ms=24452
floor  Summary [ 371.027s] 5215 passed, 22 skipped — .floor/2026-09-06T00-11-34Z/, 0 FAIL
```

★ **STOP-3 fired correctly and it was my DESIGN that was wrong.** My formula double-counted
dispatch (`put`/`count` already include a process round trip), and it assumed 8000 sequential
publishes when `publish=` times **2000** (`circuit.wat:1400`, `foldl` over `range 0 n`, n=2000).
The executor said so instead of fitting the model to the answer.

## ★★★ I RAN THE DECISIVE EXPERIMENT THE UNITS POINTED AT — AND IT INVALIDATES THE METRIC

`publish-until-accepted!*` (`circuit.wat:965-979`) retries on `PublishResponse::Full` with
`await-timer-ms 1`. So `publish=` is largely **main sleeping in 1 ms increments against a cap-64
inbox.** Testable in one run — inbox cap 64 → 4096:

| | publish | drain | **publish + drain** |
|---|---|---|---|
| cap 64 | 37.3 s | 0.24 s | **37.5 s** |
| cap 4096 | **22.4 s** | **15.0 s** | **37.4 s** |

`total=8000; distinct=8000; dup=0` in both. `outbox` went `max=342 ms` → `max=20218 ms`,
`>1000=7728`.

★★ **The 40 % "win" is pure displacement.** Raising the cap moves 15 s out of `publish` and into
`drain`; **throughput is identical to within 0.3 %.** The cap is not a perf knob, it is a
*queueing-location* knob.

⛔ **`publish` alone is a Goodhart metric, and I had drawn two stones aimed at it.** The honest
measure is **`publish + drain`** — or `8000 / (publish + drain)` as throughput. Tree reverted; the
experiment was a probe, not a change.

## ★ ROW 3 — THE INTERPRETATION FLOOR IS 143 µs

A bare thread-locus round trip. Every generated client method pays it; process adds ~36 µs of
IPC. 8000 × 143 µs ≈ 1.1 s against a ~37.5 s system.

**Interpretation is nowhere near the leader.** The builder's terminal condition is a long way off,
and that is now a measured statement rather than an assumption.

## ROW 7 — THE NEXT TARGET, CORRECTED

Grok said *"in-band publish-path stamps."* **The cap experiment supersedes that.** We no longer
need to find where publish's wall goes — we know: **main waits, because the topic worker drains
at a fixed rate, and that rate is the system's true throughput.**

★ So the next stone targets **the topic worker's per-message drain cost**, and it is measured
against **`publish + drain`**, which the cap cannot move. Known components of one drained message:
`Queue/receive` + `Seen/check` + `Seen/mark` + `Queue/ack` + m× `Queue/send` — each ≥ one
process round trip (179 µs) plus a store verb (`put` 675 µs, `count` 517 µs).

**`put` at 675 µs is now the largest measured primitive on that path**, and unlike `publish` it
cannot be optimized by moving the queue somewhere else.
