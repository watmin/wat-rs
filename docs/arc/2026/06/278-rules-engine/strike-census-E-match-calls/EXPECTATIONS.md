# EXPECTATIONS — census E

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the bump precedes the guard | `census_count("match:calls")` is above `let pat = alpha_pattern(cond)?;` |
| 2 ★ | before/after reported | `match:calls` on the fanout world, both sides of the move — **0 → 0 expected; nonzero is STOP-1, and a finding** |
| 3 ★ | the parallel is now true | both `matcher.rs` and `compiled_cond.rs` say both counters bump before their guards, and say what the parallel is for |
| 4 ★ | no behaviour change | one statement moved; no branch, no signature, no new counter |
| 5 | `interp_calls` untouched | still a hand-counted local in `alpha_discrimination.rs` |
| 6 | `fanout_cost.rs` guards intact | `prod:derivations == 40_000` survives verbatim |
| 7 | floor | **≥ 5465 — tell me the final number and what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 2 is the whole point: the move either proves the world is clean or surfaces
an interpreter entry nobody could see.**

## Trap doors, named in advance

- **Re-pinning a moved number.** If `match:calls` goes nonzero, that is the discovery — reporting
  it is the deliverable, not adjusting the test until it is quiet.
- **Renaming instead of moving.** Rejected in DESIGN: it would leave the documented parallel dead.
- **Claiming the widening is observed when no world shows it.** If 0 → 0 everywhere, say the
  widening is unobserved; do not imply the fix demonstrated anything it did not.
- **Touching `interp_calls`** — it is deliberately a hand count, independent of the census.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A nonzero `match:calls` quietly absorbed into an updated expectation.
- A rename.
- Any behaviour change.
- A before/after that reports only one side.
