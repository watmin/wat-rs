# EXPECTATIONS — `filter:test-pass` names one quantity

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | one key, one quantity | the reuse arm no longer bumps `filter:test-pass`; every increment site carries a doc naming what it counts |
| 2 ★ | `test-pass ⊆ test-evals` | true by construction after the split — state where that is now guaranteed |
| 3 ★ | no gate asserts an identity | `0.0 < 50.0` is gone: either driven on an axis with `evals > 0`, or an explicit refusal |
| 4 ★ | the refusal is DRIVEN | if you take the refusal route, a control test drives it and requires the panic — describing it is not proving it |
| 5 | numbers moved, and named | every changed census number reported with before/after |
| 6 | no engine change | same facts, same rows, same fires |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is the point of the strike.**

## Trap doors, named in advance

- **Tuning a threshold to keep green.** STOP-3. This is the failure the strike exists to remove.
- **A second consumer of the union.** STOP-2 — and it is a finding, not a nuisance.
- **Splitting the key but leaving the vacuous assertions.** Then the census is honest and the gate
  still proves nothing; row 3 is what makes this worth doing.
- **Re-run the floor at FINAL state.** Five gates have fired unexpectedly across this session.

## What would make me reject the result

- Any assertion that is still an arithmetic identity.
- A threshold moved instead of a gate fixed.
- A behaviour change in the engine.
- A red floor of any size.
