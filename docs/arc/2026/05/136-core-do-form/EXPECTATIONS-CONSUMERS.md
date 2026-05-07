# Arc 136 — Consumer Sweep EXPECTATIONS (slice 1b)

**Drafted 2026-05-06.** Pre-handoff scorecard for slice 1b.

**Brief:** `BRIEF-CONSUMERS.md`
**Output:** EDITS to `.wat` files + embedded wat strings in
`.rs` files; let*-with-unit-bindings → do form. COMMIT + PUSH
when workspace 0-failed.

## Setup — workspace state pre-spawn

- HEAD: `ff45f38` (arc 136 slice 1a substrate shipped)
- Working tree clean
- Pre-baseline: `cargo test --release --workspace` = 0 failed (1978/0)
- New symbol `:wat::core::do` available across the workspace
- Detection: `grep -rln '((_ :wat::core::unit)'` finds ~50-150 files
  with sites to consider

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Workspace 0 failed | `cargo test --release --workspace` returns 0 failed across all crates throughout the sweep |
| 2 | No substrate edits | No changes to `src/*.rs` (slice 1a's territory) |
| 3 | Pure-unit chains transformed | Every site where ALL bindings are `((_ :wat::core::unit) <form>)` becomes `(:wat::core::do <form>... <body>)` |
| 4 | Mixed sites preserved | Sites with any non-unit-discard binding stay as `let*` (verified by sampling remaining `((_ :wat::core::unit)` sites post-sweep) |
| 5 | Grep count reduced | `grep -rln '((_ :wat::core::unit)' ...` count drops substantially from baseline (predicted: ~70-90% reduction) |
| 6 | Embedded sites included | `tests/*.rs` and `src/*.rs` embedded wat strings swept too |
| 7 | Latent bugs surfaced | Any sites where `(_ :unit)` was silently coercing non-unit values get reported as honest delta (do's stricter non-final discipline reveals them) |
| 8 | Commit + push | sonnet commits + pushes; HEAD advances |
| 9 | Honest report | Per BRIEF reporting requirements (sweep summary, latent surfaces, verification, path, deltas) |
| 10 | Time-box honored | <= 90 min wall-clock |

**Hard verdict:** all 10 must hold. Rows 1, 3, 4, 5 are the
load-bearing rows (sweep correctness + workspace cleanliness).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | Net negative — do form is shorter than let*-with-unit-bindings. Expected ~3-5 LOC reduction per transformed site; ~300-500 total LOC delta. |
| 12 | clippy clean | No new clippy warnings. |
| 13 | Cargo run cycles | 4-8 cargo verification runs throughout the sweep (one per major batch + final). >12 = grinding signal. |
| 14 | No-grinding discipline | No site required >3 reads/edits to resolve. |

## Independent prediction

- **Most likely (~65%) — Mode A clean.** Mechanical 1:1 transform;
  semantics-preserving; workspace stays green throughout. ~50-70
  min wall-clock.
- **Mode B-substrate-bug (~5%):** edge case in slice 1a's do form
  surfaces (e.g., a let* site with very specific shape exposes a
  bug in eval_do). Sonnet stops + reports.
- **Mode C-unexpected-shape (~15%):** a class of let*
  sites turns out to be subtler than "pure-unit chain vs mixed"
  — e.g., a site where the body itself is wrapped in `(_ :unit)`
  somehow, or where the transform reveals a semantics question.
  Surface gap.
- **Mode D-latent-bug (~10%):** transformation surfaces real
  pre-existing bugs (let*-with-unit-bindings was hiding non-unit
  silent coercion). Surface count; don't try to fix in 1b.
- **Mode B-time-violation (~5%):** sweep doesn't complete in 90
  min. Surface progress.

## Time-box

90 minutes wall-clock. ScheduleWakeup at T+90 min.

## What sonnet's success unlocks

**Mode A clean:** workspace stays 0-failed; codebase advertises do
form everywhere appropriate; let*-with-unit-bindings crutch
retired. Slice 2 closure (INSCRIPTION + 058 row + USER-GUIDE)
spawns next.

**Mode B/C/D:** surface gap; orchestrator adjusts brief or
substrate; reland.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Verify load-bearing rows by re-running `cargo test --release
  --workspace` locally.
- Sample 3-5 transformed sites to verify the do form reads cleanly
  + semantics preserved.
- Sample 2-3 mixed sites that were preserved to verify they are
  genuinely mixed (not false-negative skips).
- Confirm sonnet committed + pushed.
- Open follow-up tasks for slice 2 (closure paperwork).

## Why this matters

User direction 2026-05-06: "1b is on deck." Slice 1a shipped the
substrate; slice 1b retires the crutch via mechanical sweep.
The do form's discipline (Clojure-faithful; non-finals'
types unconstrained; final form's type IS the do's type) is
already in place. Now the codebase migrates to use it.

Mode A clean = the do form is the canonical sequencing form
across the codebase; the let*-with-unit-bindings noise pattern
retires; slice 2 closes arc 136.
