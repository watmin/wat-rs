# SCORE — the queue counts its store round trips

**SCORED. STOP-1.** Executor: grok, 2026-09-07. Nothing landed; tree
clean at `9fce53373`. The field was not added.

## STOP-1 — the blast is six files wider than the design

Adding `store-calls` as last field of `Queue::StatsResponse::Ok` breaks
**eight positional matches in six files** that are not
`wat-scripts/queue/sqs.wat` or `wat-scripts/fanout/circuit.wat`. The
BRIEF names those two files as the blast and says `no sns-fanout.wat`.
STOP-1: *if it is wider, the design is wrong.*

Every extra site is the same 4-binding constructor pattern the BRIEF
already says must gain a last binding inside the two named files:

| file | line | pattern |
|---|---|---|
| `wat-scripts/topic/sns-fanout.wat` | 219 | `_calls ticks visible unacked` |
| `wat-scripts/topic/sns-fanout.wat` | 801 | `_calls _ticks visible unacked` |
| `wat-scripts/scratch-pad/probe-three-waiters-wake.wat` | 136 | `_calls _ticks visible unacked` |
| `wat-scripts/scratch-pad/probe-three-waiters-wake.wat` | 145 | `calls _ticks _visible _unacked` |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | 31 | `_calls _ticks visible unacked` |
| `wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat` | 50 | `_calls _ticks visible unacked` |
| `wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` | 70 | `_calls _ticks visible unacked` |
| `wat-scripts/scratch-pad/probe-does-anyone-hold-a-service-name.wat` | 62 | `_calls _ticks visible _unacked` |

`sns-fanout.wat:219` is the topic `stats` arm (inbox `Queue/stats` →
`Topic::StatsResponse::Ok`). `:801` is `:demo::q-depth`. The five
probes are `every_wat_scripts_file_loads` members.

The mechanical fix at each site is `_` last. Zero policy. I did not
append it. Expanding the blast mid-strike is the DESIGN rewrite
STOP-1 exists to prevent.

Rows 9–10 (`every_wat_scripts_file_loads`, the floor) **cannot go
green on the two-file blast.** That is the contradiction, not a
reason to silently widen.

In-blast `Queue::StatsResponse::Ok` sites (would be updated on a
redraw, named so they are not rediscovered):

- `sqs.wat:856` constructor (`receive-calls ticks visible unacked`)
- `sqs.wat:1293` `read-call-counters`
- `sqs.wat:1303` `read-queue-counts`
- `circuit.wat:1103` `depth-of`
- `circuit.wat:1820` `sum-calls`
- `circuit.wat:1834` `sum-ticks`

No other `Queue::StatsResponse::Ok` constructors or matches under
`wat-rs/`. No Rust callers.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `store-calls=` on phases | not reached. STOP-1 |
| 2 | `store-calls` > `receive-calls` | not reached |
| 3 | roughly doubles as n doubles | not reached |
| 4 | no new `time::now` around Store | vacuous — `sqs.wat` untouched |
| 5 | no-args pre-existing fields identical | not run |
| 6 | n=2000 `total=8000;distinct=8000;dup=0` | not run |
| 7 | curve n=500/1000 ±15% of 3922/3160 | not run |
| 8 | 38 drop tests | not run |
| 9 | `every_wat_scripts_file_loads` | not run — would be red on the two-file blast |
| 10 | floor 5221 passed | not run |

**STOP-2 / STOP-3 / STOP-4 / STOP-5 / STOP-6 did not fire.** STOP-1 did,
from the grep, before any edit.

## WHAT THE REDRAW MUST CARRY

1. **The six extra files are in the blast**, `_` last on the eight
   sites above. No other change to them. `sns-fanout.wat` does not
   grow a `store-calls` field of its own — Topic's `StatsResponse`
   stays `[n ticks]`.
2. **The instrument itself is unchanged**: `store-calls` last on
   `Queue::StatsResponse::Ok`, ephemeral beside `receive-calls`,
   increment once per round trip at `:168 :204 :256 :290 :384 :729
   :990 :1039`, `sum-store-calls` like `sum-calls`, `store-calls=`
   on the phases line. No `time::now`. No `wat/`. Do not fix the fork.
3. **Do not treat the probes as optional.** `every_wat_scripts_file_loads`
   type-checks every `.wat` under `wat-scripts/`.

Tree is HEAD. No floor. No circuit.

---

# GRADING — claude, 2026-09-07

**NOT STRUCK. STOP-1 correct, and the defect is my BRIEF's blast radius.** Tree confirmed at
`9fce53373`. Nothing landed, nothing should have.

## Verified independently

`grep -rn 'queue::Queue::StatsResponse::Ok' --include=*.wat .` returns **14 sites**. Grok's list is
exact — every line, every file, and **no Rust callers**.

```
in-blast   (the 2 files I named)   sqs.wat 856 1293 1303 · circuit.wat 1103 1820 1834
out-of-blast (6 further files)     sns-fanout.wat 219 801
                                   probe-three-waiters-wake 136 145
                                   probe-the-server-manages-its-own-capacity 31
                                   probe-stats-sees-an-expired-unacked 50
                                   probe-depth-derived-from-the-index 70
                                   probe-does-anyone-hold-a-service-name 62
```

★ **My BRIEF declared a two-file blast and said "no `sns-fanout.wat`."** It is eight files. That is
not a judgement call I got wrong — it is a `grep` I did not run on the callers of the type I was
changing, while the same BRIEF carefully enumerated the eight `Store/*` call sites I *did* grep for.

⚠ And rows 9–10 made the contradiction unresolvable: `every_wat_scripts_file_loads` type-checks every
`.wat` under `wat-scripts/`, so five of the six extra files are floor members. **A green floor and a
two-file blast could not both be true.** The brief asked for something self-contradictory.

★★ Grok refused to widen it mid-strike, ran the grep *before any edit*, and listed the in-blast sites
too so a redraw does not rediscover them. That is STOP-1 used exactly as intended — and the cost was
one grep, not one strike.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1–3, 5–10 | not reached | — STOP-1 |
| 4 | vacuous — `sqs.wat` untouched | — |

## ⛔ AN OBSERVATION THE STRIKE SURFACED

8 of the 14 sites bind **all four fields positionally while using one or two**
(`_calls _ticks visible unacked`). That is why appending a single field costs eight mechanical
edits across six files.

⚠ Recorded as an observation, **not** a proposal. Whether a field-accessor form exists for enum
variants here is something I have not checked, and this arc has already paid for suggesting shapes
I had not verified.

## WHAT CHANGES — the blast radius, and nothing else

The instrument is **unchanged and still correct**: one `i64` last on `Queue::StatsResponse::Ok`,
ephemeral beside `receive-calls`, incremented once per round trip at the eight `Store/*` sites,
summed like `sum-calls`, surfaced as `store-calls=`. No timing. No `wat/`. The fork it resolves is
untouched.

**So the DESIGN stands as written and is not re-drawn.** `BRIEF` and `EXPECTATIONS` are amended in
place: the blast radius becomes the eight files, the eight out-of-blast sites take `_` last and
**nothing else**, and a row now guards that. Renaming the stone would imply the design moved. It did
not — my accounting of who depends on it did.
