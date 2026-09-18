# EXPECTATIONS — root_join_delta batching

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | both counters, separable, with denominators | span entries and `record_token` calls per elements seeded |
| 2 ★ | token writes batched | three hash ops per `(node, child)`, not per element. Both numbers quoted |
| 3 ★ | ORDER proven unchanged | explicitly argued, not assumed — name what reads `beta`/`d_beta`/`match_pool` mid-nest |
| 4 ★ | the span branch DECIDED by its count | hoisted with numbers, or refuted with numbers |
| 5 | gate is a formula | in the axis parameters, not a recorded integer |
| 6 | no wall-clock claim | counts only |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is the one that decides whether this ships at all.**

## Trap doors, named in advance

- **Assuming the span branch is hot.** I have been wrong on this exact question twice, in opposite
  directions. Count it.
- **Order.** `production_delta` was safe because nothing read `wm.production` mid-nest. That is NOT
  given here — later passes read `beta` and `d_beta`. STOP-1.
- **`push_match`** mutates `match_pool` per element inside the loop. STOP-2.
- **Hoisting a cold branch** — churn on the engine's most element-dense loop for no measured gain.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A batch landed without an explicit order argument.
- The span branch hoisted on an assumption rather than a count.
- A pinned integer where a formula belongs.
- Any fact-level or ordering difference.
- A red floor of any size.
