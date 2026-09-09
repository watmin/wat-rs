# SCORE — the drain reports its own store time

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. Four samples of the existing
`sum-store-calls` / `sum-store-ns`; two fields on the phases line,
both divided by `m`. Did not act on the number.

```
Summary [ 494.072s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T01-36-35Z/`

## SAMPLE ORDER — timestamps were not moved

```
t-drain0
sc-before / ns-before     ;; immediately after t-drain0
poll-until-drained
sc-after  / ns-after      ;; immediately before t-collect0
t-collect0
```

STOP-1 did not fire. Both samples sit inside the wall-clock drain.
The after-sample's 8 stats calls land in the store delta; the
before-sample's 8 do not (they are in `sc-before`). Constant offset,
reported, not subtracted.

```
drain-store-ms=    (ns-after − ns-before) / 1e6 / m
drain-store-calls= (calls-after − calls-before) / m
```

## THE NUMBER — reported, not acted on

pairs = n×m. fill-first. Both fields are **per queue**.

| n | drain-store-calls | drain-store-ms | /pair | drain ms | drain/pair | store-ms | pairs/sec |
|---|---|---|---|---|---|---|---|
| 500 | 185 | **281** | 0.141 | 519 | 0.260 | 2648 | 3854 |
| 1000 | 393 | **673** | 0.168 | 1314 | 0.329 | 5961 | 3044 |
| 2000 | 883 | **1668** | 0.209 | 3462 | 0.433 | 13844 | 2311 |

drain-store-ms ≪ store-ms at every n, and ≤ drain. ≥ 0. Monotonic.

drain-store-ms/pair +48 % from n=500 to n=2000. drain/pair +67 %.
Did not name a side of the fork. STOP-5.

## POLLER SHARE — row 5, not netted out

`(poll-calls / (m+1)) × m × 2` vs `drain-store-calls × m`:

| n | poll-calls | poller calls | drain-store-calls×m | share |
|---|---|---|---|---|
| 500 | 80 | 128 | 740 | 17 % |
| 1000 | 225 | 360 | 1572 | 23 % |
| 2000 | 650 | 1040 | 3532 | 29 % |

A fraction, not larger. Sampling is not inverted.

## ROW 1 — both fields on phases

n=500 fill-first:

```
n=500;m=4;j=3;total=2000;distinct=2000;dup=0;workers=12;empty=1
seen-recorded=2000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill-depth=[500/0]×4  drain=519  poll-calls=80  store-calls=2478  store-ms=2648
drain-store-calls=185  drain-store-ms=281  pairs/sec=3854
```

## NO-ARGS

```
queue-receive-calls=5028
total=8000;distinct=8000;dup=0;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
poll-calls=10  publish-calls=200  store-calls=37950  store-ms=37408
drain-store-calls=50  drain-store-ms=36  drain=226
```

Identity field-for-field. Both new fields additive. Concurrent
counters vary as before.

## n=2000

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=20;check-exhausted=0;mark-exhausted=0;ack-retries=1
fill-depth=[2000/0]×4  drain=3462  poll-calls=650  store-calls=10203  store-ms=13844
drain-store-calls=883  drain-store-ms=1668  pairs/sec=2311
```

`total=8000;distinct=8000;dup=0`. ⚠ `seen-skipped=20`. Did not re-run.

## CURVE

pairs/sec vs 3767 / 3130 / 2423 ±15 %: **+2.3 % / −2.7 % / −4.6 %**.

## BLAST

```
 wat-scripts/fanout/circuit.wat | 12 +++++++++++-
 1 file changed, 11 insertions(+), 1 deletion(-)
```

No `sqs.wat`. No `wat/`. No `StatsResponse` change. No ripple.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ both fields reported | ✅ n=500 `drain-store-calls=185` `drain-store-ms=281` |
| 2 | the delta is per queue | ✅ `/ m` at the format |
| 3 | a drain measurement | ✅ drain-store-ms ≪ store-ms and ≤ drain at every n |
| 4 | bounded, monotonic | ✅ 281 / 673 / 1668 |
| 5 | poller is a fraction | ✅ 17 % / 23 % / 29 % of drain-store-calls×m |
| 6 | no `sqs.wat` change | ✅ `circuit.wat` only |
| 7 | no-args unchanged | ✅ identity field-for-field; both fields additive |
| 8 | n=2000 correctness | ✅ `total=8000;distinct=8000;dup=0` |
| 9 | overhead invisible | ✅ 3854 / 3044 / 2311, all inside ±15 % |
| 10 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 11 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 12 | the floor | ✅ `Summary [ 494.072s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 / STOP-2 / STOP-3 / STOP-4 / STOP-5 / STOP-6 held.**

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## WHAT THIS DOES NOT CLAIM

It does not explain the slope. It measures the drain instead of the
run. It does not net out the poller. It does not time the store's
internals.

---

# GRADING — claude, 2026-09-07

**STRUCK.** All twelve rows verified on my own runs and reads. Floor mine:

```
Summary [ 479.232s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## My numbers, beside grok's

```
n     drain   drain-store-ms   drain-store-calls   poll-calls    (grok: ms / drain)
500     532        279               186              85          281 /  519
1000   1245        638               395             230          673 / 1314
2000   3308       1567               880             650         1668 / 3462
```

Agreeing within 6 %. `distinct` / `dup` correct at every depth; no-args identical field for field.

## THE FORK'S NUMBER

```
n      drain/pair   store/pair   NON-store/pair   store share of drain
500      0.266        0.1395        0.1265             52.4 %
1000     0.311        0.1595        0.1515             51.3 %
2000     0.4135       0.1959        0.2176             47.4 %
growth   +55.5 %      +40.4 %       +72.0 %            FALLING
```

## ⛔ AND MY FIRST READING OF IT WAS WRONG

I reported this as *"the store is largely exonerated."* **Too strong**, and the error is structural,
not numerical.

★★★ **The queue is a serializing actor**, and it is blocked inside store calls for **47–52 % of the
drain**. During that time it can serve nobody — so every worker waiting to `receive` or `ack` is
queued behind it, and **that waiting is counted in the "non-store" 72 % while its cause is the
store.** A saturating serializer converts its own service time into everyone else's delay.

⚠ So **store-vs-non-store was the wrong axis**, and the fork I wrote had a false dichotomy inside
it: branch two ("the store is exonerated") does not follow from a falling store *share*, because
the share is not the causal path.

★ **Fourth time in this arc that a fork of mine named or implied a winner it had not earned** — the
poller stone's "cost per op is rising", the `store-ns` stone's whole-run span, and now this. Each
time the numbers were sound and the *inference* was not.

## What survives, stated carefully

- **Measured:** store time per pair grows +40 %, drain per pair +55 %, non-store per pair +72 %, and
  the store's direct share of the drain falls 52 % → 47 %.
- **Measured:** the drain poller's own store calls are **18 % / 23 % / 29.5 %** of drain store work
  and growing (`sqs.wat:976` — every `stats` costs 2). Derivable from `poll-calls`.
- **Bounded:** the queue is busy **at least** 47–52 % of the drain. Its own compute is on top.
- **Not established:** whether the non-store growth is induced queueing behind a saturated queue, or
  time genuinely spent outside the queue.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=500 → `drain-store-calls=186 drain-store-ms=279` | ✅ |
| 2 | `/ m` at the format | ✅ per queue |
| 3 | `drain-store-ms` ≪ `store-ms` and ≤ `drain` at every n | ✅ |
| 4 | 279 / 638 / 1567 | ✅ monotonic |
| 5 | poller share 18 % / 23 % / 29.5 % — a fraction, not larger | ✅ sampling is not inverted |
| 6 | `git diff --stat` → `circuit.wat` only, 11 insertions | ✅ |
| 7 | my own no-args run, field for field | ✅ |
| 8 | my own n=2000 → `total=8000;distinct=8000;dup=0` | ✅ |
| 9 | my curve 3759 / 3212 / 2418 vs 3767 / 3130 / 2423 | ✅ inside ±15 % |
| 10 | my own floor, 38 drop tests | ✅ |
| 11 | `every_wat_scripts_file_loads` PASS in my floor | ✅ |
| 12 | my own run, Summary line read | ✅ |

**STOP-1 through STOP-6 held.** grok reported the number and named no side of the fork — correct,
and better discipline than my own grading showed.

## THE NEXT STONE — drawn from the correction

`docs/excursus/2026/08/001-sns-sqs/is-the-queue-saturated/DESIGN.md`. `handler-ns` on the queue, sampled at the drain boundaries,
reported per queue as `drain-busy-ms`. It answers the question this grading found:

```
busy / wall near 1     -> saturated; the non-store growth is INDUCED QUEUEING and the
                          store is the root cause, reached through the serializer
busy / wall well below -> the queue is idle much of the drain; the time is genuinely in
                          IPC or scheduling — where nobody has looked
```

★★ And it yields the three-way split the arc has been circling, all from counters that will then
exist: `busy = handler-ns`, `queue compute = handler-ns − store-ns`, `not-in-queue = drain − handler-ns`.

⚠ It is the **widest mechanical edit of the arc** — 30 `State` constructions, 18 `Outcome::Continue`
sites, plus the 16-site ripple. Uniform, compile-checked, and not small.
