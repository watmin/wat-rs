# EXPECTATIONS — production_delta's entry hoist

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the BEFORE is measured and reported | a real count on the fanout cell, before any change |
| 2 ★ | the count drops to per-node | after: one `entry` per deriving production node. Both numbers quoted |
| 3 ★ | the gate is a FORMULA | in the axis parameters, not a recorded integer |
| 4 ★ | behaviour identical, INCLUDING ORDER | same facts, same rows, same sequence |
| 5 | the explain index untouched | `idx.entry(derived)` keyed on a varying value stays |
| 6 | no wall-clock claim | counts only |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 1 is the deliverable's foundation — if the before number surprises, say so.**

## Trap doors, named in advance

- **Confirming the DESIGN instead of measuring it.** The last one was wrong by 3×. Report what the
  counter says, not what the brief predicted.
- **Order.** Buffering and flushing can reorder `wm.production`'s vectors relative to
  `wm.derived_facts`. STOP-2.
- **Holding the `&mut` open** — the explain arm takes `&wm` whole. STOP-3.
- **A pinned integer instead of a formula.**
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A before number that was assumed rather than read.
- A millisecond presented as the result.
- Any fact-level or ordering difference.
- A red floor of any size.
