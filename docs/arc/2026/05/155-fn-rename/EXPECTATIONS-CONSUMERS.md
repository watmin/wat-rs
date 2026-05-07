# Arc 155 — Consumer Sweep EXPECTATIONS (slice 1b)

**Drafted 2026-05-06 evening.**

**Brief:** `BRIEF-CONSUMERS.md`
**Output:** EDITS to consumer wat + embedded wat. NO commits.
**Model:** `model: "sonnet"` explicit per FM 12.

## Setup

- HEAD: `072f1e0`
- Working tree dirty with sweep 1a substrate (5 files: 3 src + types.rs + new test)
- Pre-baseline: 1085 BareLegacyLambda firings + 0 BareLegacyLowercaseFn (define-param sites bypass walker per slice 1a's honest delta) + 69 downstream panics

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Workspace 0-failed | `cargo test --release --workspace` returns 0 failed |
| 2 | Arc155 tests 12/12 | `cargo test --release --test wat_arc155_fn_rename` shows 12 passed |
| 3 | Operator-position sweep complete | `grep ':wat::core::lambda' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/` returns 0 source spellings |
| 4 | Type-position sweep complete | `grep ':fn(' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/` returns 0 source spellings |
| 5 | Embedded sites included | `tests/*.rs` and `src/*.rs` embedded wat strings swept |
| 6 | No substrate edits | No changes to `src/check.rs`, `src/runtime.rs`, `src/special_forms.rs`, `src/types.rs` (sweep 1a's territory) |
| 7 | No commits | HEAD unchanged from `072f1e0` |
| 8 | Sweep order respected | stdlib first |
| 9 | No unexpected substrate red | Only Mode-A diagnostic kinds; NO panics outside intentional thread-panic tests; NO unrelated TypeMismatch |
| 10 | Honest report | Per BRIEF reporting requirements |

**Hard verdict:** all 10 must hold. Rows 1, 3, 4, 9 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | Iteration count | 4-8 cargo test runs to converge |
| 12 | clippy clean | No new clippy warnings |
| 13 | Phase A walker convergence | BareLegacyLambda count → 0 before Phase B begins |
| 14 | No grinding | No site required >3 iterations |

## Independent prediction

- **Most likely (~65%) — Mode A clean.** Hybrid sweep is well-
  scoped (walker-driven for operator + grep-driven for type);
  ~476 sites; mechanical 1:1 transforms. ~30-50 min Sonnet
  wall-clock.
- **Mode B-substrate-bug (~5%):** sweep 1a's `:wat::core::Fn(`
  parser recognition has an edge case.
- **Mode C-grep-context (~15%):** the Phase B grep `:fn(` may
  match contexts other than type expressions (e.g., docstrings
  with example wat code, comments, etc.); sonnet handles per-site
  classification.
- **Mode D-grinding (~5%):** few sites take >3 iterations.
- **Mode B-time-violation (~10%):** doesn't complete in 90 min.

## Time-box

90 minutes wall-clock.

## What success unlocks

**Mode A clean:** workspace 0-failed; orchestrator atomically
commits sweep 1a + 1b; arc 155 slice 2 closure (orchestrator-side
per discipline) ships next.

## After sonnet completes

- Read this file FIRST
- Score each row
- Verify load-bearing rows by re-running:
  - `cargo test --release --workspace` (0 failed)
  - `cargo test --release --test wat_arc155_fn_rename` (12/12)
  - `grep -rln ':wat::core::lambda'` and `grep -rln ':fn('`
- Sample 3-5 transformed sites
- THEN: orchestrator atomically commits sweep 1a + 1b
