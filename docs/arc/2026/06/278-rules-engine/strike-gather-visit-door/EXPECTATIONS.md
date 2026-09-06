# EXPECTATIONS — the gather-visit door

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the three sites count | every element examined by a gather bumps the instrument |
| 2 ★ | the ratio is reported BOTH ways | `small`/`big`/ratio before and after, verbatim |
| 3 ★ | the lint exists and is MUTATION-PROVED | re-introduce a raw bucket walk → RED; restore → green. Quote both |
| 4 ★ | the lint has a non-vacuity guard | its file list cannot go silently empty |
| 5 | the two O(1) reads are NOT instrumented | and the SCORE says why — they examine nothing |
| 6 | no engine change | same facts, same rows |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |
| 9 | cost | no `*_cost` gate moved, or the delta surfaced |

★ load-bearing. **Row 2 is the one that decides whether a second strike follows.**

## Trap doors, named in advance

- **Raising `2.0` to keep green.** STOP-1. The threshold is not the finding.
- **A lint that cannot redden.** Row 3. This arc has shipped two gates that could not fail; do not
  make it three.
- **Instrumenting `bucket.len()` / `is_empty()`** to make the count look complete. They examine
  nothing; counting them would inflate the very quantity the gate divides.
- **An iterator wrapper that costs.** STOP-3 — `census_gather_visit` compiles out in release, the
  wrapper may not.
- **Re-run the floor at FINAL state.** Five gates have fired unexpectedly across this session.

## What would make me reject the result

- The threshold raised instead of a crossing reported.
- A lint that cannot be reddened.
- The two O(1) reads instrumented.
- A behaviour change in the engine.
- A red floor of any size.
