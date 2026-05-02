# Arc 132 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning sonnet, BEFORE deliverable.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** `crates/wat-macros/src/lib.rs` modifications +
~150-word report.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single Rust-file diff | Only `crates/wat-macros/src/lib.rs` modified for the substrate change. May include up to 5 `.wat` test files with explicit `:time-limit` annotations added IF workspace tests legitimately need >200ms. |
| 2 | DEFAULT_TIME_LIMIT_MS = 200 | The `const DEFAULT_TIME_LIMIT_MS: u64 = 200;` is present. Value is the literal 200 (no other choice). |
| 3 | If-else collapses | The `if let Some(ms) = site.time_limit_ms { ... } else { ... }` retires; one unified emission path with `unwrap_or(DEFAULT_TIME_LIMIT_MS)`. |
| 4 | Wrapper shape preserved | The post-collapse wrapper has the same shape as today's with-`:time-limit` path: thread::spawn with `let __wat_handle =`, `recv_timeout`, split `Err(Timeout)` / `Err(Disconnected)` → `resume_unwind` (arc 129). |
| 5 | Workspace tests stay green | `cargo test --release --workspace` exit=0. Tests that needed >200ms have explicit `:time-limit "<longer>"` annotations added. |
| 6 | At most 5 wat-test edits | If sonnet adds explicit annotations to >5 wat-test files in this slice, STOP and report — slice 2 should handle the sweep instead. |
| 7 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 8 | Honest report | Report includes the exact final shape of the emission, the workspace impact (count of tests needing annotation), honest deltas. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | ~5-15 LOC change in `lib.rs`. Mostly deletion (else branch). >30 LOC = re-evaluate. |
| 10 | Comment update | The block comment above the body emission is updated to reflect the new universal-wrapper semantic. |
| 11 | No new public API | No `pub fn`, no new exports. |
| 12 | Workspace test runtime | `cargo test --release --workspace` total time stays under 90s post-change. |

## Independent prediction

- **Most likely (~70%):** all 8 hard + 3-4 soft pass. Most
  tests run in single-digit ms; 200ms is plenty of headroom.
  0-2 tests need explicit annotations.
- **Some-test-fallout (~20%):** 3-5 tests need explicit
  annotations (hermetic-fork tests with real I/O latency).
  All 8 hard still pass; LOC slightly higher.
- **Many-timeouts (~8%):** >5 tests timeout. Sonnet stops
  per row 6; slice 2 is needed for the sweep. Hard row 5
  fails as a result.
- **Edge case in macro emission (~2%):** the arc 129
  resume_unwind path doesn't apply correctly to the
  default-on shape (some interaction we didn't anticipate).
  Surfaces another arc.

## Methodology

After agent reports back:

1. Read this file FIRST.
2. Score each row with concrete evidence.
3. `git diff --stat` should show 1 Rust file + ≤5 wat files.
4. `grep -n "DEFAULT_TIME_LIMIT_MS" crates/wat-macros/src/lib.rs`.
5. Verify the wrapper shape preserves arc 129's split-arms.
6. Score; commit SCORE-SLICE-1.md as a sibling.

## Why arc 132 matters

Arc 132 is the SAFETY-NET arc that complements the
structural-rule arcs (117, 126, 131). Together they make
deadlock-class failures unmissable:
- Arc 117 catches scope-deadlock at compile time
- Arc 126 catches channel-pair-deadlock at compile time
- Arc 131 catches HandlePool-as-Sender at compile time
- **Arc 132 catches ALL OTHER hangs at runtime via 200ms guard**

Belt + suspenders. The substrate ships honest defaults; the
discipline is structural; future authors get fast feedback
regardless of which check (or no check) catches their bug.
