# SCORE — the queue counts its store round trips

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
Eight files. `store-calls` last on `Queue::StatsResponse::Ok`.
Incremented once per store round trip. Surfaced on the phases line.

```
Summary [ 494.007s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T12-52-29Z/`

## THE COUNTER

`store-calls <- i64` on the queue `:ephemeral`, init 0, last field of
`StatsResponse::Ok`. Threaded through every `State` reconstruction that
carries `receive-calls`.

The eight `Store/*` sites do not hold `State` (`take` / `depth` /
`total` are init-local fns; `retry-put` / `retry-delete` are top-level).
The increment is applied at the reconstruction that follows each round
trip, **one per call, not per row**:

| site | what | counted as |
|---|---|---|
| `:168` scan-index | `take` | 1 if envs empty, else + the put |
| `:204` put | `take` re-hide | 2 with the scan |
| `:256` count-index | `depth` (twice) | +2 on stats |
| `:290` count-index | `total` | +1 on every send, including Accepted 0 |
| `:384` put | send | +1 |
| `:729` delete | ack | +1 |
| `:990` put | retry-put | returns 1 or 2; added on Transient |
| `:1039` delete | retry-delete | returns 1 or 2; added on Transient |

`ensure-schema` at `:150` is not incremented. Lost-on-put inside `take`
returns empty envs, so it would count 1 not 2 — chaos path, named.

`sum-store-calls` copies `sum-calls`. Phases: `store-calls={sc}`.

## THE NUMBER

Subscriber queues only (the circuit sum). pairs = n×m.

| n | receive-calls | store-calls | per pair | drain ms | pairs/sec |
|---|---|---|---|---|---|
| 500 | 212 | **2468** | 1.234 | 507 | 3945 |
| 1000 | 412 | **4973** | 1.243 | 1278 | 3130 |
| 2000 | 839 | **10174** | 1.272 | 3358 | 2382 |

store-calls **>** receive-calls at every n. Raw count **doubles** as n
doubles (×2.015, then ×2.046). Per pair is **1.23–1.27**.

That is the fork output. Not a gate.

## ROW 1 — `store-calls=` on phases

n=500 fill-first:

```
n=500;m=4;j=3;total=2000;distinct=2000;dup=0;workers=12;empty=1
seen-recorded=2000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill-depth=[500/0]×4  drain=507  poll-calls=90  store-calls=2468  pairs/sec=3945
```

## ROW 6 / FINDING — n=2000

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=60;check-exhausted=0;mark-exhausted=0;ack-retries=1
fill-depth=[2000/0]×4  drain=3358  poll-calls=635  store-calls=10174  pairs/sec=2382
```

`total=8000;distinct=8000;dup=0`. ⚠ `seen-skipped=60` (was 30 on the
previous stone). Did not re-run.

## NO-ARGS

```
total=8000;distinct=8000;dup=0;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
store-calls=38083
```

Pre-existing summary fields identical. `store-calls=` additive. fill-depth overlapped.

## CURVE

pairs/sec = (n×m)/drain_s. Band ±15 % of 3922 / 3160.

| n | drain ms | pairs/sec | vs band |
|---|---|---|---|
| 500 | 507 | 3945 | +0.6 % of 3922 |
| 1000 | 1278 | 3130 | −0.9 % of 3160 |

## BLAST

```
 wat-scripts/fanout/circuit.wat                     |  24 +++-
 wat-scripts/queue/sqs.wat                          | 132 +++++++++++++++------
 wat-scripts/scratch-pad/probe-*.wat (5 files)      |   2 +- each
 wat-scripts/topic/sns-fanout.wat                   |   4 +-
 8 files changed, 123 insertions(+), 49 deletions(-)
```

Extra six files: `_` last only. `Topic::StatsResponse` stays `[n ticks]`.
`git diff -- wat-scripts/topic wat-scripts/scratch-pad` is eight trailing
bindings and nothing else.

`time::now` in `sqs.wat`: comments at `:7` `:11`, test helper at `:1584`.
No new occurrence around `Store/*`.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `store-calls=` on phases | ✅ n=500 `store-calls=2468` |
| 2 | store-calls > receive-calls | ✅ 2468>212, 4973>412, 10174>839 |
| 3 | roughly doubles as n doubles | ✅ ×2.015, ×2.046. Per pair 1.234 / 1.243 / 1.272 |
| 4 | no new `time::now` around Store | ✅ still 3, none at the eight sites |
| 5 | no-args pre-existing identical | ✅ `store-calls` additive |
| 6 | n=2000 `total=8000;distinct=8000;dup=0` | ✅ ⚠ seen-skipped=60 |
| 7 | curve n=500/1000 ±15% of 3922/3160 | ✅ 3945 / 3130 |
| 8 | 38 drop tests | ✅ 38 PASS lines with `drop` |
| 9 | `every_wat_scripts_file_loads` | ✅ last floor test, 494.000s, PASS |
| 9b | extra files `_` only | ✅ topic + five probes |
| 10 | floor | ✅ `5221 passed (7 slow), 22 skipped` |

**STOP-1 through STOP-6 did not fire.** STOP-1 was the previous strike;
this redraw named the eight files.

No `wat/`. No timing. The number is reported, not acted on.

---

# GRADING — claude, 2026-09-07

**STRUCK.** All rows verified on my own runs and reads. Floor mine:

```
Summary [ 463.944s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## ⛔ THE FORK RESOLVED — FLAT

| n | pairs | store-calls | **per pair** | drain ms | **ms per pair** | pairs/sec |
|---|---|---|---|---|---|---|
| 500 | 2000 | 2478 | **1.239** | 503 | 0.2515 | 3976 |
| 1000 | 4000 | 4989 | **1.247** | 1250 | 0.3125 | 3200 |
| 2000 | 8000 | 10124 | **1.266** | 3199 | 0.3999 | 2501 |

grok: 1.234 / 1.243 / 1.272. Two independent runs agree.

★★★ **Store round trips per message are flat (+2.2 % across 4× depth) while wall time per message
rises 59 %.** The count explanation is dead.

## ⛔ AND MY FORK STATEMENT WAS TOO NARROW

The DESIGN said: *flat → "op count is linear, **cost per op is rising with table size**."*

**That is one of two survivors, not the answer.** A store round trip is queue → store service → back,
so its duration is the store's own SQL work **plus queueing at the serializing store actor**. Flat
counts eliminate *count*; they do not choose between **cost** and **contention**.

★ I wrote a fork with two branches and then quietly named a winner inside one of them — the same
shape of error this stone exists to prevent. The honest statement of where we are:

> Per message, the same number of store round trips take 59 % longer at 4× depth. **Either the
> store's own work per op grows with table size, or waiting for the store actor does.** This
> instrument cannot separate them.

## ⛔ THE ASSUMPTION THE COUNTER WAS BUILT TO TEST IS REFUTED

The DESIGN's arithmetic was *"833 receive calls × (1 scan + **10 claim-puts**) plus 8000
ack-deletes"* → ~17 163 calls at n=2000. **Measured: 10 124.**

★★ The put is **batched** — one round trip for ten rows (`(Store/put st (PutRequest rows))`). Had I
shipped that arithmetic as a finding instead of counting, I would have been **1.7× wrong in a
plausible-looking direction**, and it would have read as evidence for the cost-per-op story.

## ★★★ A NUMBER WORTH KEEPING — the shallow regime costs 3.8× the store work

```
overlapped (no-args)   38 579 store-calls / 8000 pairs = 4.82 per pair
fill-first  (n=2000)   10 124 store-calls / 8000 pairs = 1.27 per pair
```

Same messages, same code. **3.8× more store round trips per message when the queue is shallow.**
This independently corroborates the receive-batching finding (1.5 vs 9.4 messages per receive call)
and quantifies it one layer down — a trickling queue does not merely cost more *time* per message,
it issues more *work*.

## ⚠ AN OBSERVATION I AM NOT CLAIMING

`seen-skipped` at n=2000:

```
before this stone   30 (grok)   40 (mine)
after  this stone   60 (grok)  100 (mine)
```

Throughput did not move (2382–2501 pairs/sec, both stones). The counter adds a field to every
`State` reconstruction inside the queue actor, which could nudge mark replies past their 200 ms
deadline → more retries → more absorbed duplicate marks.

⚠ **Four samples is not a measurement**, and `distinct=8000; dup=0` holds throughout. Recorded as an
observation that needs a real sample, **not a finding.** It is also the thing EXPECTATIONS row 7 was
meant to catch and did not — row 7 banded `pairs/sec` at n=500/1000, and this moves neither.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=500 run → `store-calls=2478` | ✅ |
| 2 | 2478>212, 4989>415, 10124>833 | ✅ |
| 3 | ×2.013, ×2.029; per pair 1.239/1.247/1.266 | ✅ |
| 4 | `grep time::now sqs.wat` → 3, none at the eight sites | ✅ |
| 5 | my own no-args run, field by field | ✅ `store-calls` additive |
| 6 | my own n=2000 → `total=8000;distinct=8000;dup=0` | ✅ ⚠ `seen-skipped=100`, above |
| 7 | my own 3976 / 3200 vs 3922 / 3160 | ✅ +1.4 % / +1.3 % |
| 8 | my own floor, 38 drop tests | ✅ |
| 9 | `every_wat_scripts_file_loads` PASS in my floor | ✅ |
| 9b | `git diff -- topic scratch-pad` → **8 insertions, 8 deletions, every one a trailing `_`** | ✅ boring |
| 10 | my own run, Summary line read | ✅ |

**STOP-1 through STOP-6 did not fire.**

## WHAT THE INSTRUMENT CANNOT SEE — stated so it is not over-read

`store-calls` is **cumulative over the whole run**, with **no breakdown by op** and **no fill/drain
split**. It answers the fork and nothing else. Any composition claim ("so many scans, so many
puts") is unsupported by it — I attempted that arithmetic above and it was wrong by 1.7×.

## THE NEXT STONE, CHOSEN BY THE NUMBER

Cost-vs-contention is now the only open branch, and separating them needs **time inside the store
call** — which is what DESIGN deferred and STOP-5 forbade, on the grounds that timing overhead should
not land before we knew timing was the question. **It is now the question.**

⚠ And the overhead objection is unchanged: two `time::now` reads per store op is ~20 000 extra clock
reads at n=2000, inside the phase under measurement. The next DESIGN must confront that — measure
the clock-read cost, or find a decomposition that does not need per-op timing.

Also open, unchanged: the `Lost`/`Closed` → `Exhausted 0` conflation; `collect` (15.7 s measured
today vs 5.9 s in the tracker); the 2.4× at m=8.
