# Arc 154 — Substrate EXPECTATIONS (slice 1a)

**Drafted 2026-05-06 evening.**

**Brief:** `BRIEF-SUBSTRATE.md`
**Output:** EDITS to `src/check.rs`, `src/runtime.rs`,
`src/special_forms.rs`, NEW `tests/wat_arc154_kill_let_star.rs`.
NO consumer wat edits. NO commits.

## Setup

- HEAD: `d883209`
- Pre-baseline: workspace 1988 / 0 (post arc 153 + arc 136 close)
- DESIGN locked: switch `:wat::core::let` parallel→sequential;
  mint BareLegacyLetStar walker; substrate-as-teacher Pattern 3
  mirroring arc 153 slice 1a's recipe

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | EXACTLY 4 files (3 modified + 1 new); no other crate; no consumer wat |
| 2 | `BareLegacyLetStar` variant minted | New CheckError variant in `src/check.rs` (where `BareLegacyUnitType` / `BareLegacyUnitName` siblings live) with Display referencing arc 154 + canonical `:wat::core::let` |
| 3 | Walker detects `:wat::core::let*` | Operator-position walker emits `BareLegacyLetStar` per offending site; mirrors arc 153 walker structure |
| 4 | `:wat::core::let` sequential | The current `infer_let_star` / `eval_let_star` / tail / step logic moves under `let` keyword; new tests verify sequential semantics work via `:wat::core::let` |
| 5 | Parallel `:wat::core::let` retired | Old `infer_let` / `eval_let` paths retired; zero behavioral footprint (zero consumers anyway) |
| 6 | New tests 6-10/6-10 pass | `cargo test --release --test wat_arc154_kill_let_star` all pass |
| 7 | Walker narrowness | Other keywords unaffected; only `:wat::core::let*` Path triggers walker (test #10 covers) |
| 8 | Workspace failure shape | `cargo test --release --workspace` fires many `BareLegacyLetStar` errors on existing `:wat::core::let*` sites; NO panics outside the established intentional thread-panics, NO unrelated TypeMismatch |
| 9 | No commit | HEAD unchanged from `d883209` (or whatever HEAD is post BRIEF commit; spawning sonnet from BRIEF-committed state) |
| 10 | Honest report | Per BRIEF reporting requirements |

**Hard verdict:** all 10 must hold. Rows 3, 4, 5, 8 are
load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | 80-150 LOC across 4 files (matches arc 153 slice 1a profile) |
| 12 | Pattern fidelity | Walker shape matches `walk_type_for_legacy_unit_name`; CheckError variant mirrors `BareLegacyUnitName` shape |
| 13 | clippy clean | No new clippy warnings |
| 14 | No grinding | No backwards-compat shims; no defensive code |

## Independent prediction

- **Most likely (~75%) — Mode A clean.** Pattern 3 walker is
  established (arc 109 slice 1d + arc 153 sliced 1a both shipped
  this exact recipe successfully). Substrate edits are
  mechanical "move these arms; rename these functions; add
  walker." ~25-40 min wall-clock.
- **Mode B-substrate-internal-bug (~10%):** edge case in tail-call
  or step path post-rename. Honest STOP.
- **Mode C-unexpected-interaction (~10%):** some special-form
  arc 144 reflection trio interaction (lookup-form sees
  `:wat::core::let*` differently than expected post-walker).
  Surface gap.
- **Mode B-time-violation (~5%):** sweep doesn't complete in
  60 min.

## Time-box

60 minutes wall-clock. ScheduleWakeup at T+60 min.

## What success unlocks

**Mode A clean:** sweep 1b can spawn — sonnet reads
`BareLegacyLetStar` diagnostics + edits ~827 sites mechanically;
atomic commit when workspace returns to 0-failed.

## After sonnet completes

- Read this file FIRST
- Score each row
- Verify load-bearing rows by re-running `cargo test --release
  --test wat_arc154_kill_let_star` locally
- Sample 2-3 workspace failures to confirm `BareLegacyLetStar`
  shape (not panics, not unrelated TypeMismatch)
- DO NOT COMMIT — working tree stays modified for sweep 1b

## Why this matters

User direction 2026-05-06 evening: *"new arc - let's do it."*
Slice 1a substrate ships the rename + walker; slice 1b sweeps
consumers; slice 2 retires walker body + closure paperwork
(orchestrator-side INSCRIPTION).

Two foundation marks landed earlier today (`nil`, `do`); arc 154
lands the third — single-letform vocabulary. Wat-rs's Lisp
identity strengthens by one keyword cleanup.
