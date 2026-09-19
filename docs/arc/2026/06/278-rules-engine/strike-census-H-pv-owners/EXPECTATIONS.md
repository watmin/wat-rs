# EXPECTATIONS — census H

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | HEAD measured first | `calls`, `total`, `mean`, `arm` quoted as they stand **before** the assertion |
| 2 ★ | the tripwire is armed | `assert_eq!(total, 0, …)` present, carrying the existing LATENT CLIFF text |
| 3 ★ | the non-vacuity is written down | the one-`#[cfg(test)]`-block argument recorded AT the assertion |
| 4 ★ | that non-vacuity is proved live | delete `merge:pv-calls` → `assert_eq!(calls, STRATA)` REDs; quote it; restore |
| 5 ★ | rename + site facts | `merge:pv-owners-sum`; the increment site states "sum across calls" and "0 means Tree, not zero owners" |
| 6 | no split | no separate Array/Tree counters added |
| 7 | floor | **≥ 5465 — final number and what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is what stops row 2 being a tautology**, and row 4 is what stops row 3
being a claim.

## Trap doors, named in advance

- **`assert_eq!(total, 0)` as an unfalsifiable-by-deletion gate.** Delete the bump and it still
  passes. The paired-block argument is the whole defence and it must be written where the assertion
  is, not only in this strike's docs (`[[derive-what-your-own-measurement-implies]]`).
- **Asserting whatever HEAD happens to show.** If `total != 0`, that is STOP-1 — a finding, not a
  value to pin.
- **Splitting the arms.** Census G's lesson: a name nobody reads is not an improvement.
- **Losing the cliff text.** It is the failure message; it points the next reader at
  `strat_merge_cow_parts`.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- An assertion added without the HEAD measurement quoted first.
- The non-vacuity argument left in the strike docs rather than at the assertion.
- Row 4 skipped or driven against a copy.
- `merge:pv-owners` still emitted.
