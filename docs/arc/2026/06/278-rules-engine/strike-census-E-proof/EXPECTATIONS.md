# EXPECTATIONS — the census E proof

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | **the mutation REDs** | revert E's move → the `match:calls` assertion fails, both numbers quoted verbatim |
| 2 ★ | restored and green | move restored, test passes, quote the passing numbers |
| 3 ★ | all three assertions present | `match:calls == interp_calls`, `compiled:exec == calls`, `calls == interp_calls` |
| 4 ★ | the numbers are real | report `calls`, `interp_calls`, and both counter readings — not "passed" |
| 5 | doc row added | the test's doc names the new row and what it proves |
| 6 | nothing else touched | `alpha_discrimination.rs` only; `matcher.rs` byte-identical to HEAD at the end |
| 7 | floor | **≥ 5465 — tell me the final number and what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 1 is the entire strike.** Three green assertions without it prove nothing —
that is precisely the state census E is already in.

## Trap doors, named in advance

- **A gate with one possible outcome.** If every `cond` in the corpus is an alpha pattern, the
  assertion holds before and after E's move and proves nothing. Row 1 is the only thing that can
  tell the difference (`[[derive-what-your-own-measurement-implies]]`).
- **Forcing the red by adding facts.** If the mutation stays green, that is a finding to report,
  not a corpus to edit quietly. STOP-1.
- **Asserting on a copy.** Drive the LIVE test with the LIVE engine change reverted — not a
  duplicated loop (`[[a-mutation-proof-must-drive-the-gate-not-a-copy]]`).
- **Leaving `matcher.rs` mutated.** Row 6: it must be byte-identical to HEAD when you finish.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A green mutation reported as success.
- Assertions added without the mutation run.
- Numbers described rather than quoted.
- `matcher.rs` differing from HEAD at the end.
