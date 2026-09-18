# EXPECTATIONS — predicted gather visits

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | equality asserted per path | visits **==** the keyed prediction, not a bound |
| 2 ★ | ≥3 `(G,W)` points per path | a formula fitting one point is a coincidence |
| 3 ★ | the equality reddens where the ratio does not | simulated whole-memory regression: ratio PASSES, equality FAILS. **Quote both** |
| 4 ★ | which assertion is the proof, said plainly | the equality; the ratio is a second reading |
| 5 | no fudge terms | no constant added to make a formula fit |
| 6 | no engine change | zero lines in `src/rete/kernel/fire/` |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is the deliverable.**

## Trap doors, named in advance

- **A formula fitted to the readings.** Predictions come from the mechanism (each token probes its
  own bucket), not from curve-fitting the numbers we happen to see. STOP-2.
- **Dropping the ratio.** It models shapes the formula does not; demote it, do not delete it.
- **Claiming the equality proves more than it does.** It proves visits match the keyed prediction on
  the driven paths at the driven points. It does not prove the folds not yet driven.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A bound where an equality was asked for.
- One `(G,W)` point per path.
- Row 3 unattempted, or the equality failing to redden under the simulated regression.
- A fudge constant.
- A red floor of any size.
