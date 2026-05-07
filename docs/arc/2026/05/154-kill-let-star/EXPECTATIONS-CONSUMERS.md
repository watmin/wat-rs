# Arc 154 — Consumer Sweep EXPECTATIONS (slice 1b)

**Drafted 2026-05-06 evening.**

**Brief:** `BRIEF-CONSUMERS.md`
**Output:** EDITS to consumer wat files + embedded wat in
`tests/*.rs` + `src/*.rs` lib tests. NO commits.

## Setup

- HEAD: `bd27820`
- Working tree dirty with sweep 1a substrate (4 files)
- Pre-baseline: ~1260 BareLegacyLetStar walker fires + ~72 downstream panics

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Workspace 0-failed | `cargo test --release --workspace` returns 0 failed |
| 2 | Arc154 tests 10/10 | All 10 tests in `tests/wat_arc154_kill_let_star.rs` pass (the 7 currently-blocked positive tests unblock when stdlib is clean) |
| 3 | Type-position sweep complete | `grep ':wat::core::let\*' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/` returns 0 source spellings |
| 4 | Embedded sites included | `tests/*.rs` and `src/*.rs` embedded wat strings swept |
| 5 | No substrate edits | No changes to `src/check.rs`, `src/runtime.rs`, `src/special_forms.rs` (sweep 1a's territory) — embedded wat strings INSIDE these files OK |
| 6 | No commits | HEAD unchanged from `bd27820` |
| 7 | Sweep order respected | stdlib (`wat/*.wat`) migrated before tests |
| 8 | No unexpected substrate red | Only Mode-A diagnostic kinds: BareLegacyLetStar (drives sweep) + pre-existing intentional thread-panic tests; NO substrate panics, NO unrelated TypeMismatch |
| 9 | Honest report | Per BRIEF reporting requirements |
| 10 | Time-box honored | <= 120 min |

**Hard verdict:** all 10 must hold. Rows 1, 3, 4, 8 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | Iteration count | 4-8 cargo test runs to convergence |
| 12 | clippy clean | No new clippy warnings |
| 13 | LOC delta neutral-ish | Each transform `:wat::core::let*` → `:wat::core::let` saves 1 char per site (~827 saves) |
| 14 | No grinding | No site required >3 iterations |

## Independent prediction

- **Most likely (~70%) — Mode A clean.** Substrate-as-teacher
  loop established over arc 109/153/etc.; per-site transform is
  pure mechanical (no value-position classification needed —
  unlike arc 153's `()` sweep). ~80-100 min wall-clock.
- **Mode B-substrate-bug (~5%):** edge case in sweep 1a's
  walker fires on a non-let-star context.
- **Mode C-unexpected-shape (~10%):** a class of `:wat::core::let*`
  references in unusual contexts (e.g., reflection / quasiquote /
  AST-as-data) where the transform needs care. Surface gap.
- **Mode D-grinding (~5%):** a few sites take >3 iterations.
- **Mode B-time-violation (~10%):** sweep doesn't complete in
  120 min (~827 sites is heavier than arc 153's ~455).

## Time-box

120 minutes wall-clock. ScheduleWakeup at T+120 min.

## What success unlocks

**Mode A clean:** workspace 0-failed; orchestrator atomically
commits sweep 1a + 1b + arc 154 SCORE/INSCRIPTION at slice 2;
arc 154 slice 2 closure paperwork (orchestrator-side per
discipline) ships next.

**Mode B/C/D:** surface gap; orchestrator adjusts brief.

## After sonnet completes

- Read this file FIRST
- Score each row
- Verify load-bearing rows by re-running:
  - `cargo test --release --workspace` (0 failed)
  - `cargo test --release --test wat_arc154_kill_let_star` (10/10)
  - `grep -rn ':wat::core::let\*'` (0 source spellings)
- Sample 3-5 transformed sites to verify post-rename reads
  cleanly
- THEN: orchestrator atomically commits sweep 1a + 1b together

## Why this matters

User direction 2026-05-06 evening: *"new arc - let's do it."*
Sweep 1b completes arc 154's vocabulary collapse. After atomic
commit + slice 2 closure: wat-rs ships its third foundation mark
of the day (`let` joins `nil` and `do` as today's vocabulary
landings). The Lisp on Rust's user-facing surface keeps
consolidating toward what the user is building towards.
