# EXPECTATIONS — excursus 001 stone 7: the fan-out proof

**Written BEFORE the strike, 2026-08-31.** Blast radius derived from the BRIEF's own section.

## ⚠ Two outcomes are both valid deliveries

- **The proof passes** — N×M outcomes, all worker ids present, zero duplicates. The composition
  is demonstrated and the actor's serialization becomes a measured fact.
- **Duplicates appear** — ★ **a FINDING and a successful stone.** STOP-2 forbids fixing it.

The failure mode is neither: a green summary from a circuit whose workers never actually ran in
parallel, or a property "proven" with a sleep.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | `Outcome` lifted into `:messages` | pure move, no rename, no call-site churn (as stone 6) |
| 2 | circuit freezes | `--check` → **0** (it is `1` today) |
| 3 | the workaround is GONE | no `read-foreign` / `ForeignRecord/get` in the drain |
| 4 | ★ the summary is NON-ZERO | `total`, `distinct`, `workers`, `empty` all > 0 and internally consistent |
| 5 | ★ fan-out completeness | `total = N × M` |
| 6 | no loss | every queue's final `receive` is empty |
| 7 | ★ parallelism by ids | all `M×J` worker ids present — **no clock, no sleep** |
| 8 | ★ duplicate count reported | whatever it is. Zero is a result; non-zero is a finding |
| 9 | one queue service per queue | read the wiring (STOP-1) |
| 10 | workers are processes | `:locus (:wat::spawn::process)` |
| 11 | standalone at weight | N=2000/M=4/J=3 → 8000 outcomes, or the number it broke at |
| 12 | floor | **`FLOOR=0`** |
| 13 | blast radius | `fanout/` + one `.rs` + SCORE. topic/, queue/, substrate untouched |
| 14 | prior stones | topic `"3 3"`; queue `bound=x;…`; both stone-5 repros still `--check = 1` |

## Runtime prediction

**2–4 hours.** Jobs 1 and 2 are mechanical — the lift is the third of its kind and the
workaround deletes to two accessor calls. **Job 3 is the work**, and it is the first time the
circuit will actually run end to end.

## Trap-doors

1. **★ Row 4 is the honesty gate.** Stone 4 reported `total=0` and correctly called `dup=0`
   vacuous. **A zero summary must never be reported as a pass** — if the circuit still produces
   zeros, that is the deliverable, and the duplicate count remains vacuous.
2. **★ Removing the workaround may reveal, not fix.** It was written around a real blockage;
   deleting it could expose the next thing rather than complete the path. **Do not assume the
   silent-zero path (`None → ""` → ack nothing) was the cause** — it is a candidate, unmeasured.
3. **A third instance of the beside-the-surface defect.** Two were found one stone apart because
   a type-check halts at the first error. **The current red may not be the last one** — if a
   third appears after fixing `Outcome`, that is expected, not a surprise.
4. **Redelivery vs duplication.** A worker slower than the visibility window will *correctly*
   see its message again. If the summary cannot tell that from a genuine double-delivery, row 8's
   number is meaningless. This was stone 4's subtlest trap and it is unchanged.
5. **18 processes on 12 cores** is oversubscribed by design — fine for correctness, bad for
   wall-clock. Do not read slowness as a defect, and do not assert on timing.

## Not in this stone

- Any change to `wat-topic`, `wat-queue`, or the substrate — if one is needed, that is a finding.
- Promotion of either to `wat/`.
- Dead-letter queues, retry limits, FIFO ordering.
