# Arc 135 Slice 3 — Sonnet Brief: telemetry/WorkUnit + telemetry/WorkUnitLog

**Goal:** apply the *complectēns* discipline to the WorkUnit family in `crates/wat-telemetry/wat-tests/telemetry/`.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** TWO files —
1. `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` — 🟠×4 + 🟡 (62, 57, 56, 55, 47 line deftests; 5 candidates).
2. `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` — 🟠×2 (89-line + 64-line deftests).

This is the largest slice by helper count. Many shared concepts (build counter / build duration / collect / emit) → reusable helpers.

## Read in order

1. `docs/arc/2026/05/135-complectens-cleanup-sweep/BRIEF-SLICE-1.md`.
2. Slice 1 + 2 SCORE docs (if they exist by the time you start).
3. `.claude/skills/complectens/SKILL.md` (Edge cases section).
4. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`.
5. Worked demonstrations.
6. Targets.

## Constraints

Same as slice 1.

## What success looks like

Each deftest body 3-7 lines; per-helper deftests; top-down; workspace green.

## Report

Same shape as previous slices.
