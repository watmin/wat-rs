# EXPECTATIONS — census I

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | rename complete | `grep -rn 'seed:mixed-class-activate' src/ tests/` → **no hits** |
| 2 ★ | the mutation REDs | delete the bump → `assert_eq!(activated, 3)` fails `0 != 3`; quote verbatim; **name the arm and line**; restore |
| 3 ★ | units stated at the sites | two comment lines: `batch-class-*` per CLASS, this one per FACT of a mixed class |
| 4 | siblings untouched | `seed:batch-class-uniform` / `-mixed` literals and counts unchanged |
| 5 | severity reported honestly | the SCORE says latent — no number was wrong, the consumer was already correct |
| 6 | floor | **≥ 5465 — final number and what moved it.** New arms are a PASS; name them |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 2 is required precisely because it is available** — census D and G had no red
and reporting that was right; here one exists, so its absence would be a gap.

## Trap doors, named in advance

- **Inflating the severity.** The audit says "benign but real" and so should the SCORE. A mild row
  written up as a near-miss is its own kind of false report.
- **Renaming a correct sibling for symmetry.**
- **Naming the arm the brief predicted rather than the one that fired**
  (`[[one-mutation-cannot-prove-a-multi-arm-gate]]`).
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- `seed:mixed-class-activate` still present anywhere.
- The mutation skipped, or driven against a copy.
- A sibling changed.
- Severity written up as worse than it is.
