# SCORE — the instrument fits the question

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me on a quiet box.

```
Summary [ 362.873s] 5213 tests run: 5213 passed (4 slow), 15 skipped
FLOOR=0        .floor/2026-09-04T00-45-36Z/        my own run, 0 FAIL/TIMEOUT lines
```

## ★ THE RACE IS DEAD, AND THE MEASUREMENT IS THE INVERSION OF THE DEFECT

The reproducer, my run, 3/3:

```
BEFORE   gap=300  after-drain=got  pending=0  recovered-after-naps=-1  SPINS-FOREVER
AFTER    gap=300  after-drain=got  pending=1  recovered-after-naps=0   would-return
```

**`pending` went 0 → 1.** The message is still in the queue, because the absence check no longer eats
it. That single digit is the whole stone.

## ★ ROW 2 — the race is now NAMED instead of silent

Driving the real gate with the gap induced, my run, 3/3:

```
gap=0     inflight=yes; after-drain=none; after-expiry=got      ← the floor path, unchanged
gap=300   inflight=yes; after-drain=got;  after-expiry=got      ← the race, REPORTED
```

At a 300 ms gap the gate returns `after-drain=got`, which fails the Rust assertion
`after-drain == "none"` **loudly, with a value that names what happened.** Before, the same
condition produced a 30 s timeout and an empty ARM.

**The assertion was not weakened.** `stalled_subscriber_does_not_stall_others` still reports
`healthy=got; stalled=held; dt-ms=2; blocked=no`.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ the race cannot stall | ✅ 3/3 — `SPINS-FOREVER` → `would-return`, `pending=1` |
| 2 | ★★ the race becomes an assertion | ✅ 3/3, above |
| 3 | no `receive` proves a negative | ✅ two `Queue/receive` sites remain — `receive-blocking` (presence) and `claim-one!` (a claim). `wait-pending`/`wait-inflight` are **gone** |
| 4 | ★ a bounded wait reports | ✅ `unacked-never-rose: last=0/0 attempts=3 elapsed=4` — depth, attempts, elapsed |
| 5 | ★ a dead peer is distinguishable | ✅ `dead=-1/-1`, and **it short-circuits**: `unacked-unread: last=-1/-1 attempts=0 elapsed=0` |
| 6 | presence collapsed | ✅ `receive-blocking … (Wait::UpTo (Millisecond 2000))` |
| 7 | absence non-destructive | ✅ `after-visible (first (q-depth subq))` |
| 8 | cross-queue wait survives as a bounded poll | ✅ `require! (poll-until-unacked inbox 2000)` |
| 9 | `take-one`'s hold is visible | ✅ **`claim-one! subq "q0" 1000000000000`** — visibility a required argument at the call site |
| 10 | one sentinel convention | ✅ `(-1, -1)`, matching `ticks-of`'s existing `-1` |
| 11 | the lying comment | ✅ |
| 12 | `visible`/`unacked` | ✅ `StatsResponse::Ok _calls _ticks visible unacked` |
| 13 | out of scope untouched | ✅ no `accept!`, `face-start-tw`, `nap-ms`, `do-receive` merge |
| 14 | ★ the invariant | ✅ `total=8000; distinct=8000; dup=0`, **all five runs** |
| 15 | throughput | ✅ 26750–27177 ms, inside the 25.5–27.4 band |
| 16 | the floor | ✅ **5213/5213, my run** |

### Better than specified, twice

**The report gained a third state.** I asked for absence vs presence. It returns `none` / **`unread`** /
`got` — so a *failed read* has its own word instead of masquerading as absence. That is the
`(Tuple 1 1)` lesson applied one level up, at the place a human reads.

**The bounded poll short-circuits on a dead peer** rather than burning its whole budget:
`attempts=0`. A bound that waits 2000 attempts to report an already-known failure would have been
technically compliant and useless.

## ⛔ MY CENSUS WAS WRONG A FIFTH TIME — AND THIS ONE HAD TEETH

I wrote **67 helper occurrences across 5 files**. The finder found **46 keywords in 3 files**.

The command was `grep -rno "<bare token>" | wc -l` — **raw token occurrences, including comments,
prose, and unrelated identifiers.** The previous four cost nothing. This one would have:

```
circuit.wat:655   pending <- :fanout::Hist
circuit.wat:714   :pending (:fanout::hist-add (:fanout::Traces/pending tr) (:fanout::ns->ms t3 t4))
```

**`pending` in `circuit.wat` is a latency-histogram field** — the t3→t4 bucket. My census would have
swept the circuit's telemetry into a queue-semantics rename. The rename correctly applied to
`sqs.wat` only.

★ This is the **form-vs-token** failure, which I have a standing note about and reproduced anyway:
*match the FORM, not the token.* Five census errors this campaign — omitted constructors, an omitted
directory, an empty grep reported as fact, a miscount of my own list, and now a bare-token grep that
would have broken working code. **EXPECTATIONS' standing instruction — "the finder's count is the
fact; mine is a hypothesis" — is the only thing that has made this survivable.**

## ⚠ A MEASUREMENT FINDING: cross-executor throughput comparison is invalid

Three stones of five-run windows:

```
Stone B  baseline (mine)  25582–26735      Stone B  post (grok)  25591–26437
Stone B  post (mine)      26449–27388      Stone D  post (grok)  25802–26288
Stone D  post (mine)      26750–27177
```

**Grok's windows sit consistently below mine, and mine drift upward across the session** — tracking
session time, not stones. That is environmental, and it means a number measured by one executor
cannot be compared to a number measured by the other. **Only within-executor deltas count.** Recorded
so the next perf row does not read a box difference as a code change. **S25.**

## ⚠ WHAT THIS STONE DID NOT FIX — the assertion is still timing-coupled

`after-drain` asserts *"nothing has been delivered yet"* at a moment whose truth depends on timing.
It is a **NEGATIVE ASSERTION coupled to its WINDOW** in the taxonomy of
`BRIEF-278-a-liveness-bound-only-catches-a-hang.md`.

This stone converted **an unfalsifiable hang into a falsifiable assertion.** That is a real and large
improvement — but under a sufficiently loaded floor the gate can now go **red**, loudly, for a
reason it names. Per the doctrine that is a red like any other, and the right end state is an
assertion that cannot race at all. **S24**, and it is only reachable *because* the failure is now
visible.

## Still open

- **Chaos (3c/3d)** — **readable for the first time.** Next.
- **Stone D2** — `accept!` (publishes, retries unboundedly, rewrites the payload), `face-start-tw`,
  `nap-ms`'s six homes, the `do-receive` merge.
- **Stone C** — `Alarm :delay`, `Milliseconds`. Last; closes no defect.
- **S15**–**S25**, incl. **S24** (the timing-coupled assertion) and **S25** (the measurement offset).
