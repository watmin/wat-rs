# Arc 126 Slice 1 RELAND — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning the reland sonnet agent
and BEFORE its first deliverable. Durable scorecard for the
second attempt at slice 1, post arc 128.

**Reland trigger:** previous attempt
(`a37104bfc10e4c6fa`) scored 5/6 hard rows pass; row 3
(workspace green) failed because the new check fired on
deftest bodies inside `run-sandboxed-hermetic-ast` forms-blocks
at outer freeze. Arc 128 closed the substrate gap (boundary
guard in `walk_for_deadlock`); the reland inherits the guard
from inception in `walk_for_pair_deadlock`.

**Reland brief:** `BRIEF-SLICE-1-RELAND.md`
**Output channel:** `src/check.rs` modifications + ~150-word
written report.

## Hard scorecard (must-pass)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Diagnostic substring | The literal `channel-pair-deadlock` appears verbatim in the Display impl. |
| 2 | Single-file diff | Only `src/check.rs`. No `.wat` files. No other Rust files. |
| 3 | **Workspace green** | `cargo test --release --workspace` exits 0. **Same 7 ignored tests as pre-reland (6 deadlock-class + 1 mechanism); no new failures, no new ignores.** |
| 4 | Arc 117 reuse | Reuses `parse_binding_for_typed_check`, `expand_alias`, `type_contains_sender_kind` (or sibling parser if signature mismatch — same justification bar as before). |
| 5 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 6 | Honest report | 150-word report with file:line refs + test count + workspace totals + exact panic message. |
| **7 — NEW** | **Sandbox-boundary guard inherited** | `walk_for_pair_deadlock` includes arc 128's guard verbatim (the four-keyword `matches!` + `skip(2)` recurse). Discoverable via `grep "run-sandboxed-hermetic-ast" src/check.rs` returning hits in BOTH walkers. |
| **8 — NEW** | **Boundary unit test added** | A 5th unit test (`channel_pair_deadlock_skipped_in_sandboxed_forms` or similar) verifies the channel-pair anti-pattern wrapped in `run-sandboxed-hermetic-ast` does NOT fire `ChannelPairDeadlock`. Mirror arc 128's existing test pattern. |

**8-row hard scorecard.** All must pass for the reland to be
intact.

## Soft scorecard (signals)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | 150-300 LOC band ideal; 300-600 acceptable given Display verbosity (sweep 1 came in at 560). >700 = re-evaluate. |
| 10 | Function quartet | The five DESIGN-named functions present (`validate_channel_pair_deadlock`, `walk_for_pair_deadlock`, `check_call_for_pair_deadlock`, `trace_to_pair_anchor`, `type_is_receiver_kind`). |
| 11 | Unit tests covering 5 cases | All 4 original cases + the new boundary test. |
| 12 | False-negative honesty | DESIGN's caveats either confirmed or honestly tightened/loosened. |
| 13 | No new public surface | All new fns are `fn` (private), not `pub fn`. |
| 14 | No env/config flag | The check always runs. |

## What we learn — discipline calibration

If **all 8 hard rows + most soft rows pass**: the substrate-as-
teacher discipline is FULLY intact. The first sweep surfaced a
substrate gap; arc 128 closed it; the reland integrates cleanly.
Future arc-117/arc-126-shaped rules can ship via brief + DESIGN
+ existing precedents.

If **hard row 7 (boundary guard) fails**: the BRIEF-RELAND was
underspecified. The amendment didn't make "MUST mirror this
verbatim" forceful enough. Diagnose: was the canonical guard
shown clearly? Was the read-in-order list followed?

If **hard row 3 (workspace green) fails again**: a deeper
substrate issue exists. The boundary guard worked for arc 117;
something in arc 126's check fires through a different code path
(maybe a non-let* form that the boundary guard doesn't cover).
Reland-of-reland needed; this would be a real surprise.

If **all hard pass**: arc 126 slice 1 is committable. Slice 2
follows in a separate session — convert the 6 `:ignore` to
`:should-panic(expected = "channel-pair-deadlock")`.

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~75%):** all 8 hard rows pass; 5-6 soft rows
  pass; LOC ~580-650 (similar to sweep 1, plus the boundary
  guard + new test); discipline intact.
- **Second-most likely (~20%):** all hard rows pass; one soft
  drift (function naming or LOC). Sweep 1's pattern is well-
  templated; small variations expected.
- **Failure mode (~5%):** sonnet misses the boundary-guard
  inheritance row 7 because it didn't read arc 128's
  INSCRIPTION first. Brief amendment 1 should prevent this; if
  it doesn't, the brief's read-in-order strictness needs
  hardening.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST, before the agent's output.
2. Score each row of both scorecards explicitly.
3. Diff the agent's claimed file edits against `git diff
   --stat`.
4. Verify hard row 7 by `grep` for the boundary keywords in
   both walker functions.
5. Verify hard row 8 by listing the unit tests added.
6. Diagnose any failure with reference to the BRIEF or DESIGN.
7. Land the SCORE document as a sibling.

This document is the calibration record for the reland. The
original `EXPECTATIONS-SLICE-1.md` stays as the first-attempt
record.
