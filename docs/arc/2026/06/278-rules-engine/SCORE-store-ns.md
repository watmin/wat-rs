# SCORE — `store-ns`

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
Eight files. `store-ns` last on `Queue::StatsResponse::Ok` and on
`:ephemeral`. Clock pair around each of the eight `Store/*` calls.
Reported as `store-ms=` (nanos on the wire, divide by 1e6 at format).

```
Summary [ 498.323s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T01-07-44Z/`

## THE INSTRUMENT

`ns <- i64` fourth field on `:queue::TakeAcc` (`sqs.wat:95`).
`store-ns <- i64` on `:ephemeral` beside `store-calls`, init 0, last
on `StatsResponse::Ok`.

Clock pair `(epoch-nanos (now))` before and after the `Store/*` form
only, at all eight sites. Folds accumulate on `TakeAcc/ns`. Arms add
it onto `State/store-ns` the same way `calls` → `store-calls`.

`sum-store-ns` copies `sum-store-calls`. Phases: `store-ms={sms}`.

## THE NUMBER — reported, not acted on

pairs = n×m. fill-first. `store-ms` is whole-run (fill puts included),
so it **exceeds drain**. Expected.

| n | store-calls | store-ms | store-ms/pair | drain ms | drain/pair | pairs/sec |
|---|---|---|---|---|---|---|
| 500 | 2482 | **2868** | 1.434 | 508 | 0.254 | 3937 |
| 1000 | 4943 | **5863** | 1.466 | 1261 | 0.315 | 3172 |
| 2000 | 10171 | **13611** | 1.701 | 3362 | 0.420 | 2380 |

store-ms grows monotonically (2868 / 5863 / 13611). store-ms < total
at every n (21870 / 27142 / 38636).

store-ms/pair +18.6 % from n=500 to n=2000. drain/pair +65 %. Did not
name a side of the fork. STOP-5.

## ROW 1 — `store-ms=` on phases

n=500 fill-first:

```
n=500;m=4;j=3;total=2000;distinct=2000;dup=0;workers=12;empty=1
seen-recorded=2000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill-depth=[500/0]×4  drain=508  poll-calls=95  store-calls=2482  store-ms=2868  pairs/sec=3937
```

## NO-ARGS

```
queue-receive-calls=4999
total=8000;distinct=8000;dup=0;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
poll-calls=15  publish-calls=200  store-calls=38189  store-ms=37725  drain=112  fill=22091
```

Identity field-for-field. `store-ms` additive. Concurrent counters
vary as before.

## n=2000

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=20;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill-depth=[2000/0]×4  drain=3362  poll-calls=640  store-calls=10171  store-ms=13611  pairs/sec=2380
```

`total=8000;distinct=8000;dup=0`. ⚠ `seen-skipped=20`. Did not re-run.

## CURVE

pairs/sec vs 3623 / 3050 / 2303 ±15 %: **+8.7 % / +4.0 % / +3.3 %**.

## FINDING — script-level Tuple is unknown in the child

`retry-put` / `retry-delete` returning `(Tuple n ns)` typechecked in
the parent and failed in `:impls`: `first`/`second` got `:?NNNN`.
Same EmptyEnv wall as script-level `TakeAcc`.

Fix: `:queue::RetryAcc` (`n`, `ns`) in `:messages` beside `TakeAcc`.
Two i64 fields, no `Reply`. n=12 then typed and ran.

## BLAST

```
 wat-scripts/fanout/circuit.wat                     |  26 +-
 wat-scripts/queue/sqs.wat                          | 417 ++++++++++++---------
 wat-scripts/scratch-pad/probe-*.wat (5 files)      |   2 +- each
 wat-scripts/topic/sns-fanout.wat                   |   4 +-
 8 files changed, 268 insertions(+), 191 deletions(-)
```

`git diff -- wat-scripts/topic wat-scripts/scratch-pad` is eight
trailing `_` and nothing else. `Topic::StatsResponse` stays `[n ticks]`.
No `wat/`.

16th `StatsResponse::Ok` is `sum-store-ns` (`circuit.wat:1862`) — the
instrument, shaped like `sum-store-calls`. Not a pre-existing site.
STOP-1 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `store-ms=` on phases | ✅ n=500 `store-ms=2868` |
| 2 | `ns` named field on `TakeAcc` | ✅ four fields; nested-Tuple awk still **0** |
| 3 | nested-Tuple constructions still zero | ✅ 0 |
| 4 | every counting site also times | ✅ 8 of 8 |
| 5 | bounded and sane | ✅ store-ms ≥ 0 and store-ms < total at every n |
| 6 | tracks the work | ✅ 2868 / 5863 / 13611 |
| 7 | overhead invisible | ✅ 3937 / 3172 / 2380, all inside ±15 % of 3623 / 3050 / 2303 |
| 8 | no-args unchanged | ✅ identity field-for-field; `store-ms` additive |
| 9 | n=2000 correctness | ✅ `total=8000;distinct=8000;dup=0` |
| 10 | eight ripple `_` only | ✅ 8 insertions, 8 deletions |
| 11 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 12 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 13 | the floor | ✅ `Summary [ 498.323s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 / STOP-2 / STOP-3 / STOP-4 / STOP-5 / STOP-6 / STOP-7 held.**
STOP-7's premise held: `ns` rides `TakeAcc`. The Tuple wall was on
the retry helpers, not on `TakeAcc`.

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## WHAT THIS DOES NOT CLAIM

It does not explain the slope. It does not time the store's internals.
`store-ns` is the round trip as the queue sees it: the store's work
plus the IPC.

---

# GRADING — claude, 2026-09-07

**STRUCK.** All thirteen rows verified on my own runs and reads. Floor mine:

```
Summary [ 478.097s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## My numbers, beside grok's

```
n      store-calls  store-ms   fill    drain    ms per store round trip
500       2472       2883      4107     531      1.166   (grok 1.155)
1000      4967       6019      8363    1278      1.212   (grok 1.186)
2000     10197      13822     17162    3302      1.355   (grok 1.338)
```

`total=8000; distinct=8000; dup=0` at n=2000. No-args identical field for field.

★ **Store round trips are not constant-cost — they slow ~16 % with depth.** Two independent runs
agree (+16.2 % / +15.8 %). We knew the *count* was flat; now we know the *per-call time* is not.
That is genuinely new.

## ⛔ AND THE INSTRUMENT CANNOT ANSWER THE FORK IT WAS DRAWN FOR — my error

`store-ns` is **cumulative over the whole run**. I wrote that in the DESIGN as a limitation and did
not notice it made the counter unable to resolve the question it exists for. **The fork was about
the drain.** Where the store calls actually happen:

```
n=500    fill  4107 ms   drain  531 ms   →  fill is 89 % of the working time
n=2000   fill 17162 ms   drain 3302 ms   →  fill is 84 %
```

★★★ So the +16.2 % is dominated by **fill** store calls and says almost nothing about the drain's
**+61 %** per-receive-call growth. **A stone that satisfies all thirteen of its own rows and misses
its purpose.**

⚠ **And a second reason not to over-read it:** `sum-store-ns` sums across **m=4 queues**, so it is
not commensurate with a single wall-clock timeline at all. Dividing `store-ms` by `drain` would be
the same category error this arc has already paid for three times — I am not committing a fourth.

★ Neither branch of the fork is called. The DESIGN's promise — *"no prediction"* — is kept, but only
because the instrument cannot discriminate, which is not the reason I intended.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=500 run → `store-ms=2883` | ✅ |
| 2 | `ns` is the 4th named field, `sqs.wat:96-100` | ✅ |
| 3 | nested-Tuple constructions → **0** | ✅ the last stone's gain is not given back |
| 4 | 18 `epoch-nanos` in sqs.wat; 8 of 8 sites timed | ✅ |
| 5 | `store-ms` < `total` at every n (21620 / 27671 / 38835) | ✅ |
| 6 | 2883 / 6019 / 13822 | ✅ monotonic |
| 7 | my curve 3767 / 3130 / 2423 vs 3623 / 3050 / 2303 | ✅ inside ±15 % |
| 8 | my own no-args run, field for field | ✅ |
| 9 | my own n=2000 | ✅ |
| 10 | `git diff -- topic scratch-pad` → **8 insertions, 8 deletions** | ✅ boring |
| 11 | my own floor, 38 drop tests | ✅ |
| 12 | `every_wat_scripts_file_loads` PASS in my floor | ✅ |
| 13 | my own run, Summary line read | ✅ |

**STOP-1 through STOP-7 held.** STOP-5 in particular: grok reported the number and **did not name a
side of the fork**, which was right — the instrument cannot support either.

★★ **And grok hit the same EmptyEnv wall a second time, independently.** `retry-put`/`retry-delete`
returning a script-level `(Tuple n ns)` typechecked in the parent and failed in `:impls` with
`:?NNNN`. The fix was `:queue::RetryAcc` in `:messages` (`sqs.wat:103`) — the previous stone's
lesson applied without being briefed. That is the pattern generalising, not a repeat of the failure.

## THE FIX — small, circuit-local, and it is the next stone

**Sample `store-ns` at the phase boundaries.** `sum-store-ns` already exists; call it at `t-drain0`
and again at `t-collect0` and report the delta as `drain-store-ms=`. No `sqs.wat` change, no ripple,
no new field — and it answers the fork this counter only gestures at.

⚠ It must also report the **per-queue** figure, or divide by `m`, so the summed-across-actors number
is never set beside a single timeline again.

## OPEN

1. **`drain-store-ms`** — the phase-boundary sample above. Next.
2. **`circuit.wat`** — 36 nested-Tuple access chains, 33 nested constructions, the 268-line `-tick`
   arm. Its own stone; `sqs.wat` is now the worked example for it.
3. **`seen-skipped` at n=2000** — 20 (grok) this stone, 60/110 last, 60/100 before. Four stones of
   anecdote with `dup=0` throughout. It wants a real sample.
4. **The slope** — still unexplained, and now known to be **only partly** store-side.
