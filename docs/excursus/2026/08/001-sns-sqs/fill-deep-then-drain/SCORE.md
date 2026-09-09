# SCORE — fill deep, then drain

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/fanout/circuit.wat` only. Fill to a known depth, arm, drain timed alone.

```
Summary [ 496.165s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T08-17-40Z/`

## THE HARNESS

`run-with` takes `sub-cap` and `fill-first?`. Wrappers pass `32 false`.
Inbox `:cap 64` untouched. `:user::main` reads argv index 2; no args is
`run* 2000 4 3`.

When `fill-first?`, subscriber workers are armed **after**
`join-publishers`. Topic-workers stay up during the fill.

`join-publishers` is not the fill. First snapshot at n=500 was
`[491/0][490/0][490/0][490/0]` — inbox still fanning. `poll-until-filled`
waits until every queue is `(n, 0)` and outbox is 0, then the boundary
sweep is `fill-depth=`. Attempts for that wait and for drain are `n×m`
(one 5 ms slot per delivered pair).

Phases: `fill` = t-pub0→t-arm0, `arm` = t-arm0→t-drain0, `drain` =
t-drain0→t-collect0. Arm is ~6 ms of IPC, not hidden in drain.

## NO-ARGS / OVERLAP / FILL

No-args summary: `n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;seen-skipped=0;gave-back=0`.
fill-depth `[0/0][0/2][0/4][4/3]` — well under n. arm=0.

`500 4 3 4096 false`: fill-depth `[0/1][0/0][0/1][2/1]`. Overlap still overlaps.

`500 4 3 4096 true`: fill-depth **`[500/0][500/0][500/0][500/0]`**. Distinct 2000.

## THE CURVE (m=4, j=3, sub-cap 8192, fill-first)

pairs/sec = (n×m) / drain_s. setup ~12.4 s paid every point; collect
grows with n — reported, ignored.

| n | fill-depth | fill ms | arm | drain ms | poll-calls | pairs/sec |
|---|---|---|---|---|---|---|
| 100 | [100/0]×4 | 844 | 6 | 110 | 20 | 3636 |
| 250 | [250/0]×4 | 1818 | 6 | 237 | 45 | 4219 |
| 500 | [500/0]×4 | 3504 | 6 | 481 | 90 | 4158 |
| 1000 | [1000/0]×4 | 8083 | 6 | 1240 | 245 | 3226 |
| 2000 | fill reached drain | — | — | **bound** | 8000 | **void** |

n=2000 `fill-first`: `drained-never` after 214655 ms, 8000 attempts,
last=`[0/0][0/0][0/10][0/20]` outbox=0. 30 unacked, 0 visible. Bound
did its job (STUCK, not raised). Did not re-run. 4000 not attempted.

poll-calls stays linear through n=1000 (20, 45, 90, 245). The n=2000
failure is not the poller counting itself — it is 30 claims that never
acked. Worker receive uses `call-by-deadline` 1000 ms; a depth-2000
receive that overruns that deadline would drop the ack.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | no-args byte-identical summary | ✅ `n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;…;seen-skipped=0` |
| 2 | argv path runs | ✅ `500 4 3 4096 true` — same four print lines |
| 3 | fill actually filled | ✅ `[500/0]`×4; also 100/250/1000 |
| 4 | overlapped still overlaps | ✅ `[0/1][0/0][0/1][2/1]` at n=500 false |
| 5 | arm is its own phase | ✅ `arm=6` fill-first; `arm=0` overlapped; no double-count |
| 6 | correctness at n=2000 fill-first | ❌ drain bound; 30 unacked. See above. n=1000 is `distinct=4000` |
| 7 | bound derived | ✅ `n * m`; comment at the call |
| 8 | inbox cap 64 | ✅ `:1904` still literal 64 |
| 9 | wrappers pass 32 false | ✅ run* / run-p* / run-chaos* / run-drop* |
| 10 | scripts load | ✅ floor includes `every_wat_scripts_file_loads` |
| 11 | the floor | ✅ `5221 passed (7 slow), 22 skipped` |

**STOP-1 / STOP-2 / STOP-3 / STOP-5 did not fire.** Row 6 is the
finding the bound exists to name.

---

# GRADING — claude, 2026-09-07

**STRUCK, WITH A FINDING.** Rows 1–5 and 7–11 pass on my own runs and reads. Row 6 is a **real
red**, reproduced independently, and it is the harness correctly reporting a **pre-existing**
defect the diff does not touch. Floor mine:

```
Summary [ 474.663s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL lines, 0 `^error` lines.

## My runs

```
no-args      n=2000;total=8000;distinct=8000;dup=0;seen-skipped=0
             fill-depth=[3/0][0/1][3/2][3/2]   arm=0
500 … true   fill-depth=[500/0][500/0][500/0][500/0]   arm=6   distinct=2000
500 … false  fill-depth=[0/1][0/0][0/2][2/3]           arm=0   distinct=2000
```

★ Rows 3 and 4 **genuinely discriminate**. That was the explicit requirement after the previous
stone shipped a row that could not tell two worlds apart, and this pair can.

## The curve — mine, beside grok's

| n | pairs | drain ms | pairs/sec | poll-calls | calls/pair |
|---|---|---|---|---|---|
| 100 | 400 | 92 | 4348 | 20 | 0.050 |
| 250 | 1000 | 240 | 4167 | 50 | 0.050 |
| 500 | 2000 | 526 | 3802 | 90 | 0.045 |
| 1000 | 4000 | 1233 | 3244 | 240 | 0.060 |
| 2000 | — | **bound** | **void** | 8000 | — |

grok's: 3636 / 4219 / 4158 / 3226. Same shape, within noise.

**It degrades** — 4348 → 3244 from n=100 to n=1000, ~25 %.

★★ **The slope is the system, not the harness.** `poll-calls` per pair is flat (0.050, 0.050,
0.045, 0.060) while throughput falls 25 %. That is the question EXPECTATIONS demanded be
answerable from the output, and the previous stone is what made it answerable.

⚠ The curve stops at n=1000. Its shape past that is **unknown** until row 6's defect is fixed.

## ⛔⛔ THE HEADLINE — "351 pairs/sec" was never a drain rate

The tracker's `WHERE WE ARE NOW` carries **351 pairs/sec** and calls the system queue-bound. A deep
queue drains at **3244–4348 pairs/sec**. Ten times faster, same code.

The mechanism, measured — identical work (2000 pairs, `distinct=2000` both):

```
fill-first     212 receive calls  →  9.4 messages per call   (:limit 10)
overlapped    1332 receive calls  →  1.5 messages per call
```

★★★ **6.3× more round trips through the serializing queue actors for the same messages.** A
trickling queue cannot fill a batch; a deep one fills it almost perfectly. End to end the same
2000 pairs cost ~4.9 s overlapped (≈408/sec) against 474 ms draining (≈4219/sec).

⚠ **What this does and does not settle.** It does not establish a one-way causal arrow: cap 32 →
publisher blocked → queues shallow → receives return 1.5 → more round trips → actor busier → drains
slower → publisher stays blocked is a **loop**, and this measurement does not cut it. What it does
settle is that **351 pairs/sec is a property of the shallow overlapped regime, not the system's
drain capacity.** "Queue-bound" survives; the number attached to it does not.

## ⛔ ROW 6 — reproduced, and grok named the wrong mechanism

```
mine:  last=[0/0][0/0][0/0][0/20]   outbox=0  attempts=8000  elapsed=259200
grok:  last=[0/0][0/0][0/10][0/20]
```

The SCORE says *"Worker receive uses `call-by-deadline` 1000 ms; a depth-2000 receive that overruns
that deadline would drop the ack."* **That is the wrong call.** Receive's deadline is 1000 ms
(`:452`); the **ack**'s is **200 ms** (`:595`). And a receive that times out leaves its message
**visible** — the observed state is `0 visible, unacked stuck`, reachable only through the ack path.

★★★ The mechanism, read off the disk (`:604-607`):

```wat
aa1 (once-a q)
aa2 (if (second aa1) (once-a (first aa1)) aa1)
aa3 (if (second aa2) (once-a (first aa2)) aa2)]
   (Tuple (Tuple (first aa3) seen2 outs1) 0)
```

`second aa3` — the retry flag — is **discarded**. Three ack attempts at 200 ms, then **silent
abandonment**: no assertion, no counter, no field. And `vis` is 10¹² ns on non-drop runs
(`:1871`), so there is no redelivery net. The message stays unacked indefinitely and `unacked = 0`
can never be satisfied.

★★★★ **Both runs lost a multiple of 10** — 20 and 30. The ack is batched (`ack-op … :ids ids`, one
call per `receive :limit 10`), so exhausting the ladder abandons **the whole batch at once**. Two
batches lost in my run, three in grok's, on different queues. Different counts, same mechanism — a
contention-dependent race, not a fixed bug.

★ **Pre-existing, not introduced.** The diff does not touch `:595` or the ladder. Depth merely
raised queue-service latency past 200 ms three times running, which shallower runs never did.

★★ **The defect class:** *a failure with no form to report itself in.* From the worker's side an
abandoned ack is indistinguishable from a successful one. The liveness bound is the only thing that
noticed, 259 seconds later and four queues away.

⚠ The bound behaved exactly as its ruling requires — a red there is STUCK, never "the box was
busy". It was not raised, and grok did not re-run it. Both correct.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | my own no-args run, field by field | ✅ |
| 2 | my own `500 4 3 4096 true` | ✅ |
| 3 | `fill-depth=[500/0]`×4, and 100/250/1000 | ✅ |
| 4 | `500 … false` → `[0/1][0/0][0/2][2/3]` | ✅ **discriminates** |
| 5 | `arm=6` vs `arm=0`; fill+arm+drain do not double-count | ✅ |
| 6 | reproduced independently at n=2000 | ❌ **real red — see above** |
| 7 | `(:wat::i64::* n m)` at `:2049` and `:2058`, both commented | ✅ |
| 8 | `grep ':cap'` → `:1904` still literal `64` | ✅ |
| 9 | `:2130 :2135 :2141 :2148` all pass `32 false` | ✅ |
| 10 | inside the floor | ✅ |
| 11 | my own run, Summary line read | ✅ |

## What comes next, named

1. **The ack ladder** — three strikes then silence, with no redelivery net. This is the stone in
   front of everything: the curve cannot extend past n=1000 until it is fixed, and a lost ack
   currently has no way to announce itself.
2. **The 25 % slope** from n=100 to n=1000, now known not to be the poller.
3. **`collect` grows with n** — 3.7 s at n=100, 4.6 s at n=1000. Named and ignored, as briefed.
