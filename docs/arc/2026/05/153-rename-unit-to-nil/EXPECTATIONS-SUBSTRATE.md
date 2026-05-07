# Arc 153 — Substrate EXPECTATIONS (slice 1a)

**Drafted 2026-05-06.**

**Brief:** `BRIEF-SUBSTRATE.md`
**Output:** EDITS to `src/types.rs` + `src/check.rs` + `src/runtime.rs`
+ NEW `tests/wat_arc153_nil_rename.rs`. NO consumer wat edits.
NO commits.

## Setup

- HEAD: `4029173` (arc 153 DESIGN committed)
- Pre-baseline: workspace 1978/0
- DESIGN locked: type-position rename + value-position
  recognition; substrate-as-teacher Pattern 3 walker

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | EXACTLY 4 files (3 modified + 1 new); no other crate; no consumer wat |
| 2 | `BareLegacyUnitName` variant minted | New CheckError variant in `src/types.rs` with Display referencing arc 153 + canonical `:wat::core::nil` |
| 3 | Walker detects `:wat::core::unit` | TypeExpr walker emits `BareLegacyUnitName` per offending site (mirrors arc 109 slice 1d's `BareLegacyUnitType` pattern) |
| 4 | `:wat::core::nil` minted as canonical type | Type registry resolves `:wat::core::nil` to the singleton type (whatever underlying TypeDef previously named unit) |
| 5 | Value-position recognition | `infer` for `WatAST::Keyword(":wat::core::nil")` returns nil type; `eval` returns nil singleton value |
| 6 | New tests 6-10/6-10 pass | `cargo test --release --test wat_arc153_nil_rename` shows all pass |
| 7 | HashMap-key regression check passes | Test #9 (other keywords still treated normally; special-case is narrow) passes |
| 8 | Workspace failure shape | `cargo test --release --workspace` fires many `BareLegacyUnitName` errors on existing `:wat::core::unit` sites; NO substrate panics, NO unrelated TypeMismatch |
| 9 | No commit | `git log --oneline` unchanged from pre-spawn HEAD `4029173` |
| 10 | Honest report | Per BRIEF reporting (crawl confirmation, edit summary, LOC delta, verification, path, deltas) |

**Hard verdict:** all 10 must hold. Rows 3, 4, 5, 8 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | 80-150 LOC across the 4 files. >250 = scope creep. |
| 12 | Pattern fidelity | Walker matches arc 109 slice 1d's BareLegacyUnitType structure; no novel framework |
| 13 | clippy clean | No new clippy warnings on modified files |
| 14 | No grinding | No backwards-compat shims; no defensive code for hypothetical scenarios |

## Independent prediction

- **Most likely (~70%) — Mode A clean.** Pattern 3 walker is a
  well-established recipe; arc 109 slice 1d shipped this exact
  shape. Value-position keyword special-case is narrow. ~25-40
  min wall-clock.
- **Mode B-substrate-internal-bug (~10%):** value-position
  special-case interacts unexpectedly with macro/quasiquote/HashMap-key
  paths. Honest STOP.
- **Mode C-broader-scope (~10%):** turns out the walker also has
  to handle `TypeExpr::Tuple(vec![])` (the `()` shape) or some
  nested parametric containing `:wat::core::unit` requires extra
  arms. Surface; orchestrator may amend brief.
- **Mode B-time-violation (~5%):** doesn't complete in 60 min.
- **Mode D-grinding (~5%):** sonnet hits per-site grinding on the
  walker.

## Time-box

60 minutes wall-clock.

## What success unlocks

**Mode A clean:** sweep 1b can spawn — sonnet reads
`BareLegacyUnitName` diagnostics and edits each consumer site +
sweeps `()` value-position to `:wat::core::nil`.

## After sonnet completes

- Read this file FIRST.
- Score each row.
- Verify load-bearing rows by re-running `cargo test --release
  --test wat_arc153_nil_rename` locally.
- Sample 2-3 workspace failures to confirm `BareLegacyUnitName`
  shape (not panics, not unrelated TypeMismatch).
- DO NOT COMMIT. Working tree stays modified for sweep 1b.

## Why this matters

User direction 2026-05-06: "name swap first, then close out the do
forms." Slice 1a substrate ships the rename + value-position
recognition; slice 1b sweeps consumers; arc 136 slice 2 closure
runs after.

The triplet `:wat::core::nil` / `:wat::core::Some(t)` /
`:wat::core::None` reads cleanly with three names for three roles.
Wat-rs becomes a Lisp on Rust with a vocabulary that honors both
traditions.
