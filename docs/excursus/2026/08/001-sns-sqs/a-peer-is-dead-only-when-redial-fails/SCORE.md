# SCORE — a peer is dead only when redial fails

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me on a quiet box.

```
Summary [ 362.754s] 5213 tests run: 5213 passed (3 slow), 15 skipped
FLOOR=0        .floor/2026-09-04T02-25-52Z/        my own run, 0 FAIL/TIMEOUT lines
```

## ★ THE PAIR HELD — the fault is survived AND death is still detectable

**Row 1 — the system survives a real sever.** Five runs, my own:

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12     ×5
```

**And it is non-vacuous, which I checked rather than assumed.** Every `/start` in `circuit.wat` is
process locus — including `seenh` at `:932`, which carries the 256-byte frame cap — and the poison
is `200 × "xxxxxxxxxx"` = **2000 bytes against a 256-byte cap**. The sever genuinely happens, twice
per poisoned worker (`poison` → `Lost`, `poison2` → `Closed`), and the worker redials and threads
the fresh peer back in. All twelve workers finish.

**Row 2 — the wall still fires.** 3/3, against a genuinely dead process-echo:

```
queue: redial failed — peer is dead, not a broken pipe
```

Neither row counts alone. Row 1 could be passed by deleting every assertion; row 2 by changing
nothing. **Together they say the recovery is real and falsifiable.**

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ survives a real sever | ✅ 5/5 `dup=0`, poison non-vacuous (2000 B vs 256 B cap, process locus verified) |
| 2 | ★★ the wall still fires | ✅ 3/3, message verbatim above |
| 3 | ★ `Closed` arms converted | ✅ **0 / 0 / 0** genuine fatal `Closed` remaining |
| 4 | `Stopped` untouched | ✅ zero **existing** arms changed |
| 5 | `wat/service.wat` untouched | ✅ absent from the diff |
| 6 | no retry limit invented | ✅ |
| 7 | the premise probe still green | ✅ `CLOSED-IS-RECOVERABLE` |
| 8 | the invariant | ✅ `dup=0`, all five |
| 9 | throughput | ⚠ one of five above my band — **and my band is the problem.** See below |
| 10 | the floor | ✅ **5213/5213, my run** |

## ⛔ THE FIRST FLOOR WAS RED, AND IT WAS CAPTURED — this is the process working

`.floor/2026-09-04T01-53-16Z/ARM.txt`, `5210 passed, 3 failed`, whole ARM kept:

```
FAIL probe_arc278_sane_circuit::receive_calls_are_not_triple_the_messages
FAIL probe_ex001_fanout::fanout_compute_is_complete_and_lossless
FAIL probe_arc278_sane_circuit::redelivery_is_absorbed_by_the_consumer
```

Cause: **thread locus does not enforce the frame cap the way process does** — the poison lands as
`Message` and no sever occurs. The code now says so in place and refuses to assert either way:
*"Do not assert either way: a missing sever is not death."*

★ **That is the fourth locus asymmetry this arc** — after the duration-0 timer, the
`Closed`-vs-`TIMED-OUT` divergence, and the coerce-arms-green-at-thread finding. **S28.** Four
independent instances is no longer a curiosity; it is a property of the substrate that the
IPC-stands-in-for-the-network model assumes away.

The red was captured whole and not re-run away. That is the doctrine paying for itself.

## Three false positives — all in MY grep heuristics, in one grading pass

1. **My census said 17 fatal `Closed` arms; the truth was 16.** `sqs.wat` was 4 fatal + 1 arm
   returning nil, not 5/5. My `grep -A2 … | grep -c assertion-failed!` counted a *neighbouring*
   arm's assertion as belonging to the `Closed` arm.
2. **The remaining count read `0/0/1`, and the 1 was a false neighbour** — grok caught it. That
   `Closed` arm actually redials (`(Tuple (dial-store) empty-envs)`); the `assertion-failed!` two
   lines down belongs to a different match arm (`queue.take: scan-index failed`). The true remainder
   is **0/0/0**.
3. **Row 4's check reported 2 changed `Stopped` lines.** Both were `+` lines inside the *new* poison
   block — exhaustive match arms in added code. No existing arm moved.

Every one of these was my instrument mis-reading a form by proximity. The BRIEF's standing line —
*"the count you find is the fact"* — caught the first; grok caught the second; reading the diff
instead of counting it caught the third.

## ⚠ ROW 9 — the band expired, and that is a method finding

```
mine   pre-B   25582–26735      post-B  26449–27388
       post-D  26750–27177      now     27013–27594     ← one run above the 27.4 ceiling
grok   post-B  25591–26437      post-D  25802–26288      now  25795–26880
```

**My windows have drifted monotonically upward across ~5 hours; grok's have not moved.** The band was
measured *pre-Stone-B* and used as a gate four stones later on a box that has since drifted ~+1.5 s.

★ **A perf band measured hours ago on a drifting box is not an instrument.** That is
*a finding expires when the regime changes*, applied to my own gate — the regime that changed was the
measuring box, not the code. The fix is to **re-baseline on the grading box per stone**, not once per
campaign.

Separately, the poison genuinely adds work — a 2 KB claim, a sever, and a redial per poisoned worker.
With this data the drift and the real cost are **not separable**, so I claim neither. **S29.**

## Still open

- **Chaos (3c/3d)** — **the gate is down.** `Closed` recovers, the waits report, the consumer is
  idempotent, and a real sever has been survived end-to-end.
- **Stone D2** — `accept!`, `face-start-tw`, `nap-ms`'s six homes, the `do-receive` merge.
- **Stone C** — `Alarm :delay`, `Milliseconds`. Last.
- **S15**–**S29**, incl. **S26** (`Stopped` → clean exit), **S27** (the reactor's own arms),
  **S28** (four locus asymmetries), **S29** (the expired band).
