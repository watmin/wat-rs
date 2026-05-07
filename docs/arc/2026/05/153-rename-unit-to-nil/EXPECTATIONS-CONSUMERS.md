# Arc 153 — Consumer Sweep EXPECTATIONS (slice 1b)

**Drafted 2026-05-06.**

**Brief:** `BRIEF-CONSUMERS.md`
**Output:** EDITS to consumer wat files + embedded wat in
`tests/*.rs` and `src/*.rs` lib tests. NO commits.

## Setup

- HEAD: `dbe72e1`
- Working tree dirty with sweep 1a substrate (4 files)
- Pre-baseline: ~35 BareLegacyUnitName + ~69 downstream panics

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Workspace 0-failed | `cargo test --release --workspace` returns 0 failed |
| 2 | Arc153 tests 10/10 | `cargo test --release --test wat_arc153_nil_rename` shows 10 passed |
| 3 | Type-position sweep complete | `grep ':wat::core::unit' wat/ wat-tests/ crates/ examples/ tests/ src/` returns 0 source spellings (or only intentional historical comments) |
| 4 | Value-position sweep complete | `()` value-position sites all migrated; only type-position parens + intentional vector-literal contexts remain |
| 5 | No substrate edits | No changes to `src/check.rs`, `src/runtime.rs`, `src/types.rs` (sweep 1a's territory) — embedded wat strings INSIDE these files are allowed; substrate Rust code is not |
| 6 | No commits | HEAD unchanged from `dbe72e1` |
| 7 | Sweep order respected | stdlib (`wat/*.wat`) migrated before tests |
| 8 | No unexpected substrate red | Only Mode-A diagnostic kinds surfaced; no panics, no unrelated TypeMismatch |
| 9 | Honest report | Per BRIEF reporting (sweep summary, iteration cycles, latent bugs, path, deltas) |
| 10 | Time-box honored | <= 120 min |

**Hard verdict:** all 10 must hold. Rows 1, 3, 4, 8 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | Iteration count | 4-10 cargo test runs to converge. >15 = grinding signal. |
| 12 | clippy clean | No new clippy warnings on modified files |
| 13 | LOC delta net negative | Each transform `:wat::core::unit` → `:wat::core::nil` saves 1 char; `()` → `:wat::core::nil` adds chars (so net depends on ratio) |
| 14 | No grinding | No site required >3 iterations to converge |

## Independent prediction

- **Most likely (~55%) — Mode A clean.** Substrate-as-teacher
  loop is established; per-site transform is mechanical;
  workspace converges cleanly. ~80-100 min wall-clock.
- **Mode B-substrate-bug (~5%):** edge case in sweep 1a's
  value-position recognition surfaces under heavier consumer use.
- **Mode C-unexpected-shape (~20%):** value-position `()`
  classification is harder than predicted — many sites need
  judgment calls (was this `()` intended as unit value or as
  empty-list / empty-vector?). Sonnet surfaces a class of
  ambiguous sites; orchestrator may need to rule on a few.
- **Mode D-grinding (~10%):** a few sites take >3 iterations
  due to unusual nesting / parametric containment.
- **Mode B-time-violation (~10%):** sweep doesn't complete in
  120 min (heavier than predicted; many embedded wat strings
  in lib tests may surprise).

## Time-box

120 minutes wall-clock. ScheduleWakeup at T+120 min.

## What success unlocks

**Mode A clean:** workspace 0-failed; orchestrator commits sweep
1a + 1b + arc 153 SCORE docs atomically; arc 153 slice 2
closure ships next; arc 136 slice 2 closure runs after.

**Mode B/C/D:** surface gap; orchestrator adjusts brief.

## After sonnet completes

- Read this file FIRST.
- Score each row.
- Verify load-bearing rows by re-running:
  - `cargo test --release --workspace` (0 failed)
  - `cargo test --release --test wat_arc153_nil_rename` (10/10)
  - `grep -rn ':wat::core::unit'` (0 source spellings)
- Sample 3-5 transformed sites to verify the post-rename shape reads cleanly.
- THEN: orchestrator commits atomically (sweep 1a substrate + sweep 1b consumers + this SCORE).

## Why this matters

User direction 2026-05-06: "let's roll." Sweep 1b is the consumer
migration that completes arc 153's rename. After atomic commit:
arc 153 slice 2 closure (INSCRIPTION + 058 row + USER-GUIDE +
WAT-CHEATSHEET + CONVENTIONS update + retire transitional
typealias). Arc 136 slice 2 closure runs after, with do form's
return positions canonically expressed as `:wat::core::nil`.
