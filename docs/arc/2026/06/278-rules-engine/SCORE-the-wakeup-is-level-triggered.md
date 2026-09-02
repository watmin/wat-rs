# SCORE — the wakeup is level-triggered

**STRUCK.** Executor: grok, 2026-09-02. Every starred circuit size run twice.

The park and the repair shipped together. `1000×4×3` and `2000×4×3` — the sizes that
hung — both complete, both lossless, both times.

```
Summary [ 351.218s] 5183 tests run: 5183 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T07-35-05Z/`

## What landed

`wat-scripts/queue/sqs.wat`, `wat-scripts/topic/sns-fanout.wat`,
`wat-scripts/fanout/circuit.wat`.

- One helper per service, closed in `:init` (process children do not see sibling
  defns — the worker-pump zombies from the last strike). `arm-tick` /
  `arm-deliver` decide the `arms` vector for every return path.
- `tick-armed?` / `deliver-armed?` on `:ephemeral`. `-tick` / `-deliver` pass
  `false` into the helper (the alarm was consumed). The helper sets `true` when
  it arms.
- `was-empty?` gone from both services (row 5).
- `Alarm :after` exists once per service, inside the helper (row 6).
- Worker `:wait-ns 250000000`. Held-worker untouched. `wat/` / `src/` empty.

## Scale matrix (row 11)

| N | M | J | result | calls | ticks | drain ms | stop ms | wall s |
|---|---|---|---|---|---|---|---|---|
| 12 | 4 | 3 | ✅ | 60 | — | 279 | 1022 | 10.1 |
| 500 | 4 | 3 | ✅ | 2016 | 194 | 9793 | 2468 | 21.4 |
| 1000 | 2 | 3 | ✅ | 2012 | 132 | 13928 | 1699 | 20.9 |
| 1000 | 4 | 2 | ✅ was hang | 4016 | 376 | 20667 | 2475 | 31.2 |
| 1000 | 4 | 3 | ✅ was hang, **twice** | 4033 / 4036 | — | 29994 / 30096 | 2649 / 2407 | 42.2 / 42.0 |
| 2000 | 4 | 3 | ✅ was hang, **twice**, then ticks run | 8080 / 8079 / 8052 | 2350 | 96614 / 96757 / 73461 | 4469 / 4193 / 4487 | 111.1 / 111.2 / 87.7 |

All lossless: `dup=0`, `empty=1`, and at weight `total=8000; distinct=8000`.

The third 2000 run is after deadline-based re-arm on `-tick` (the 1 ms stub
used on the first two). Drain dropped 96 s → 73 s. `ticks=2350` is the same
order as 8000 messages, not 141,297 receive calls.

## Row 4 — the flag is allowed to exist because it is checked

`wat-scripts/scratch-pad/probe-three-waiters-wake.wat` now drives **stats ten
times while waiters are parked**, then sends. J=1,2,3 still
`status=drained; p=0; f=0; got=9` on the first wait check.

That is the missed-return-path class: stats used to return an empty `arms`
vector while waiters were non-empty. If the helper's `armed?=true` branch
were wrongly taken with no alarm outstanding, this send would not wake and
the probe would timeout with leftover depth — the same stuck tail as
`SCORE-the-workers-stop-polling.md`. The previous strike *is* that failure:
park adopted, helper absent, `1000×4×3` frozen at `out=1; q*=p1f1`.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ deadlock gone at 1000×4×3 and 2000×4×3 | ✅ both complete, both twice |
| 2 | ★ `total=8000; distinct=8000; dup=0` | ✅ both 2000 runs |
| 3 | ★ park is on | ✅ worker `:wait-ns 250000000` |
| 4 | ★ invariant asserted | ✅ stats-while-parked still wakes; previous hang is the flag-wrong shape |
| 5 | ★ `was-empty?` gone | ✅ zero hits in queue + topic |
| 6 | ★ one place decides | ✅ `Alarm :after` only in the two helpers |
| 7 | empty polls collapse | ✅ **8052** calls against 141,297 |
| 8 | ticks do not amplify | ✅ **2350** at 2000×4×3 — message order, not request order |
| 9 | no substrate change | ✅ `git diff wat/ src/` empty |
| 10 | held-worker untouched | ✅ still `:wait-ns 50000000`; circuit diff has 0 held-worker lines |
| 11 | scale matrix | ✅ all six sizes complete and lossless |
| 12 | wall time | **87.7 s** against 85.3 s polling — reported, not promised |
| 13 | floor | ✅ 5183/5183, `FLOOR=0`, `.floor/2026-09-02T07-35-05Z/` |

## The shape that landed

```
arm iff collection non-empty AND not already armed
-tick / -deliver pass armed?=false (alarm consumed)
every Outcome/SelfOutcome takes arms from the helper
```

Process children cannot see sibling defns, so the helper is closed in `:init`
and stored on state (same reason as `State/take`). A file-level defn would
zombie the queue the way `:fanout::worker-pump` zombied the workers.

## Wall time (row 12)

87.7 s against 85.3 s polling. Receive calls 8052 vs 141,297. The hops
moved; drain is still ~73 s of topic `-deliver` plus the untouched drain
poller. A modest wall with rows 1 and 7 green is a successful stone.

`stop=` at weight is 4.2–4.5 s: `collect-stop` is sequential over 12
process workers, each waiting out an in-flight 250 ms park. Not 12 × wait
inside one Stop, and not a hang.

## Next

The substrate `ensure-alarm` outcome named in the DESIGN is still the
rung-3 repair — "armed twice" and "armed zero times" both unrepresentable,
no flag. It wants `wat/` and its own stone. The flag is bookkeeping that
row 4 is allowed to exist because it is checked.
