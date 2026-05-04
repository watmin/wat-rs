# Arc 144 Slice 4 — Pre-handoff expectations

**Drafted 2026-05-03 post-arc-146-closure.** Small verification
slice. New test file mirrors existing arc 144 harness shape.

**Brief:** `BRIEF-SLICE-4.md`
**Output:** NEW `tests/wat_arc144_uniform_reflection.rs`. NO Rust
substrate edits. NO wat substrate edits. NO new primitives.

## Setup — workspace state pre-spawn

- Arc 144 slices 1, 2, 3 shipped. Slice 3 ended Mode B-canary;
  the length canary stayed RED with a precise diagnostic.
- Arc 146 slices 1-5 shipped (closed today). Length canary now
  GREEN per arc 146 slice 2.
- Arc 148 + arc 150 + arc 132 amend + TypeScheme inline-field
  cleanup all shipped.
- Workspace failure profile (per FM 9 baseline): 1832 passed /
  5 failed (only documented arc 130 + pre-existing panicking-
  test noise).

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | NEW `tests/wat_arc144_uniform_reflection.rs` ONLY. NO src/ edits. NO wat/ edits. NO other test file edits. |
| 2 | Test count | 6-10 integration tests covering the 6 Binding kinds + length canary regression. Don't duplicate existing arc 144 / arc 146 coverage; if an existing test already covers a kind sufficiently, REFERENCE it in a comment + skip the duplicate. |
| 3 | UserFunction kind covered | Test verifies `:wat::runtime::lookup-define` returns Some for a user-defined function + emission carries `:wat::core::define`. |
| 4 | Macro kind covered | Same shape for `:wat::core::defmacro`. |
| 5 | Primitive kind covered | Same shape for a TypeScheme primitive (e.g., `:wat::core::foldl`); Primitive variant returns Some + signature-of returns Some. |
| 6 | SpecialForm kind covered | Same shape for `:wat::core::if`; signature-of returns Some + emission carries the slice-2 sketch. |
| 7 | Type kind covered | Same shape for a user-defined struct; emission carries `:wat::core::struct`. |
| 8 | Dispatch kind covered | Same shape for `:wat::core::length` (arc 146 Dispatch entity); emission carries `:wat::core::define-dispatch` + arms list. |
| 9 | Length canary regression test | A test in the new file replicates the arc 143 slice 6 shape (`define-alias :user::size :wat::core::length` + use it on a HashMap; assert correct length result). |
| 10 | All 9 baseline tests still pass + workspace unchanged | Per § 7 FM 9 — re-run all baselines pre-shipping. No new failures. |

**Hard verdict:** all 10 must pass. Rows 3-8 are the load-bearing
rows (uniform reflection coverage). Row 9 protects the most
recently-restored canary.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | 50-200 LOC test file. >300 LOC = consider whether duplicate coverage crept in. |
| 12 | Style consistency | Mirror `tests/wat_arc144_lookup_form.rs`'s harness pattern (the `run` helper + line-by-line assertions on `:write-line` output). |
| 13 | clippy clean | No new warnings. |
| 14 | Audit-first discipline | Sonnet read existing arc 144 + arc 146 test files BEFORE writing; surfaces redundancy IF found (don't duplicate); reports any unexpected substrate friction as honest delta. |

## Independent prediction

- **Most likely (~70%) — Mode A clean ship.** Pattern is solidly
  trodden by arcs 144 slice 1 + arc 146 slice 1's test files;
  sonnet mirrors. Length canary already passes. ~10-15 min wall-
  clock.
- **Mode B-substrate-gap (~15%):** sonnet finds that one of the 6
  kinds doesn't actually return Some via lookup-form (e.g.,
  SpecialForm path may have edge cases the slice 2 SCORE didn't
  cover). Honest STOP at first red; surface gap; orchestrator
  decides whether to fix substrate or rescope test.
- **Mode B-coverage-rollup (~10%):** sonnet finds existing arc 146
  / arc 144 tests already cover most kinds; slice 4's net new is
  smaller than expected. Honest delta; ship the gap-coverage
  tests + reference existing tests in comments.
- **Mode C (~5%):** unforeseen edge — e.g., a Binding kind has
  variant-specific reflection behavior the brief didn't anticipate.

## Time-box

30 min wall-clock (≈2× the predicted upper-bound of 15 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

**Arc 144 slice 5** — closure paperwork. INSCRIPTION + 058 row +
USER-GUIDE entry + ZERO-MUTEX cross-ref. Shape mirrors arc 146
slice 5 + arc 148 slice 6.

**Arc 109 v1 closure trajectory** — another major chain link
closes.

## After sonnet completes

- Re-read brief assumptions against the SCORE
- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows by running tests + spot-checking the
  new test assertions
- Write `SCORE-SLICE-4.md`
- Commit BEFORE drafting slice 5's BRIEF (calibration preserved)

## What's notable about this slice's shape

This is the **smallest substantive sonnet sweep in the cascade so
far.** Pure verification; no substrate edits. The brief's pre-
flight crawl checklist is the substantive work — sonnet has to
read existing arc 144 + arc 146 test files to avoid duplicate
coverage. Calibration data point: how does sonnet handle a
"verify what already works" brief vs "build what's new"?
