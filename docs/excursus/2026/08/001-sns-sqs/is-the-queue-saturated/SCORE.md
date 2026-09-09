# SCORE — is the queue saturated?

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`handler-ns` last on `StatsResponse::Ok` and on `:ephemeral`. Drain
delta reported as `drain-busy-ms=` per queue. Did not act on ρ.

```
Summary [ 498.624s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T02-19-56Z/`

## THE INSTRUMENT

`handler-ns <- i64` on `:ephemeral`, init 0, last on `Ok` after
`store-ns`. Each impl arm binds `start-ns` from
`Invocation/start-ns` (`SelfInvocation/start-ns` on `-tick`).
Every `queue::State` reconstruction except init and the two helper
handoffs adds `(epoch-nanos (now)) − start-ns` onto the incoming
`s`'s `handler-ns`.

The two Transient handoffs (`s-r` into `send-after-put` /
`ack-after-delete`) **copy** `handler-ns` so the helper can add the
full elapsed. Those helpers have no `ctx`; `start-ns` is threaded
in. STOP-2 did not fire — the framework timestamp stays the
definition of handler start.

`sum-handler-ns` copies `sum-store-ns`. Sampled at the same
`t-drain0` / `t-collect0` boundaries as `drain-store-ms`, divided
by `m`.

## THE NUMBER — reported, not acted on

pairs = n×m. fill-first. All drain-* fields are **per queue**.

| n | drain-store-ms | drain-busy-ms | drain | busy/wall | queue compute | not-in-queue | pairs/sec |
|---|---|---|---|---|---|---|---|
| 500 | 290 | **321** | 561 | 0.57 | 31 | 240 | 3565 |
| 1000 | 671 | **742** | 1365 | 0.54 | 71 | 623 | 2930 |
| 2000 | 1659 | **1829** | 3471 | 0.53 | 170 | 1642 | 2305 |

`drain-busy-ms` ≥ `drain-store-ms` at every n (STOP-8 held).
`drain-busy-ms` ≤ `drain`. Monotonic 321 / 742 / 1829.

queue compute = busy − store. not-in-queue = drain − busy.
Did not name a side of the fork. STOP-6.

## ROW 1 — `drain-busy-ms=` on phases

n=500 fill-first:

```
n=500;m=4;j=3;total=2000;distinct=2000;dup=0;workers=12;empty=1
seen-recorded=2000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
drain=561  poll-calls=85  store-calls=2518  store-ms=2786
drain-store-calls=188  drain-store-ms=290  drain-busy-ms=321  pairs/sec=3565
```

## NO-ARGS

```
total=8000;distinct=8000;dup=0;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
publish-calls=200  drain-store-ms=27  drain-busy-ms=34
```

Identity field-for-field. `drain-busy-ms` additive.

## n=2000

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
drain=3471  poll-calls=630  drain-store-ms=1659  drain-busy-ms=1829  pairs/sec=2305
```

`total=8000;distinct=8000;dup=0`.

## CURVE

pairs/sec vs 3759 / 3212 / 2418 ±15 %: **−5.2 % / −8.8 % / −4.7 %**.

## BLAST

```
 wat-scripts/fanout/circuit.wat                     | 29 +++++--
 wat-scripts/queue/sqs.wat                          | 95 ++++++++++++----------
 wat-scripts/scratch-pad/probe-*.wat (5 files)      |   2 +- each
 wat-scripts/topic/sns-fanout.wat                   |   4 +-
 8 files changed, 84 insertions(+), 56 deletions(-)
```

`git diff -- wat-scripts/topic wat-scripts/scratch-pad` is eight
trailing `_`. `Topic::StatsResponse` stays `[n ticks]`. No `wat/`.

17th `StatsResponse::Ok` is `sum-handler-ns` — the instrument.
STOP-1 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `drain-busy-ms=` / m | ✅ n=500 `drain-busy-ms=321` |
| 2 | busy includes store | ✅ 321≥290, 742≥671, 1829≥1659 |
| 3 | busy ≤ drain | ✅ 321≤561, 742≤1365, 1829≤3471 |
| 4 | monotonic | ✅ 321 / 742 / 1829 |
| 5 | every arm accumulates | ✅ 27 ADD, 2 COPY (Transient handoffs), init 0. Helpers take `start-ns`. |
| 6 | overhead invisible | ✅ 3565 / 2930 / 2305, all inside ±15 % |
| 7 | no-args unchanged | ✅ identity field-for-field; `drain-busy-ms` additive |
| 8 | n=2000 correctness | ✅ `total=8000;distinct=8000;dup=0` |
| 9 | eight ripple `_` only | ✅ 8 insertions, 8 deletions |
| 10 | nested-Tuple still zero | ✅ awk = 0 |
| 11 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 12 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 13 | the floor | ✅ `Summary [ 498.624s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 through STOP-8 held.**

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## WHAT THIS DOES NOT CLAIM

It does not explain the slope. It does not make anything faster.
It yields the three-way split. The ratio is the output, not a
finding this stone is allowed to name.

---

# GRADING — claude, 2026-09-07

**STRUCK.** All thirteen rows verified on my own runs and reads. Floor mine:

```
Summary [ 477.091s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## ⛔ THE MODEL I DREW IS REFUTED

My runs, beside grok's:

```
n     drain   busy      ρ      store   queue-compute   not-in-queue
500     516    296   0.574      266         30             220
1000   1315    710   0.540      643         67             605
2000   3405   1758   0.516     1594        164            1647

grok:                 0.57 / 0.54 / 0.53
```

★★★★★ **ρ falls: 0.574 → 0.516.** The queue gets **less** saturated as depth grows. `W ∝ ρ/(1−ρ)`
required ρ to rise; it does the opposite. **The single-server amplification model is dead** — ninth
mechanism killed in this arc, and this one was mine, drawn one stone earlier.

★ The DESIGN said it plainly before the strike — *"that is arithmetic that fits, not evidence"* —
and the row set carried no prediction. That is the only reason this cost one strike instead of a
detour: **the stone was built to kill the model, not to confirm it.**

## What the split actually shows

```
per pair       drain    busy    queue-compute    NOT-in-queue
n=500          0.258   0.148       0.015            0.110
n=1000         0.329   0.1775      0.0168           0.151
n=2000         0.426   0.2198      0.0205           0.206
growth          +65 %   +48 %       +37 %            +87 %
```

★★★ **Everything dilates together while ρ stays flat.** The largest and fastest-growing term is
time when the queue is **idle** — so the workers are waiting somewhere the queue cannot see.

⚠ And `queue compute` growing +37 % is itself odd: the queue does the same work per op at any depth,
and nothing in its state (`waiters` bounded by workers, `outbox` drained per tick) scales with depth.

## THE TWO CANDIDATES — neither chosen by argument

**1. `Seen` contention plus a growing map.** `circuit.wat:2210` starts **one** `:fanout::seen`
service, granted to **all 12 workers** — versus 4 queues with 3 clients each, so **3× the
contention**. Every message costs it a `check` **and** a `mark`. And its state grows without bound:

```wat
:ephemeral [claimed <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])]
…
(:wat::hashmap::assoc claimed key true)        ;; circuit.wat:178, inside a fold
```

`claimed` reaches **n×m = 8000 entries**. It is the only structure on the hot path whose size tracks
messages processed, and waiting on it is *exactly* "not-in-queue" from the queue's viewpoint.

**2. Uniform dilation.** Everything grew together (+37 / +48 / +65 / +87 %) with ρ flat — also the
signature of allocator or cache pressure as the process heap grows.

★ Candidate 1 is cheap to test and **much smaller than this stone**: `Seen` lives in `circuit.wat`,
so there is **no `Queue::StatsResponse` ripple** — no 16-site blast, no `sqs.wat` edit. The same
`handler-ns` shape on a service with 12 clients instead of 3.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=500 → `drain-busy-ms=296` | ✅ per queue |
| 2 | 296≥266, 710≥643, 1758≥1594 | ✅ busy includes store |
| 3 | 296≤516, 710≤1315, 1758≤3405 | ✅ |
| 4 | 296 / 710 / 1758 | ✅ monotonic |
| 5 | grok: 27 ADD, 2 COPY at the Transient handoffs, init 0 | ✅ and the copies are correct — the helper adds the full elapsed |
| 6 | my curve 3876 / 3042 / 2350 vs 3759 / 3212 / 2418 | ✅ inside ±15 % |
| 7 | my own no-args run | ✅ identity; `drain-busy-ms` additive |
| 8 | my own n=2000 | ✅ `total=8000;distinct=8000;dup=0` |
| 9 | 8 insertions, 8 deletions in the ripple files | ✅ |
| 10 | nested-Tuple constructions in sqs.wat → **0** | ✅ not given back |
| 11 | my own floor, 38 drop tests | ✅ |
| 12 | `every_wat_scripts_file_loads` PASS | ✅ |
| 13 | my own run, Summary line read | ✅ |

**STOP-1 through STOP-8 held.** STOP-6 in particular: grok reported ρ and named no side of the fork.

★★ **The widest edit of the arc landed clean** — 30 `State` sites, 16-site ripple, 8 files, floor
green first time. `TakeAcc`'s named fields are why; before them this shape killed two consecutive
strikes.

## OPEN

1. **`Seen`** — the candidate above. Next.
2. **The uniform-dilation alternative** — if `Seen` comes back flat, this is what remains, and it is
   a different kind of investigation (runtime, not topology).
3. **`circuit.wat`** — 36 nested-Tuple chains, the 268-line `-tick` arm. `sqs.wat` is now the worked
   example.
4. **The slope** — still unexplained after nine dead mechanisms, but the search space is much
   smaller: not the count, not contention at the store, not the poller, not queue saturation.
