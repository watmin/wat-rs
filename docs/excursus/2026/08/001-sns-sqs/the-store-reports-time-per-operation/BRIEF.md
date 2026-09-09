# BRIEF — the store reports time per operation

## The work, in one paragraph

The queue times its store calls in aggregate — one `store-calls`, one `store-ns` — across **five distinct
operations**. The last measurement showed store calls growing linearly while store *time* grows
superlinearly, so per-call latency rises with row count, and nothing says which operation is responsible.
Split the counters by op, report them per tier, then run the same three-point sweep and report per-op mean
latency at each n.

## Why this measurement

`1b2c65034` established `drain` as the only superlinear phase (n^1.23) and handed over one candidate:

```
drain store-calls   2.024, 2.037    LINEAR
drain store-ms      2.513, 2.616    SUPERLINEAR
mean per-call       1.79 → 2.22 → 2.85 ms
```

**Two hypotheses of mine are already dead**, and you should not spend time on either: it is **not**
`wat/query/mem.wat` (the drain's queues are `sqlite-store`, `circuit.wat:2142` and `:2160`), and it is
**not** a missing SQL index (`sqlite-store.wat:211` has `PRIMARY KEY(ipk, isk, pk, sk)` — index-usable and
covering for `SELECT 1`). Measure; do not theorise.

## Read in order

1. **`wat-scripts/queue/sqs.wat:108`, `:147-148`, `:385-386`** — the existing `store-calls` / `store-ns` on
   `:ephemeral` and their initialisation. **Your four pairs go beside them, on `:ephemeral`, so
   `:queue::queue::Record` stays out of the diff.**
2. **`wat-scripts/queue/sqs.wat`** — search `:wat::query::Store/`. Five ops: `put` ×3, `delete` ×2,
   `count-index` ×2, `scan-index` ×1, `ensure-schema` ×1. **Each already sits inside timing brackets that
   feed `store-ns`** — you are changing attribution, not adding clock reads.
3. **`wat-scripts/queue/sqs.wat`** — search `seen-ids` for the worked precedent of threading a new
   `:ephemeral` field through every State constructor (landed `0e026d2de`).
4. **`wat-scripts/fanout/circuit.wat`** — the per-tier report line added by `96a840db0`. It gains the split.
5. **`docs/excursus/2026/08/001-sns-sqs/the-slope-belongs-to-a-phase/SCORE.md`** — the sweep's shape, its
   medians-with-spread discipline, and the n=2000 baseline band.

## Implementation sketch

```wat
;; :ephemeral gains four pairs, beside store-calls / store-ns
put-calls <- :wat::core::i64     put-ns    <- :wat::core::i64
delete-calls <- :wat::core::i64  delete-ns <- :wat::core::i64
count-calls <- :wat::core::i64   count-ns  <- :wat::core::i64
scan-calls <- :wat::core::i64    scan-ns   <- :wat::core::i64

;; at each existing call site, the SAME elapsed value already computed for
;; store-ns is additionally attributed to its op's counter.
```

Then the sweep:

```
./target/release/wat wat-scripts/fanout/circuit.wat <n> 4 3 8192 true 1000
   n ∈ {1000, 2000, 4000},  ≥3 runs each,  one at a time, box quiet
```

Report per op per n: `calls`, `ns`, and **`ns/calls`** — plus the ratios `n2000/n1000` and `n4000/n2000`
with their observed spread.

## Blast radius

`wat-scripts/queue/sqs.wat` and `wat-scripts/fanout/circuit.wat`. **No `wat/`. No `src/`.**

Expect more edit sites than the sketch shows — every `:ephemeral` field threads through each State
constructor. The last two stones ran 3→6 and 12→12; report the real count.

## STOP triggers

**STOP-1** — if the four pairs do not reconcile with `store-ns`, **STOP and report the gap** with both
numbers. An unaccounted remainder names an operation the DESIGN missed, and that is worth more than the
split. This arc has had a split fail exactly this way before.

**STOP-2** — if a counter increment needs a store call, a clock read that did not already exist, or any new
state beyond the eight fields, **STOP and say what.** The timing brackets already exist; you are changing
attribution.

**STOP-3** — do **not** touch `wat/query/sqlite-store.wat`, `wat/query/mem.wat`, or anything under `wat/`.
Stdlib, the builder's call, and this stone is a measurement.

**STOP-4** — do **not** change admission, visibility, ack semantics, the cap, or `setup`.

**STOP-5** — do **not** report a ratio without its spread, and do **not** name a culprit the numbers do not
support. *"All four ops have flat per-call latency and the growth is elsewhere"* is a complete finding.

**STOP-6** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

A per-tier line carrying the four pairs beside the aggregate, reconciling within rounding. A sweep table of
≥9 runs with per-op `ns/calls` at n=1000/2000/4000 and the ratios with spreads. `distinct = n×m` and `dup=0`
on every run. Wall clock and phases inside the known band (row 4). Load stated per run. Floor Summary read
after the sweep: 5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

**Write the SCORE to
`SCORE.md`** in the shape of the
neighbouring SCORE files. **Do not commit.**

State plainly **which operation's per-call latency grows**, with its ratios — or that none does.
