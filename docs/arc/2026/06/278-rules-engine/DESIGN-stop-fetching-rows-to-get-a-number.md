# DESIGN — stop fetching rows to get a number

**The first perf stone.** `wat/query.wat` + both stores + `wat-scripts/queue/sqs.wat`.
Architectural, not micro: fewer round trips, and each one cheaper by construction.

## ⛔ THE STANDING RULE FOR THIS PHASE

**Every perf stone reports the NEW DOMINANT TERM.** Not "we saved 17 s" — *"the largest remaining
contributor is now X, measured."* The arc's terminal condition is **wat's interpretation overhead
becoming the dominant term**; without naming the leader each round we cannot tell we have arrived,
and we optimize past the point of return.

## WHY — measured, not suspected

`publish` went **26.6 s → 50.1 s** across four correctness stones. The circuit's own stage
histograms localize it exactly:

```
THEN  outbox  50-250=7944    max=260ms      t1->t2 <1ms=8000   t2->t3 <1ms=8000
NOW   outbox  250-1000=7739  max=5693ms     t1->t2 <1ms=8000   t2->t3 <1ms=8000
```

`outbox` is `t0` (publish, `circuit.wat:996`) → `t1` (topic worker pickup, `sns-fanout.wat:383`).
**Every other stage is unchanged and sub-millisecond.** So it is queueing delay: the topic worker
drains slower, so rows wait longer.

`probe-what-a-scan-costs.wat`, against the same sqlite store at the same depth:

| scan | rows | µs each | ×16 000 |
|---|---|---|---|
| `limit 65` — what the cap gate issues | 60 | **1336** | **21.4 s** |
| `limit 1` | 1 | **448** | 7.2 s |

★ **The cap gate does two scans on every send** (`sqs.wat:293`). 8000 sends × 2 = **16 000 scans
× 1.33 ms = 21.4 s** — essentially the whole 23.5 s regression, arriving exactly when *depth is
read, not counted* landed.

★★ **This is not sqlite being slow. It is us asking wrong**: 16 000 round trips that each
materialize 60 rows and sum them into one integer. Row materialization is **888 µs of the
1336 µs — two thirds of the cost is building rows we discard.**

## ⛔ THE ONE CONTRACT DECISION

**A depth question is answered by counting, and the send path asks only what it uses.**

Two changes, composing on the same path:

1. **One scan, not two.** `visible + unacked` **is** `total`, the `[0, +inf)` scan alone. Three of
   the four `depth` call sites (`sqs.wat:294`, `:472`, `:502` — all on the **send** path)
   immediately sum the pair and discard the split. Only `stats` (`:741`) consumes it. Give the
   send path a `total` that does **one** scan. → **16 000 → 8 000 calls.**
2. **Count, not fetch.** A new `count-index` verb on `:wat::query::Store`, implemented in both
   stores, returning a count and **no rows**. → **1336 → ~435 µs.**

Together: **21.4 s → ~3.5 s.**

★ The depth stone explicitly rejected a count verb as out of scope, and that was right *then* —
it would have confounded a correctness stone. **It is in scope now, and it is worth two thirds.**
It also pays off at every depth read, not just the cap gate: scaffolding we build on.

## ⚠ WHAT THIS PROJECTION IS AND IS NOT

21.4 → 3.5 s is arithmetic from a **microbenchmark on a quiet box**. The circuit runs
concurrently, so the saving will not be linear. **The stone must prove it end to end; my
arithmetic is a hypothesis, not a result.**

## FILES

`wat/query.wat` (the surface), `wat/query/mem.wat`, `wat/query/sqlite-store.wat`,
`wat-scripts/queue/sqs.wat`.

## OUT OF SCOPE = REJECTED

- **Compiling wat.** The builder's ruling: chase architectural perf until interpretation overhead
  is the dominant term, then stop. Not this arc.
- **The `Record` rebuild on every `receive`/`ack`** and **select+timer per client call** — both
  located and measured, both their own stones. Keeping them separate keeps attribution clean.
- **Caching depth.** That is the counter we deleted, and it could not be correct.
