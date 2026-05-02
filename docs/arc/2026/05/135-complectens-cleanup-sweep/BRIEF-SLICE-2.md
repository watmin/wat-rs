# Arc 135 Slice 2 — Sonnet Brief: telemetry/Console + telemetry/Service

**Goal:** apply the *complectēns* discipline to TWO test files in `crates/wat-telemetry/wat-tests/telemetry/`. Cold start.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** TWO files —
1. `crates/wat-telemetry/wat-tests/telemetry/Console.wat` — 🟠 (80-line + 65-line deftests).
2. `crates/wat-telemetry/wat-tests/telemetry/Service.wat` — 🟠×2 + 🟡 (59, 58, 42 line deftests).

NO substrate changes. NO other test files. NO documentation. NO commits.

## Read in order

1. `docs/arc/2026/05/135-complectens-cleanup-sweep/BRIEF-SLICE-1.md` — slice 1's brief (different files; same shape).
2. `docs/arc/2026/05/135-complectens-cleanup-sweep/SCORE-SLICE-1.md` (if it exists by the time you start) — the calibration data from slice 1.
3. `.claude/skills/complectens/SKILL.md` — the spell. The "Edge cases" section is load-bearing.
4. `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the discipline.
5. **Worked demonstrations:** `crates/wat-lru/wat-tests/lru/CacheService.wat` (single-prelude) + `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` (two-prelude).
6. Then your targets.

## Same constraints as slice 1

- Two-file diff only.
- NO substrate. NO commits.
- Outcomes preserved.
- Apply SKILL edge-case guidance.

## What success looks like

Each deftest body 3-7 lines; per-helper deftests; top-down; workspace green.

## Report

Same shape as slice 1: per-file body line-counts (BEFORE → AFTER), helpers, per-helper deftests, outcomes, honest deltas, four questions.
