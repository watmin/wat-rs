# Arc 131 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning sonnet and BEFORE its
deliverable. Durable scorecard.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** `src/check.rs` modifications + ~150-word report.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff | Only `src/check.rs` modified. No `.wat` files. No other Rust files. No documentation. |
| 2 | New HandlePool surface arm | `type_contains_sender_kind` adds `wat::kernel::HandlePool` arm. The arm recurses into args; if any arg contains a Sender (via existing recursion), returns `Some("HandlePool")`. Otherwise returns None. |
| 3 | New offending_kind value | The Display arm + diagnostic code emit `"HandlePool"` (literal string, no spaces, no hyphens) when the surface match fires on HandlePool. |
| 4 | Doc-comment retired | The `:wat::kernel::HandlePool<T>` "future arc" caveat at lines 1990-1994 is replaced with text explaining the new check. |
| 5 | Two new unit tests | `arc_131_handlepool_with_sender_fires` + `arc_131_handlepool_without_sender_silent` present in `src/check.rs::tests`. |
| 6 | Unit tests pass | `cargo test --release -p wat --lib check` exit=0; new tests pass; existing arc 117 tests pass. |
| 7 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 8 | Honest report | Report includes file:line refs + the exact final form of the new arm + unit test count + workspace failure prediction (count of existing tests likely to fire). |

**Hard verdict:** all 8 must pass for slice 1 to ship clean.
Workspace test failures from existing service tests are
EXPECTED data for slice 2's sweep; row 6 only requires the
`-p wat --lib check` unit tests to pass.

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | ~15-30 LOC change in `src/check.rs` for the surface arm + doc-comment update. ~30-60 LOC for the two unit tests. Total ~50-100 LOC. >150 LOC = re-evaluate. |
| 10 | Display arm refinement | Optional: sonnet may update the Display arm to make HandlePool kind more specific in the message. Either choice is fine; surface in the report. |
| 11 | Pattern fidelity | The new arm mirrors the existing surface match pattern (`matches!(head.as_str(), ...)`) — consistent style. |
| 12 | No new public API | No `pub fn`, no new types, no new exports. |
| 13 | Failure-prediction quality | Sonnet's report includes a `grep`-based estimate of how many existing tests fire the new check (Console + telemetry + service-template + arc 130's LRU). Rough count is fine; the goal is calibration for slice 2's scope. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~65%):** all 8 hard + 4-5 soft pass cleanly.
  The change is small + well-scoped + has a clear precedent
  (the existing Channel/Sender match arm). Sonnet ships in
  3-7 min.
- **Second-most-likely (~20%):** all 8 hard pass, one soft
  drift (e.g. Display arm wording, or LOC slightly over).
- **Unit test construction difficulty (~10%):** the unit tests
  need to construct a wat source with HandlePool + custom
  typealias chains. Sonnet may struggle with the exact
  TypeExpr shape, returning unit tests that don't quite fire
  the check correctly. Surface in the report; iterate.
- **Recursion ordering bug (~5%):** the new arm's
  `is_some()` recursive check might race with the existing
  alias-peel logic for HandlePool itself if HandlePool
  somehow becomes aliased. Unlikely but possible.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards with concrete evidence.
3. Diff via `git diff --stat` (expect 1 file).
4. Verify hard rows 2-5 by `grep -n "HandlePool" src/check.rs`
   and `grep -n "fn arc_131" src/check.rs`.
5. Verify hard row 6 by reading the cargo-test totals from
   the agent's report.
6. Score; commit SCORE-SLICE-1.md as a sibling.

Then PROVE THE FIRE: revert the LRU test's diagnostic block to
its post-arc-130 state (with the deadlock pattern), run the
test, observe ScopeDeadlock fires at freeze with kind
"HandlePool" — the deadlock observed in arc 130 sweep is now
caught at compile time.

## What we learn

- **All 8 hard pass:** discipline scales; arc 117 + 131 jointly
  enforce the structural rule. Slice 2 sweep refactors
  existing service tests; slice 3 verifies on arc 130's case.
- **Row 6 fails:** sonnet's check has a logic bug. Diagnose
  via the failing unit test; iterate.
- **High workspace-failure count (predicted by sonnet):**
  slice 2's scope is bigger than expected. Adjust accordingly.

## Why this slice matters

Arc 131 closes the "future arc" caveat arc 117's author
deliberately marked. The chain:

| # | Arc | Hard | Substrate gap |
|---|---|---|---|
| 1 | arc 126 sweep 1 | 5/6 | arc 128 |
| 2 | arc 126 sweep 2 reland | 14/14 | none |
| 3 | arc 126 sweep 3 | 6/8 | arc 129 |
| 4 | arc 129 sweep 1 | 14/14 | none |
| 5 | arc 130 sweep 1 (killed) | TBD | **arc 131 (this) + arc-130-internal substrate bug** |

Each non-clean sweep names a gap. Arc 131 closes one of arc
130's two surfaced gaps. Arc 130 then needs another sweep
post-arc-131 to close the substrate-internal bug (driver
dies after Put — not a deadlock, a separate runtime defect).
