# EXPECTATIONS — census E reachability

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the number | `wc -l` of the tripwire file, quoted. **Either value is a PASS** — this is a measurement |
| 2 ★ | floor GREEN with the tripwire in | the measurement is void otherwise; quote the Summary line |
| 3 ★ | `matcher.rs` clean at the end | `git diff --quiet` — modulo at most one comment sentence |
| 4 ★ | the wording, if zero | "never observed across the full suite", dated, with the floor named — **not** "cannot happen" |
| 5 | if non-zero, verbatim | `sort \| uniq -c` output, uninterpreted |
| 6 | no corpus or test change | nothing added anywhere to make a case exist |
| 7 | floor after revert | **≥ 5465 — final number and what moved it** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 1 has no expected value on purpose.** A measurement strike whose scorecard
predicts its own answer is not a measurement.

## Trap doors, named in advance

- **Treating zero as proof of impossibility.** It is evidence of non-occurrence across one suite.
  Row 4 exists because the wording is the deliverable.
- **Chasing the callers if non-zero.** STOP-2 — report and stop; attribution is a separate question
  with its own evidence.
- **Deleting the `None` branch on a zero.** A guard that never fires is not dead code
  (`[[a-guard-that-never-fires-is-not-dead-code]]`); it handles a malformed `cond`.
- **Leaving the tripwire in.** The last strike left `matcher.rs` mutated; row 3 is why.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A number without the floor that produced it.
- "Cannot happen" phrasing on a zero result.
- Any corpus or fixture addition.
- A tripwire still in the tree.
