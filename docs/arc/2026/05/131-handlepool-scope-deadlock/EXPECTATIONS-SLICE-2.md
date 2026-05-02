# Arc 131 Slice 2 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning sonnet, BEFORE
deliverable.

**Brief:** `BRIEF-SLICE-2.md`
**Output:** N `.wat` test file refactors + ~200-word report.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Test-only diff | All modifications in `.wat` files under `wat-tests/` or `crates/*/wat-tests/`. No Rust files. No documentation. No substrate `.wat` files (`wat/` or `crates/*/wat/`). |
| 2 | Inner-let* pattern applied | Each modified file's outer let* binds only the Thread (or Threads); the inner let* owns state + pool + handle + work; inner returns the Thread. Mirrors SERVICE-PROGRAMS.md § "The lockstep". |
| 3 | `:should-panic` preserved | Tests with `:should-panic("channel-pair-deadlock")` annotations (LRU CacheService.wat, HolonLRU step3-6, step-B) keep the annotation. The substring still matches because arc 126 still fires (the helper-verb signature is unchanged). |
| 4 | **Workspace test green** | `cargo test --release --workspace` exit=0. All previously-passing tests still pass; no regressions. |
| 5 | File count in band | 14-25 files modified (sonnet's prediction was 14-20; allow some slack). >25 = stop and report. |
| 6 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 7 | No semantic changes | Test logic preserved; only binding-scope nesting changes. Each test's assertions, expected values, and side-effect checkpoints are identical pre/post refactor. |
| 8 | Honest report | 200-word report with file list + count, exact final form of one refactored test, workspace totals, honest deltas. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC delta | Mostly net-zero or slight increase (extra `(:wat::core::let* (...) ...)` wrapping). Total file delta in 100-500 LOC range. >1000 LOC = re-evaluate scope. |
| 10 | Multi-service handling | Files with multiple services (e.g. Console + LRU together) handle joint Thread ownership cleanly. Sonnet describes the chosen shape. |
| 11 | Workspace runtime | `cargo test --release --workspace` total time stays <90s. The refactors shouldn't change runtime characteristics. |
| 12 | No false-positive `:should-panic` | Sonnet doesn't add `:should-panic` annotations to tests that don't need them. Console + telemetry tests should pass cleanly post-refactor (no annotation needed). |

## Independent prediction

Before reading the agent's output:

- **Most likely (~50%):** all 8 hard + 3-4 soft pass. Sonnet
  refactors 16-20 files in 8-15 min wall-clock; workspace
  green.
- **Multi-service complexity (~25%):** all 8 hard pass, soft
  drift on row 10 — sonnet picks slightly different shapes
  for files with multiple services (Console+telemetry,
  Console+LRU). Outcome still committable.
- **Test-semantics drift (~15%):** sonnet inadvertently
  changes a test's behavior during refactor (e.g. moves a
  side-effect outside the inner scope). Workspace fails;
  needs surgical fix on the affected test.
- **Scope explosion (~7%):** >25 files surface as needing
  refactor. Sonnet stops per row 5; we re-scope.
- **Type-checker surprise (~3%):** the inner-let* shape
  changes some tuple-destructure typing in a way that fails
  type-check. Surfaces as compile error; iterate.

## Methodology

After agent reports back:

1. Read this file FIRST.
2. Score each row with concrete evidence.
3. `git diff --stat` should show only `.wat` test files
   (count in band).
4. Verify hard row 4 by reading workspace test totals.
5. Verify hard row 3 by `grep` on `:should-panic` lines.
6. Score; commit SCORE-SLICE-2.md.
7. Commit arc 131 slices 1+2 together (workspace now green).

## Why this slice matters

Slice 2 is the FIRST large consumer-sweep arc in the failure-
engineering chain. Previous sweeps were ≤6 files. If sonnet
ships 16-20 files clean, the artifacts-as-teaching discipline
is validated for substrate-wide structural enforcement —
the substrate teaches via the new check; consumers adapt
via the canonical fix; both directions propagate cleanly.

If slice 2 doesn't ship clean: the issue is either the brief
(too vague for large refactors), the canonical-fix shape
(real edge cases the doc doesn't capture), or both. Either
informs the next iteration.

## What follows

After arc 131 slices 1+2 commit:
- Arc 132 spawns (default 200ms time-limit).
- Arc 132 sweep handles any newly-exposed timeouts (≤5 sites).
- Arc 132 commits.
- Workspace fully green; the deadlock-class chain ships.
