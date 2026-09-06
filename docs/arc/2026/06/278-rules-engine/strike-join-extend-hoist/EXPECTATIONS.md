# EXPECTATIONS — hoisting join_extend's per-alpha triple

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | lookups are COUNTED | a `#[cfg(test)]` counter at the three sites; a before reading on a real axis |
| 2 ★ | the count DROPS to per-node | after: `3 × join nodes`, not `3 × emitted pairs`. Both numbers quoted |
| 3 ★ | the gate is a FORMULA | predicted count in the axis parameters, not a recorded integer |
| 4 ★ | behaviour identical | same facts, same rows; differential and oracle green |
| 5 | no hasher change | zero edits to a map's type |
| 6 | no wall-clock claim | the SCORE reports counts, not milliseconds |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 2 is the deliverable, row 4 is the one that must not bend.**

## Trap doors, named in advance

- **Claiming a speedup.** Six samples or no number, and none are asked for. A lookup count is the
  claim; a millisecond is not.
- **A recorded integer instead of a formula.** `[[gate-the-ratio-not-the-millisecond]]` — the count
  must be derived from the axis, so it stays true when the fixture changes.
- **The borrow shape.** `FireCtx` exists to hold split borrows; if the hoist forces a wider borrow
  that costs more, that is STOP-3, not a thing to absorb.
- **A hasher swap riding along.** STOP-4. It changes iteration order, and this engine has already
  shipped an order-dependent defect (F1).
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A millisecond presented as the result.
- A pinned integer where a formula belongs.
- Any fact-level difference.
- A hasher changed.
- A red floor of any size.
