# EXPECTATIONS — census K/L

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | K amended, measurement kept | the 2026-08-01 reading marked historical + named as the cause of the children's removal; today's two-per-pass shape stated; the column and its argument survive |
| 2 ★ | L scoped correctly | claim held for STEP2 **and CATCHUP**, denied for MAINTAINER with its guard as the reason |
| 3 ★ | the audit corrected | the SCORE says the audit's *"neither can ever emit a 0 row"* was half wrong, and quotes CATCHUP's `n_all.saturating_sub(already)` |
| 4 ★ | evidence quoted | the `alpha:*` grep (two marks, line numbers) and each site's `n` argument |
| 5 | comments only | `git diff` touches no line outside a comment |
| 6 | severity honest | doc-only; nothing was at risk; no mutation invented |
| 7 | floor | **≥ 5465 — final number and what moved it** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is the one worth the strike**: a work-list row that was checked and came
back partly wrong gets amended, not quietly worked around.

## Trap doors, named in advance

- **Deleting the 2026-08-01 measurement** because its magnitude is stale. It is why the column
  exists; a column whose justification is gone is the next thing someone deletes.
- **Making MAINTAINER uniform** by moving its call outside the guard. Rejected — nothing reads it,
  and it is an engine edit for an instrument's benefit.
- **Inventing a mutation for a comment change.**
- **Repeating the audit's error** by writing "neither site can emit 0" into the new comment.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- The historical measurement deleted rather than dated.
- L's new comment still denying CATCHUP the zero row.
- Any non-comment line changed.
- A fabricated mutation proof.
