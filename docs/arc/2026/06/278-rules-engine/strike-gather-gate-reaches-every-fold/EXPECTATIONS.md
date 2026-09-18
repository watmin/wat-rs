# EXPECTATIONS — the keyed-gather gate over every fold

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | every instrumented path is entered | `Distinct\|All\|GroupBy` and the mapping no-`SeedCmp` arm both register visits > 0 |
| 2 ★ | the ratio is reported PER PATH | not one aggregate; visits per fold at both `G=10 W=80` and `G=80 W=10` |
| 3 ★ | the old axis is unperturbed | `800/800` still, or the difference explained |
| 4 ★ | the verdict is stated either way | "holds across the surface" with numbers, or "crosses at fold X" with numbers |
| 5 | no engine change | zero lines in `src/rete/kernel/fire/` |
| 6 | floor | **0 failed** |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 4 is the deliverable — a crossing is a successful strike.**

## Trap doors, named in advance

- **Raising `2.0`.** The threshold is not the variable. STOP-1.
- **A fixture too small to discriminate.** F1 needed eight producers where two agreed and proved
  nothing. If a fold reads flat, widen before concluding.
- **Aggregating the folds into one number** — then a scaling fold hides behind a flat one.
- **Curing a crossing inside this strike.** It is the next strike, drawn from these numbers.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A threshold moved.
- One aggregate number instead of a per-path split.
- A fold silently skipped as undrivable.
- An engine change.
- A red floor of any size.
