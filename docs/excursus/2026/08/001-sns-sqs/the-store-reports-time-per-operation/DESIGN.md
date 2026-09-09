# DESIGN — the store reports time per operation

**Instrument plus measurement, one stone.** Names which store operation's per-call latency grows with row
count — the mechanism candidate `the slope belongs to a phase` handed over.

## What the measurement established

`1b2c65034`, three points × ≥3 runs, `vis` pinned, harness de-contaminated:

```
drain            n2000/n1000 = 2.301    n4000/n2000 = 2.388     → n^1.23, the ONLY superlinear phase
drain store-calls            2.024                  2.037       → LINEAR
drain store-ms               2.513                  2.616       → SUPERLINEAR
mean per-call     1.79 ms  →  2.22 ms  →  2.85 ms
```

**The store is called a linear number of times and takes a superlinear amount of time per call.** That is
measured, not inferred, and it is the first grounded mechanism candidate in this arc after eleven that died.

## ⛔ What the disk then refuted — my own hypothesis, twice over

**"It's `mem.wat`'s O(N²) index clones."** No. `circuit.wat:2142` and `:2160` start **`sqlite-store`** for
the inbox and every subscriber queue. The tracker's long-carried `wat/query/mem.wat` item is **irrelevant to
this finding**, and I was one step from pointing at it.

**"The index tables have no real index, so every scan is a full table scan."** No.
`wat/query/sqlite-store.wat:211`:

```sql
CREATE TABLE IF NOT EXISTS [index_{name}] (ipk, isk, pk, sk, data, PRIMARY KEY(ipk, isk, pk, sk))
```

`ipk` leads with equality and `isk` follows with a range — index-usable, and `SELECT 1` makes it covering.
`count-index` is additionally bounded by `LIMIT cap+1` = 65. **The hypothesis is dead on the schema.**

★★ So I do not know which operation grows, and this stone exists because **the measurement is the answer,
not my reasoning about it.** Nine mechanism claims of mine have been refuted today; four of them in the last
hour, each inside five greps. Guessing again would be the tenth.

## The gap in the current instrument

The queue tracks `store-calls` and `store-ns` as **aggregates only** (`sqs.wat:108`, `:147-148`). The drain
calls five distinct operations:

| op | calls in the queue |
|---|---|
| `put` | 3 |
| `delete` | 2 |
| `count-index` | 2 |
| `scan-index` | 1 |
| `ensure-schema` | 1 |

Nothing distinguishes a growing index seek from a growing write. **A single number over five operations is
the same collapse this arc has now repaired three times** — three signal sources behind one `Accepted 0`,
four read outcomes behind two silent exits, three signals behind one wake pipe.

## What lands

Per-operation `calls` and `ns` counters on the queue's **`:ephemeral`** — caller-side, so the store is
untouched and no `Record` changes:

```
put-calls / put-ns          delete-calls / delete-ns
count-calls / count-ns      scan-calls / scan-ns
```

Reported per tier beside the existing aggregate. Then **the same three-point sweep** — n=1000/2000/4000,
`vis-ms=1000`, ≥3 runs — reporting **per-op mean latency at each n**. The op whose per-call time grows is the
answer.

★ Caller-side, not server-side, deliberately: it answers *which op* without touching `wat/` (the builder's
call) and without a second service's blast radius. If the answer is a specific op, a server-side dive into
that op follows with a much narrower target.

## The one contract decision

**The split must reconcile with the existing aggregate.** `put-ns + delete-ns + count-ns + scan-ns` equals
`store-ns` within rounding, and the calls likewise. If it does not, an operation is unaccounted for — and
that gap is worth more than the split.

## OUT OF SCOPE — REJECTED

- **Touching `wat/query/sqlite-store.wat` or `mem.wat`.** Stdlib, the builder's call, and this stone is
  measurement.
- **Fixing whatever the split implicates.** Next stone; it needs this first.
- **`ensure-schema`.** Called once at startup, in `setup`, which is constant and out of scope.
- **A server-side per-query instrument.** Follows only if the caller-side split names an op.

## Files

`wat-scripts/queue/sqs.wat` (the counters) and `wat-scripts/fanout/circuit.wat` (the per-tier report line).
