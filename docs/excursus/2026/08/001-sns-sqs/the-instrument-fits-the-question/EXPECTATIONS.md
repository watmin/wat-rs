# EXPECTATIONS — the instrument fits the question

Written **before** the strike. Re-run by me on a quiet box. The result cannot move these.

## THE BASELINE — the race, reproducible on demand

```
probe-refused-retry-self-consumes.wat, 3/3, pre-stone:
  gap=0    after-drain=none  pending=1  recovered-after-naps=0    would-return
  gap=300  after-drain=got   pending=0  recovered-after-naps=-1   SPINS-FOREVER
```

Circuit, my five runs post-Stone-B: `dup=0` all five, publish `26449 / 26712 / 26932 / 27388 /
26503 ms`.

| # | what | command | expected |
|---|---|---|---|
| 1 | ★★ **the race cannot stall** | `probe-refused-retry-self-consumes.wat` | the `gap=300` cell **cannot report `SPINS-FOREVER`** — `wait-pending` no longer exists to spin |
| 2 | ★★ **the race becomes an assertion** | drive `:user::refused-is-retried` with a 300 ms gap induced | it **fails loudly naming the race**, or passes. **Never stalls.** Report which, and the message verbatim |
| 3 | ★ **no `receive` proves a negative** | grep the helper family for a `receive` whose result is compared to `""` to conclude absence | **zero.** This is the contract decision |
| 4 | ★ **a bounded wait reports** | force one to expire | names **what it last saw** — depth, attempts, elapsed. ⛔ "timed out" alone is the empty ARM again and fails the row |
| 5 | ★ **a dead peer is distinguishable** | kill a queue mid-wait | the helper reports a **failure**, not a depth. `(Tuple 1 1)` must be gone |
| 6 | presence collapsed | `sns-fanout.wat:798,838` | one `Queue/receive` with `:wait (Wait::UpTo …)`; the `wait-*` + `take-one` pairs are gone |
| 7 | absence is non-destructive | `sns-fanout.wat:796,840` | depth reads. Nothing consumed |
| 8 | the cross-queue wait survives as a **bounded poll** | `sns-fanout.wat:793` | still a poll (no wire event for another queue's unacked), now bounded and reporting |
| 9 | `take-one`'s hold is visible | its signature | renamed, **visibility a required argument**. No hardcoded `1000000000000` inside |
| 10 | the sentinel convention | `q-depth`, `depth-of-topic`, `topic-outbox` | one out-of-band convention, stated once, matching `ticks-of`'s existing `-1` |
| 11 | the lying comment | `sns-fanout.wat:145-147` | says what the line under it does, or the line stops inventing |
| 12 | `visible`/`unacked` | `sqs.wat:64-72` + 16 consumers | renamed; `sqs.wat:69`'s comment no longer has to define the names |
| 13 | out of scope untouched | `git diff` | no `accept!`, no `face-start-tw`, no `nap-ms`, no `do-receive` merge |
| 14 | the invariant | circuit, **five runs** | `total=8000; distinct=8000; dup=0` on **all five** |
| 15 | throughput | same five runs | publish **25.5–27.4 s**. Band widened from Stone B's own row-6 finding (one run at 27388 ms) |
| 16 | the floor | `scripts/floor.sh`, **Summary line** | `5213/5213` |

## ⛔ ROW 2 IS THE STONE, AND A LOUD FAILURE IS A PASS

`probe_async_publish::refused_subscriber_is_retried_not_dropped` currently passes **by luck** — three
greens after one 30 s timeout with an empty ARM.

If this stone makes it **fail loudly with a message naming the race**, that is the stone working.
The defect was never "the test is red"; it was **"the test cannot tell you why."** A red that names
its cause is strictly better than a green that is a coin flip.

⛔ **Do not weaken an assertion to reach green.** If the honest outcome is a named failure, ship the
named failure and report it — that is a finding, and it is the one this whole arc has been chasing.

## RUNTIME PREDICTION

**90–150 minutes.** Two semantic rewrites in two files, three irreducible polls to bound, a sentinel
convention, and a rename codemod. The rename is the easy half; the bounds are where the care goes,
because a bound that cannot report is the defect wearing a fix.

## TRAP-DOOR RISKS

1. **`:vis-ns 200000000` and the 350 ms nap are WINDOWs, not bounds.** Per
   `BRIEF-278-a-liveness-bound-only-catches-a-hang.md`: raising a window deletes the scenario.
2. **`sns-fanout.wat:793` is cross-queue** — waits on `inbox`, takes from `subq`. It looks like the
   other two and is not; there is no blocking form for it.
3. **`wait-inbox-zero` and `wait-pending-zero` wait for ZERO.** Departure has no arrival event. Both
   stay polls.
4. **The `visible`/`unacked` rename is a WIRE change** — `StatsResponse` is inside the `defsurface`,
   16 consumers.
5. **Three scratch-pad probes are in the census** and the `every_wat_scripts_file_loads` gate
   type-checks them. Missing one turns that gate red at the end.
6. **`probe-refused-retry-self-consumes.wat` must keep working.** It is the evidence for this stone
   and the record of the mechanism. Its `:rr::run` cells may become unbuildable if `wait-pending`
   dies — if so, **convert them to the bounded form and keep the demonstration**, do not delete it.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 reported as "still passes" with no induced-gap run. Passing by luck is what it did before.
- Row 4 satisfied by a bound that only says "timed out".
- Row 5 not run — the `(1,1)` sentinel is what makes a dead peer look like work.
- Row 1 or 14/15 from fewer than the stated runs.
- An assertion weakened to reach green.
- The reproducer deleted rather than converted.
