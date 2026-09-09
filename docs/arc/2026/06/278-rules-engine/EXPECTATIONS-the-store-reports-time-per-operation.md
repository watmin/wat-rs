# EXPECTATIONS — the store reports time per operation

Written **before** the strike. Instrument plus a three-point sweep. `sqs.wat` and `circuit.wat` only.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **four op pairs, distinct** | read the diff | `put-calls/ns`, `delete-calls/ns`, `count-calls/ns`, `scan-calls/ns` — never summed, never reusing `store-ns` |
| 2 | ★ **the split reconciles** | the per-tier line | `put-ns + delete-ns + count-ns + scan-ns` equals `store-ns` within rounding, and calls likewise. **A gap means an op is unaccounted — report it; it is worth more than the split** |
| 3 | ★ **per-op mean latency at all three n** | the sweep table | n=1000/2000/4000, ≥3 runs each, `vis-ms=1000`. Per op: `ns/calls` at each n, and the ratios |
| 4 | ★ **the instrument is free** | wall clock and phase times | within run-to-run variance of the n=2000 band (WALL 39–40 s, fill 15.5–17.3 s). **Not store-calls** — that axis missed a quadratic term earlier in this arc |
| 5 | **counters live on `:ephemeral`** | `git diff` | `:queue::queue::Record` **not** in the diff; no Record constructor changes |
| 6 | **correctness at every point** | every run | `distinct = n×m`, `dup=0` |
| 7 | **blast radius** | `git diff --stat` | `wat-scripts/queue/sqs.wat` and `wat-scripts/fanout/circuit.wat` only. **No `wat/`** |
| 8 | **the box was quiet** | the SCORE | load stated per run; floor **after** the sweep, never beside it |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★★ **Row 2 is the non-vacuity gate.** Four counters that do not add up to the aggregate they subdivide are
four plausible numbers with an unaccounted remainder. This arc has already had a split fail exactly this way —
three failure-arm counters read 0/0/0 against 1238 rejections, and the gap named a source the DESIGN had
missed. **Reconcile or report the gap.**

★ **Row 3 is the deliverable.** The instrument alone answers nothing; the sweep is what names the op. Report
per-op `ns/calls` at each n, not just totals — an op called 4× as often with constant per-call time is linear
and uninteresting.

⚠ **Row 4 gates the axis I got wrong last time.** On the backlog stone I wrote *"the instrument costs no store
calls"* and a quadratic term walked straight through, because clone-cost is allocation, not syscalls. Here the
axis is **time**: wall clock and phase times against the known band.

★ **And this instrument should be genuinely free**, which row 4 can confirm rather than assume: the code
**already** brackets store calls with clock reads to compute `store-ns`. Splitting by op changes *attribution*,
not the number of clock reads. If wall clock moves, something more than attribution changed.

## Runtime prediction

**60–90 minutes.** Four counter pairs threaded through the State constructors (expect more sites than a sketch
names — the last two stones went 3→6 and 12→12), the report line, then ~7 minutes of sweep and ~8 of floor.

## Trap-doors

- **`ensure-schema` is not in the split.** Called once in `setup`, which is constant and out of scope. If its
  time is currently inside `store-ns`, say so — that is part of row 2's reconciliation.
- **Attribute at the call site**, not by inferring the op from a response type.
- **`count-index` is called twice per send** and `put` three times; a raw call count is not a per-call cost.
  Row 3 wants `ns/calls`.
- **The State constructors thread every `:ephemeral` field.** A dropped field is a type error; a *wrong* one
  (threading the pre-increment value) is not caught and silently under-counts.
- **Do not touch admission, visibility, the cap, or `wat/`.**

## What this stone does NOT claim

⚠ It does **not** fix anything, and makes **no prediction** about which op grows. My two hypotheses —
`mem.wat`'s clones, and a missing SQL index — were both refuted by the disk before this was drawn.
⚠ It does **not** touch `wat/query/sqlite-store.wat`. A server-side dive follows only if this names an op.
⚠ It does **not** revisit `setup`, which is constant and the builder's ruling.
