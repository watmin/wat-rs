# EXPECTATIONS — census B

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the key is split | `grep -rn '"compiled:calls"' src/ tests/` → **no hits** outside docs/history |
| 2 ★ | both mutations RED | delete each bump in turn, drive the LIVE probe, quote both REDs verbatim |
| 3 ★ | the new assertions have content | report the non-zero counts on the arms where they are non-zero, so `exec_built == 0` / `elided_empty == 0` are not vacuous |
| 4 ★ | no behaviour change | no engine value moves; `skip_span`'s condition is untouched |
| 5 ★ | the four false sentences are true | `compiled_cond.rs:955`, `accum_cost.rs:44`, `:93` message, `accum_alpha_cost.rs:1365` |
| 6 | C10 respected | `accum_cost.rs:85-90` still present and still accurate |
| 7 | floor | **≥ 5465 — tell me the final number and exactly what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 2 is the proof the split kept both sites gated; row 3 is the proof the new
assertions can fail.**

## Trap doors, named in advance

- **Vacuous new assertions.** `exec_built == 0` passes trivially if the probe stops entering the
  path at all. Row 3 exists to catch that — an equality over two zeros proves nothing
  (`[[derive-what-your-own-measurement-implies]]`).
- **Touching `skip_span`.** C10 forbids it and the DESIGN's whole argument is that this strike does
  not need to.
- **Leaving `:1345` and `:1365` both standing.** They contradict each other; the strike's job is
  that only the true one remains.
- **Re-pinning 80,200** under either new name.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A mutation proof that drove a copy of the gate instead of the live one.
- `compiled:calls` still emitted anywhere in code.
- Any engine behaviour change.
- New assertions that cannot be shown to have content.
