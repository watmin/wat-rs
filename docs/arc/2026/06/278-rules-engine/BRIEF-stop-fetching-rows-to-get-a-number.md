# BRIEF — stop fetching rows to get a number

Halve the send path's store round trips, and make each one return a count instead of rows.

Read `DESIGN-stop-fetching-rows-to-get-a-number.md` first — including the standing rule that a
perf stone reports the **new dominant term**.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/queue/sqs.wat:213-245` | the `depth` closure — two range scans, `limit = cap+1`. Copy its shape for `total` |
| `sqs.wat:293-294` | **the cap gate.** `depth` then `(+ first second)` — it wants `total` and buys a split |
| `sqs.wat:472`, `:502` | the two `Full` responses — same sum, same fix |
| `sqs.wat:741` | `stats` — **the only site that consumes the split.** Leave it on `depth` |
| `wat/query.wat:556-570` | `ScanIndexRequest` / `ScanIndexResponse` — the sibling shape `count-index` copies |
| `wat/query/sqlite-store.wat:368` | `scan-index`'s impl — a `COUNT(*)` variant returns no rows |
| `wat/query/mem.wat` | the mem-store's `scan-index` impl — the same verb, counted |
| `wat-scripts/scratch-pad/probe-what-a-scan-costs.wat` | the measurement harness. **Re-run it after the change** — it is the unit-cost proof |

## THE TWO CHANGES

1. **`total`** — a closure beside `depth`, one scan over `[0, +inf)`, returning `i64`. The three
   send-path sites call it; `stats` keeps `depth`.
2. **`count-index`** — a Store feature: `CountIndexRequest [index ipk isk-lo isk-hi]` →
   `CountIndexResponse :Ok [n <- i64]` (+ the usual `RequestTooLarge` / `RequestMalformed`).
   Implemented in **both** stores. `depth` and `total` both use it.

## BLAST RADIUS

`wat/query.wat`, `wat/query/mem.wat`, `wat/query/sqlite-store.wat`, `wat-scripts/queue/sqs.wat`.
**No `src/`, no codemod, no `circuit.wat`.**

⚠ `wat/` is frozen at build time — rebuild before every measurement.

## STOP TRIGGERS

- **STOP-1** — if adding a feature to `:wat::query::Store` forces an arm anywhere outside the two
  store impls (an exhaustive match on `Store::Op`/`Reply`), **STOP and report every site** before
  editing. The DESIGN assumes `defservice` generates the dispatch.
- **STOP-2** — if `count-index` cannot return **without materializing rows** in either store,
  STOP and report which. A "count" that fetches rows and counts them in wat buys nothing.
- **STOP-3** — if `total` and `depth` ever disagree (`total ≠ visible + unacked`), STOP. They read
  the same index; a divergence means one of the ranges is wrong.
- **STOP-4** — no `src/` changes, no compiled-wat work, and do not touch the `Record` rebuild or
  the select+timer overhead. Both are located, both are later stones, and mixing them destroys
  attribution.

## ⛔ MEASURE IN TWO STEPS, NOT ONE

EXPECTATIONS row 5 needs `publish` **after change 1 alone**, then **after change 2**. Two
measurements, one strike — that is how both fixes get attributed without a second round trip.

## PRIOR RESULT TO COPY

`SCORE-depth-is-read-not-counted.md` — the stone that introduced this cost, and my grading on it
that found the double scan.
