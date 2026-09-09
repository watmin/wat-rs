# SCORE — the ack retries like the publisher does

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. Ack ladder replaced with vis-bounded
backoff; `ack-retries` threaded like `gave-back`.

```
Summary [ 493.900s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T09-47-13Z/`

## THE ACK LOOP

`once-a` / `aa1`/`aa2`/`aa3` is gone. Per-call 200 ms stays. Receive 1000 ms
and `:1531` untouched.

A process child is assembled from service-forms only — it cannot see
`circuit.wat` helpers (`backoff-delay`, `await-timer-ms`, `elapsed-ms`). Same
constraint as the publisher `-run`. The worker inlines the formula
(`BASE=1`, `CAP=100`, uniform `[1, ceiling]`, seed threaded) and `await-ms`.
Happy path is one `call-by-deadline`. Only `DeadlineFired` with
`elapsed < vis-ns/1000000` enters the backoff foldl (8192, elapsed check
stops first). `Lost`/`Closed` redial and stop this tick, as today.

```
limit-ms = vis-ns / 1000000   ;; after vis expiry retrying is pointless
```

`ack-retries` is per receive-limit-10 **batch**, not per message. On
`worker::Record`, `DisruptsResponse::Ok`, `sum-disrupts` (3-tuple
hits/gb/ars), summary, and phases.

## STOP-4 — n=2000 fill-first still fails

```
drained-never: last=[0/0][0/0][0/0][0/10] outbox=0
attempts=8000 elapsed=211952;ack-retries=0;gave-back=1
```

Did not re-run. Did not raise the drain bound.

`ack-retries=0` and `gave-back=1` with exactly one batch (10) unacked:
the stranded batch never reached ack. Check-ladder `a1`/`a2`/`a3` still
discards after three flat 200 ms (`:516`, `gb-tick=1`, no ack). vis is
10¹² ns, so those 10 stay claimed past the drain's `n×m` bound.

The ack path is not the terminal event at this depth. The check path is
the same three-rung discard, one hop upstream.

## NO-ARGS / CURVE

No-args summary:
`n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;seen-skipped=0;gave-back=0;ack-retries=0`
Existing fields identical; `ack-retries=0` is additive. fill-depth
`[1/1][2/1][4/4][2/5]`, arm=0.

pairs/sec = (n×m) / drain_s. Baselines 4348 / 4167 / 3802 / 3244.

| n | fill-depth | drain ms | poll-calls | pairs/sec | ack-retries | vs baseline |
|---|---|---|---|---|---|---|
| 100 | [100/0]×4 | 99 | 25 | 4040 | 0 | −7 % |
| 250 | [250/0]×4 | 271 | 45 | 3690 | 0 | −11 % |
| 500 | [500/0]×4 | 503 | 95 | 3976 | 0 | +5 % |
| 1000 | [1000/0]×4 | 1259 | 250 | 3177 | 1 | −2 % |
| 2000 | fill reached drain | bound | 8000 | void | 0 (failure path) | — |

n=250 is 11 % below 4167 (previous stone 4219). Box had grok+claude.
Not re-run. Counter is readable: n=100 is 0, n=1000 is 1.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ n=2000 fill-first completes | ❌ STOP-4. Sweep and counts above. |
| 2 | `ack-retries=` in summary and phases | ✅ both; also on the failure path via require! |
| 3 | counter readable at depth | ✅ n=100 = 0; n=1000 = 1. n=2000 did not complete |
| 4 | bound is `vis-ns / 1000000` | ✅ comment at `:604-606` |
| 5 | two `backoff-delay` call sites | ❌ named helper still one caller (`:1230` publisher). Child inlines. See above. |
| 6 | no-args unchanged | ✅ existing fields identical; `ack-retries=0` added |
| 7 | chaos unchanged | ✅ drop-ack/recv/before/after tiny: `total=100;distinct=100;dup=0`. nextest `drop` 38 passed |
| 8 | curve within ~10 % | ⚠ n=250 −11 %; others inside. See table |
| 9 | n=2000 gets a drain | ❌ STOP-4 |
| 10 | scripts load | ✅ floor includes `every_wat_scripts_file_loads` |
| 11 | the floor | ✅ `5221 passed (7 slow), 22 skipped` |

**STOP-1 / STOP-2 / STOP-3 / STOP-5 did not fire.** STOP-4 did.

## NOT TOUCHED

Check ladder (`once`/`a1`/`a2`/`a3`). Mark ladder. Receive 1000 ms.
`:1531`. Per-call ack 200 ms. `wat/`, `sqs.wat`, `sns-fanout.wat`.
The 25 % slope 4348→3244.

---

# GRADING — claude, 2026-09-07

**THE CHANGE LANDS. THE STONE'S PURPOSE DOES NOT.** Row 1 fails, and it fails because **my
diagnosis named the wrong ladder.** Two further rows were bad rows of mine. Floor mine:

```
Summary [ 475.648s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop-related tests passed** (row 7).

## ⛔⛔ THE FINDING — there are THREE ladders and I fixed the one that was not firing

My own n=2000 run:

```
drained-never: last=[0/0][0/0][0/0][0/40] outbox=0 attempts=8000
               elapsed=227730; ack-retries=5; gave-back=4
```

grok's: `[0/0][0/0][0/0][0/10] … ack-retries=0; gave-back=1`

★★★★★ **Exact correspondence in both runs: `gave-back` == stranded batches, and unacked ==
gave-back × 10.** Four abandonments, forty rows; one abandonment, ten rows. Two independent
samples, same law.

★★ And `ack-retries=5` proves the ack fix is **live and working** — it retried and recovered. It is
simply not the failure path.

The worker's impl holds **three** hand-rolled retry ladders, each with the same shape:

| ladder | line | on exhaustion | consequence |
|---|---|---|---|
| **check** | `:513-518` | returns `gb-tick 1` → `gave-back` | **the terminal event.** Batch never acked; `vis` is 10¹² ns, so it is stranded past any bound |
| **mark** | `:574-576` | `second mm3` **discarded — no counter at all** | delivered-but-unrecorded; `Seen` cannot dedupe it, so a later redelivery is a **duplicate** |
| **ack** | `:591+` | **fixed by this stone** — vis-bounded backoff, counted | — |

⚠ `:518` is the **only** producer of `gb-tick = 1`. So `gave-back` counts exactly one event: a
check-ladder abandonment. Nothing is given back — the batch is stranded. **The instrument was
already there and its name says the opposite of what happened.**

★★★ **My error, precisely.** I read the ack ladder, found a real defect, and assumed it was *the*
defect. I never enumerated the alternatives — and the rule I have written down for exactly this is
*read the implementation of every candidate you are about to rule out.* I never generated the
candidate list. Three ladders sit within eighty lines of each other.

★ The mark ladder is the worse of the two remaining: liveness failures announce themselves
eventually (a bound fires), but a lost **mark** is a silent correctness hole — and unlike the check
ladder it has no counter of any kind.

## ⛔ TWO ROWS WERE BAD ROWS — mine, not grok's

**Row 5** demanded two `backoff-delay` call sites. **Architecturally impossible.** `:env-fn
"(:wat::program::EmptyEnv)"` (`:2086`) means a process child cannot see `circuit.wat` helpers. The
formula now exists in three copies:

```
:1204-1206   backoff-delay          the named helper (parent)
:1540-1541   ceiling / drawn        the Publisher child's inline   ← PRE-EXISTING
:675-676     ceiling / drawn        the worker child's inline      ← this stone
```

⚠ And the tracker **already records this** — *"already duplicated once into the Publisher child,
because a process child cannot see script helpers. That duplication is why its bugs kept
surfacing."* I read that line during recovery this session and then wrote a row requiring its
violation. Grok followed the only pattern available and was right to.

★★ It also sharpens the tracker's point: `100` is now hardcoded in **two** of the three copies.
Tuning `BACKOFF-CAP-MS` would move one of three.

**Row 8** put a ±10 % band on drains lasting ~100 ms on a shared box.

| n | mine | grok | baseline |
|---|---|---|---|
| 100 | 3448 (−21 %) | 4040 (−7 %) | 4348 |
| 250 | 4167 (0 %) | 3690 (−11 %) | 4167 |
| 500 | 4073 (+7 %) | 3976 (+5 %) | 3802 |
| 1000 | 3320 (+2 %) | 3177 (−2 %) | 3244 |

★ **We each produced one outlier, at a different n, in opposite directions**, while n=500 and
n=1000 sat inside the band both times. The band was tighter than the instrument at small n. The
honest reading: **this stone did not disturb the curve where the curve can be measured.**

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=2000 run, reproduced | ❌ **real — cause is upstream** |
| 2 | summary, phases, **and the failure path** | ✅ beyond brief; it is what made the diagnosis |
| 3 | my n=2000 `ack-retries=5`; n≤1000 all `0` | ✅ |
| 4 | `ack-limit-ms (:wat::i64::/ vis 1000000)` at `:606`, commented `:604-605` | ✅ derived |
| 5 | `grep backoff-delay` → one caller | ❌ **my bad row — impossible** |
| 6 | my own no-args run, field by field | ✅ `ack-retries=0` additive |
| 7 | my own floor, 38 drop tests | ✅ |
| 8 | my own curve, four points | ⚠ **my bad row — band too tight at small n** |
| 9 | follows row 1 | ❌ |
| 10 | inside the floor | ✅ |
| 11 | my own run, Summary line read | ✅ |

## ⛔ WHAT THE STONE ACTUALLY DELIVERED

Not its gate. Something more useful: **grok put the counters on the failure path**, which the brief
did not ask for, and that is what converted an opaque 227-second hang into an attributed one. The
ack ladder was genuinely defective and is now correct; `ack-retries=5` is the proof it fires.

⚠ **The change is kept.** It is correct, the floor is green, chaos is unchanged, and reverting
would discard both a real fix and the instrument that found the real cause.

## THE NEXT STONE — do not patch the third stem

★★★ Three ladders, three hand-rollings, one shape. Fixing `mark` the way I fixed `ack` would be
**patching the third stem** — the precise thing this arc's own discipline forbids.

The root is the *pattern*: a fixed-rung retry whose exhaustion flag is a `bool` the caller may drop.
The fix is **one bounded-retry combinator, local to the worker's impl** (a `fn` in the service body
— reachable by the child, unlike a script helper), taking `(op, inert, limit-ms)` and returning an
**outcome that must be matched**, so exhaustion cannot be discarded by any of the three call sites.

Open and unchanged: the 25 % slope (4348 → 3244), measured with zero abandonments; and the curve
past n=2000, which nobody has seen.
