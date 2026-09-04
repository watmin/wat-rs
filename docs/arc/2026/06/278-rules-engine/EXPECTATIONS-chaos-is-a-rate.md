# EXPECTATIONS — chaos is a rate

Written **before** the strike. Re-run by me on a quiet box.

⚠ **Re-baseline the circuit on the grading box before comparing anything** — S29: my windows drift
upward with session time while the executor's do not, so a band measured hours ago is not an
instrument. Take five runs at the default rate first; that is the comparison set.

| # | what | expected |
|---|---|---|
| 1 | ★★ **rate 0 arms nothing** | with the default, **no `-disrupt` alarm is armed at all** — not armed-and-inert. Show it |
| 2 | ★★ **chaos is a rate** | with a rate set, `-disrupt` fires **many times** across one run. Report the count. **One firing fails the stone** |
| 3 | ★★ **the seed replays** | two runs, same seed → **same disruption count, same points**. Chaos that cannot replay cannot be debugged |
| 4 | ★ **the invariant under chaos** | `total=8000; distinct=8000` with the rate on. ⛔ **`dup > 0` is a FINDING, not a failure** — report the number, do not tune it |
| 5 | the fresh peer is threaded | no infinite `Closed` loop; no run that hangs. Trap 3 is a rejection criterion |
| 6 | process locus only | no thread-locus test demands a tear; no second mechanism invented |
| 7 | 3c-pre's poison not copied | worker start is not unconditionally poisoned |
| 8 | scope | no `wat/service.wat`, no `src/`, no 3d |
| 9 | every outcome named | no `-1`/`-2` collapse in new code |
| 10 | the floor | `5213/5213` at the default rate |

## ⛔ ROW 4 IS THE ONE I EXPECT TO SURPRISE US

The circuit has reported `dup=0` through every stone of this campaign. **R69 records that this was
partly a blind spot**: `distinct` keys on `queue/envelope-id`, which a retry *replaces*, so a
genuine redelivery raised `total` and `distinct` together and `dup` stayed 0 by construction. The
idempotent consumer (`:fanout::Seen`) now keys on the published seq, which is the honest key.

So under real chaos, one of three things happens, and **all three are results**:

- **`dup=0`** — the consumer absorbed every redelivery. The strongest outcome, and the one that
  finally *earns* the number instead of inheriting it from a reliable transport.
- **`dup > 0`** — at-least-once producing exactly what it is defined to produce. A finding.
- **`distinct < 8000`** — ⛔ **loss.** That is the only genuine failure on this row, and it is the
  one worth the whole stone.

**Do not weaken the assertion to reach any of them.**

## RUNTIME PREDICTION

**75–120 minutes.** The arm is small and every mechanism is proven; the time goes into the re-arm
loop and threading the seed through state without losing it on the disrupt path — which is exactly
where the bug would be.

## TRAP-DOOR RISKS

1. **Threading the seed through the DISRUPT branch.** The hit path rebuilds state with a fresh peer;
   dropping the advanced seed there silently makes the sequence non-advancing — every draw identical,
   the rate wrong, and row 3 passing for the wrong reason.
2. **The re-arm must happen on BOTH branches** — hit and miss. Re-arming only on a hit means chaos
   stops the first time it does not fire.
3. **The oversized frame needs a target with a small `:max-frame-bytes`.** My probe's first two
   attempts failed because a *normal* call already exceeded the cap I chose; a cap that is too small
   breaks the service outright and looks like the disruptor working.
4. **The grant rides `post-spawn`.** Miss it and the very first call fails and nothing about chaos is
   being measured. Already paid for once.
5. **`rand::int-from` is `[lo, hi)`** — a delay window of `(0, 0)` is empty, and `Millisecond 0` has
   no form after Stone A.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 with a count of 1. That is the probe.
- Row 3 asserted rather than shown as two matching runs.
- Row 1 satisfied by an alarm that fires and returns early — the row is **no alarm armed**.
- `dup > 0` reported as a failure and tuned away, or an assertion weakened to reach `dup=0`.
- A run that hangs. After five stones removing unfalsifiable hangs, a hang is the worst outcome
  available and is strictly worse than a red.
