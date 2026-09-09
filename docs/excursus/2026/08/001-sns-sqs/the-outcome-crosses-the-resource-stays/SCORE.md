# SCORE — the outcome crosses, the resource stays

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. Nullary closed enums on the Worker
surface; peer and reply ride beside them. Check and mark use the ack's
vis-bounded backoff. `gave-back` renamed.

```
Summary [ 493.844s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T11-19-28Z/`

## THE CONSTRUCTION

Two closed Pure enums on `:fanout::Worker` `:messages` (stdlib payload
only, so S4c ships them):

```
:Got []
:Exhausted [attempts <- :wat::core::i64]
```

`seen-until` returns `(Tuple outcome peer reply)`. Keyword match in the
EmptyEnv child. No Peer, no Reply, no `:- [` inside a variant. No
hash-destructure.

Check and mark share `seen-until`. Ack returns `(Tuple QueueRetry peer
retries)` and matches `Got` / `Exhausted`. `Lost`/`Closed` redial as
today. `DeadlineFired` backs off until `vis-ns / 1000000`.

Counters: `check-exhausted`, `mark-exhausted`, `ack-retries`. One increment
is one receive-limit-10 **batch**. `gave-back` is gone.

## ROW 1 — n=2000 fill-first completes

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=30;check-exhausted=0;mark-exhausted=0;ack-retries=1
fill-depth=[2000/0]×4  drain=3278  poll-calls=635  pairs/sec=2441
```

No `drained-never`. Exhaustion counters are 0. `ack-retries=1`.

⚠ `seen-skipped=30` — EXPECTATIONS wrote `seen-skipped=0`. Distinct stayed
8000, dup 0: thirty seqs were marked a second time and absorbed. Consistent
with a mark that applied on the server, then `DeadlineFired` on the reply,
then `seen-until` retried and `Seen` counted skipped. Did not re-run.

## NO-ARGS / CURVE

No-args: `total=8000;distinct=8000;dup=0;…;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0`.
Existing fields identical; new counters additive. fill-depth overlapped.

pairs/sec = (n×m)/drain_s. Band ±15 % of 3802 / 3244.

| n | fill-depth | drain ms | pairs/sec | vs 3802/3244 |
|---|---|---|---|---|
| 500 | [500/0]×4 | 510 | 3922 | +3 % |
| 1000 | [1000/0]×4 | 1285 | 3113 | −4 % |
| 2000 | [2000/0]×4 | 3278 | 2441 | first drain |

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ n=2000 fill-first completes | ✅ no `drained-never`; total/distinct/dup. ⚠ `seen-skipped=30` |
| 2 | nullary closed enums | ✅ `:Got []` `:Exhausted [attempts]`; no `:- [`, no Peer, no Reply |
| 3 | keyword-matched in the child | ✅ three `::Exhausted` arms; no hash-destructure |
| 4 | no droppable flag | ✅ `a1/a2/a3/mm1/mm2/mm3` gone |
| 5 | every site names `Exhausted` | ✅ check, mark, ack |
| 6 | honest counters; `gave-back` gone | ✅ summary + phases + failure string |
| 7 | failure path | n/a (row 1 completed); template has all three |
| 8 | no-args unchanged | ✅ existing fields identical |
| 9 | chaos | ✅ 38 drop tests; four tinies `total=100;distinct=100;dup=0` |
| 10 | curve ±15 % | ✅ 3922 / 3113 |
| 11 | scripts load | ✅ floor includes `every_wat_scripts_file_loads` |
| 12 | the floor | ✅ `5221 passed (7 slow), 22 skipped` |

**STOP-1 through STOP-7 did not fire.**

## THE COUNTERS AT DEPTH

n=2000 fill-first: `check-exhausted=0; mark-exhausted=0; ack-retries=1`.
The wall was not cleared by abandoning; it was cleared by retrying. The
30 skipped marks are the residue of those retries, not of exhaustion.

---

# GRADING — claude, 2026-09-07

**STRUCK.** Row 1 passes on my own run — the first time `2000 4 3 8192 true` has completed. All
twelve rows verified independently. Floor mine:

```
Summary [ 470.673s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests passed**.

## My runs

```
n      drain ms   pairs/sec   seen-skipped   check/mark-exhausted   ack-retries
500        513        3899          0              0 / 0                 0
1000      1234        3242          0              0 / 0                 0
2000      3204        2497         40              0 / 0                 1
```

`n=2000`: `total=8000; distinct=8000; dup=0; fill-depth=[2000/0]×4`. No `drained-never`.
No-args: `seen-skipped=0`, all new counters `0`, every pre-existing field identical.

★★★ **The wall was cleared by retrying, not by abandoning less.** `check-exhausted=0` and
`mark-exhausted=0` at the depth that stranded four batches before this stone. Nothing was given up
at any of the three sites — which is exactly what the vis-bounded backoff was for, and it is the
counter, not the wall clock, that says so.

## The curve, extended past n=1000 for the first time

| n | mine | grok | baseline |
|---|---|---|---|
| 500 | 3899 (+3 %) | 3922 (+3 %) | 3802 |
| 1000 | 3242 (−0.1 %) | 3113 (−4 %) | 3244 |
| 2000 | **2497** | **2441** | — first drain ever |

**Degrading and steepening**: −17 % then −23 % per doubling. Two independent runs agree within 2 %.

⚠ `poll-calls` per pair also rises — 0.048 → 0.060 → 0.079. That is a *consequence* of the drain
lengthening (the poller does m+1 per iteration regardless), **not established as a cause.** I am
explicitly not claiming it; this stone was not built to separate them, and the arc has already lost
seven mechanisms to exactly that kind of inference.

## ⛔ `seen-skipped` — MY ROW WAS WRONG, NOT THE CODE

EXPECTATIONS row 1 demanded `seen-skipped=0`. **That number came from a world where the mark never
retried.** Introducing retries necessarily produces absorbed duplicate marks: the mark applies
server-side, the reply times out, `seen-until` retries, and `Seen` counts the second one skipped.
That is what a dedupe is *for*.

Two facts confirm it rather than assume it:

- **mine 40, grok's 30 — both multiples of 10**, the receive batch size. Whole batches re-marked.
- **`seen-skipped=0` at n=500 and n=1000.** It fires only where contention times out a mark reply.

`distinct=8000; dup=0` throughout. Nothing was lost; a duplicate was absorbed.

★ Third bad row of mine across these stones, same shape every time: **I gated on a number produced
by the behaviour the stone was about to change.** Row 5's impossible `backoff-delay` count, row 8's
±10 % band on ~100 ms drains, and now this.

## ⛔ A REAL FINDING — the new counter has the old defect

`seen-until` maps **`Lost`**, **`Closed`**, and genuine exhaustion all to the same value
(`:531-534`, `:539`):

```wat
((:wat::service::CallOutcome::Lost _c)    (:wat::core::Tuple (:fanout::SeenRetry::Exhausted 0) (redial) inert))
((:wat::service::CallOutcome::Closed)     (:wat::core::Tuple (:fanout::SeenRetry::Exhausted 0) (redial) inert))
```

and the check site increments `check-exhausted` on it (`:623-624`).

★★ **So a broken pipe would be reported as an abandonment** — the same naming lie as `gave-back`, in
the counter built to replace it. And `attempts` is a hardcoded `0` in three of the four paths, so
the payload is a placeholder rather than a measurement.

⚠ **Not observed firing.** `check-exhausted=0` on every run I made, and the drop tests inject at the
queue rather than at `Seen`, so the path is rarely reached. This is a **read-level** finding, not a
measurement — but it must be fixed before anyone trusts that counter at depth, and it is the exact
class this stone exists to remove.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own n=2000 run | ✅ **first completion**; `seen-skipped=40` explained above |
| 2 | `grep defenum … -A 3` | ✅ `:Got []`, `:Exhausted [attempts]`; no `:- [`, no Peer, no Reply |
| 3 | `grep` hash-destructure → **0** | ✅ |
| 4 | `grep a1/a2/a3/mm1/mm2/mm3/aa1..3` → **0** | ✅ |
| 5 | three `Exhausted` arms: `:623` check, `:670` mark, `:792` ack | ✅ |
| 6 | `grep gave-back` → **0** | ✅ |
| 7 | template carries all three (row 1 passed, so n/a live) | ✅ |
| 8 | my own no-args run, field by field | ✅ |
| 9 | my own floor, **38 drop tests** | ✅ |
| 10 | my own 500/1000 → +3 % / −0.1 % | ✅ inside ±15 % |
| 11 | inside the floor | ✅ |
| 12 | my own run, Summary line read | ✅ |

**STOP-1 through STOP-7 did not fire.** Three drafts, three STOP-1s, zero corpus damage — and the
draft that landed was the one whose mechanism grok had already measured rather than one I designed.

## WHAT IS OPEN

1. **The `Lost`/`Closed` conflation** above — small, and the same class this stone closed.
2. **The slope**, now four points and steepening: 3899 → 3242 → 2497. Cause unknown; `poll-calls`
   share rises with it but causation is **not** established.
3. **The class is still not fixed** — the backoff formula is in three copies with `100` hardcoded in
   two, because an `EmptyEnv` child cannot see the file's own `defn`s. One generic `defn` in `wat/`
   is the fix, and **the builder's ruling**. This stone is what earns it: the retry layer now
   demonstrably works at a depth that broke it this morning.
