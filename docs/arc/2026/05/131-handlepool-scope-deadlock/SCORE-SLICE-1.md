# Arc 131 Slice 1 — Score against pre-handoff expectations

**Written:** 2026-05-01, AFTER reading sonnet's report.

**Agent ID:** `a68c82d6a34fc0b18`
**Agent runtime:** 287 seconds (~4.8 min)

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single-file diff | **PASS** | `git diff --stat src/check.rs` shows 133 insertions, 5 deletions. No other Rust files. No `.wat` files modified by sonnet (the `.wat` modifications visible in `git status` are from prior sessions / user recovery, not arc 131). |
| 2 | New HandlePool surface arm | **PASS** | `src/check.rs:2011-2028`, mirrors the existing Channel/Sender match shape; recurses into args; returns Some("HandlePool") if any arg contains a Sender. |
| 3 | New "HandlePool" offending_kind | **PASS** | Literal string "HandlePool" returned by the new arm. Existing Display arm (`src/check.rs:391-400`) interpolates it correctly via `{}`. |
| 4 | Doc-comment retired | **PASS** | `src/check.rs:1990-1998` updated; the "future arc" caveat is replaced with text explaining arc 131's check. |
| 5 | Two new unit tests | **PASS** | `arc_131_handlepool_with_sender_fires` at line 11206; `arc_131_handlepool_without_sender_silent` at line 11258. |
| 6 | Unit tests pass | **PASS** | `cargo test --release -p wat --lib check`: 49 passed, 0 failed (was 47 + 2 new arc 131 tests). |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications; no commit, no push. |
| 8 | Honest report | **PASS+** | Report includes: file:line refs, exact final form of new arm, unit-test count + pass status, workspace-failure prediction (16 `.wat` files use HandlePool::pop + 14 use ::Spawn typealiases; estimate 14-20 wat-tests need refactor in slice 2), honest delta surfaced (parametric typealias didn't work due to lexer whitespace constraint; pivoted to direct types — same logical coverage). |

**HARD VERDICT: 8 OF 8 PASS. Clean ship.**

## Soft scorecard (5 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 9 | LOC budget | **PASS (within band)** | ~110 LOC total; predicted 50-100 ideal, 150 ceiling. Slightly over the ideal but well under the ceiling. The two unit tests are ~50 LOC each due to the wat-source they construct. |
| 10 | Display arm refinement | **PASS (no change needed)** | Sonnet decided not to update the Display arm — the existing `(a {})` interpolation reads correctly with "(a HandlePool)". Honest reasoning surfaced in the report. |
| 11 | Pattern fidelity | **PASS** | The new arm mirrors the existing Channel/Sender surface match style: `if head.as_str() == "wat::kernel::HandlePool" { ... return Some("HandlePool"); }`. |
| 12 | No new public API | **PASS** | No `pub fn`, no exports, no new types. |
| 13 | Failure-prediction quality | **PASS+** | Sonnet ran a `grep` survey: 16 wat-test files use `HandlePool::pop`; 14 use `::Spawn` typealiases. Concrete count of consumer-sweep scope. |

**SOFT VERDICT: 5 OF 5 PASS.** No drift.

## Independent prediction calibration

The orchestrator predicted (in `EXPECTATIONS-SLICE-1.md`):

- 65% all 8 hard + 4-5 soft pass cleanly.
- 20% all hard pass + one soft drift.
- 10% unit-test construction difficulty.
- 5% recursion-ordering bug.

**Actual:** all 8 hard + 5 soft pass. The "unit-test
construction difficulty" path PARTIALLY fired — sonnet's
first attempt at parametric typealias unit tests hit the
lexer's whitespace-in-keyword constraint. Sonnet pivoted to
direct types in the unit tests, exercising the same
recursion path. The resilience is honest: prediction said
sonnet might struggle; sonnet did struggle but recovered.

Sweep timing: 4.8 min. Continuing the trend of compressing
sweep durations (sweep 1: 13.5 min; reland: 7 min; arc 130
sweep: killed; arc 129: 2.5 min; arc 131: 4.8 min). The
artifacts keep teaching.

## What this scores tells us

- Discipline scales to substrate-extension arcs. Arc 117's
  parent + the source-comment caveat + arc 117's
  INSCRIPTION together teach what's needed.
- The "workspace failures expected" framing in the brief
  worked as intended. Sonnet didn't try to fix the failing
  service tests; correctly punted to slice 2.
- The grep-based failure prediction is calibrated data for
  slice 2: 14-20 tests = real scope, not infinite scope.

## Next steps

1. **Slice 2: consumer sweep** of the 14-20 service tests
   that fire the new check. Refactor each to inner-let*
   nesting per the canonical fix in
   `SERVICE-PROGRAMS.md § "The lockstep"`. Spawn next sonnet.
2. **Slice 3: closure** — INSCRIPTION + WAT-CHEATSHEET §10
   update + cross-references to arc 117.
3. **Prove the fire** — the unit tests in src/check.rs
   already prove the rule fires structurally. The arc-130
   LRU test, with the diagnostic block probe still in
   working tree, would also fire `ScopeDeadlock` at freeze
   time post-arc-131. Verify when slice 2 + arc 132 land.

## What's pending

The workspace can't be committed cleanly until slice 2's
sweep retires the firings on existing service tests. Slice 1
stays in working tree. Slice 2 is the unblocker.

After slice 2: commit arc 131 slices 1+2 together; workspace
green. Arc 132 (default time-limit) follows naturally as a
sibling safety net.
