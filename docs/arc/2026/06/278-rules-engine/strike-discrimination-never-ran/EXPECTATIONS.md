# EXPECTATIONS — the discrimination row

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the no-op is PROVEN first | `assert_ne!` on the unfixed rewrite goes RED. Quote it |
| 2 ★ | the rewrite targets the rule's constant | `:alpha` / `:probe::E::A`, not the other fact's value |
| 3 ★ | the `if` becomes `assert_ne!` | matching `:1654` four lines up |
| 4 ★ | `raw_count(&never)` REPORTED | the number, for both fixtures, whatever it is |
| 5 | no engine change | `reachability.rs` only |
| 6 | floor | **0 failed** |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 1 is the evidence the row never ran; row 4 is the deliverable.**

## Trap doors, named in advance

- **Fixing the rewrite without first proving it was inert.** Then "it never ran" is a claim, not a
  demonstration. Row 1 exists for that.
- **Adjusting the expectation to whatever comes back.** If it is not `Ok(0)`, that is a finding, not
  a number to accommodate. STOP-1.
- **Sweeping other `if`-guarded rewrites.** Name them. STOP-3.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- The fix landed without the RED that proves the row was dead.
- An expectation tuned to the observed count.
- Any engine change.
- A red floor of any size.
