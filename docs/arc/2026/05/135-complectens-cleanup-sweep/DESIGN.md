# Arc 135 — Complectēns cleanup sweep

**Status:** opened 2026-05-03.

## TL;DR

Apply the *complectēns* discipline to every test file flagged in arc 130's `FOLLOWUPS.md`. Eight wat test files contain monolithic deftest bodies that fail the four questions. Refactor each as a top-down dependency graph in one file: named helpers in a `make-deftest` prelude (or two preludes for mixed-outcome files), per-helper deftests proving each layer, final scenario deftests with bodies 3-7 lines.

The codebase must not advertise bad practices. We are exemplars of wat. The arc is not closed until the foundation is strong.

## Provenance

Surfaced 2026-05-03 by the first cast of `/complectens` across the codebase. The spell's mechanical phase found 22 deftests in 9 files exceeding the empirical body-line threshold (>30 lines). Two were already shipped (✓ wat-lru CacheService.wat — the worked demo; ✓ HologramCacheService.wat — the calibration sweep). Eight files remain.

The HologramCacheService calibration validated the artifacts-as-teaching hypothesis: a fresh sonnet shipped 8/8 hard + 4/4 soft from the documents alone. That validates the discipline; this arc applies it.

## Goal

After arc 135 ships:

- Every wat test file in `wat-tests/` and `crates/*/wat-tests/` passes the *complectēns* check at Level 1 + Level 2. Bodies > 30 lines either shrink to 3-7 lines via composition OR are exempted under phase-2 judgment as inherently complex with documented justification.
- Every named helper has its own deftest proving it.
- File reads top-down with no forward references.
- Workspace stays green throughout.

## Files in scope

From arc 130's FOLLOWUPS.md (priority order):

| Tier | File | Worst body | Sweep |
|---|---|---|---|
| 🔴 | `wat-tests/service-template.wat` | 106 | Slice 1 |
| 🔴 | `wat-tests/console.wat` | 101 | Slice 1 (paired) |
| 🟠 | `crates/wat-telemetry/wat-tests/telemetry/Console.wat` | 80 | Slice 2 |
| 🟠 | `crates/wat-telemetry/wat-tests/telemetry/Service.wat` | 59 | Slice 2 |
| 🟠 | `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` | 62 | Slice 3 |
| 🟠 | `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` | 89 | Slice 3 |
| 🟡 | `wat-tests/test.wat` | 42 | Slice 4 (phase-2 judgment) |
| 🟡 | `wat-tests/stream.wat` | 31 | Slice 4 |
| 🟡 | `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` | 35 | Slice 4 |
| 🟡 | `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` | 43 | Slice 4 |

## Slice plan

**Slice 1** — `service-template.wat` + `console.wat` (top-level wat-tests; flagship Console+driver pattern). Likely shared helpers across both files, but kept in their own files per the one-file rule.

**Slice 2** — `crates/wat-telemetry/wat-tests/telemetry/Console.wat` + `Service.wat`. Shared telemetry-Console + Service shape; helpers may overlap.

**Slice 3** — `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` + `WorkUnitLog.wat`. Shared WorkUnit family.

**Slice 4** — Suspect-tier files (`test.wat`, `stream.wat`, `HologramCache.wat`, `step-B-single-put.wat`). Some may exempt under phase-2 judgment; others compose cleanly.

Each slice gets BRIEF + EXPECTATIONS + sonnet sweep + SCORE per the standard discipline.

## What this arc unblocks

- **Codebase exemplarity.** Every wat test file demonstrates the *complectēns* discipline. New contributors learn the pattern from any file they read.
- **Spell calibration.** Each sweep validates the artifacts-as-teaching record again. Document gaps surface and get refined.
- **Per-helper proof tree.** When future tests fail, the per-layer deftests narrow the bisect window.
- **Arc 130 closure.** Arc 130's substrate redesign of cache services is paused; this arc removes test-file shape as a blocker.

## The four questions (as gate)

Every sweep's SCORE doc includes the four questions applied to its output:
- Obvious — failure trace narrows to a named layer
- Simple — body is 3-7 lines composing the top layer
- Honest — helper names match exactly what their bodies do
- Good UX — fresh reader traces top-down

A slice does not ship unless all four hold.

## Cross-references

- `.claude/skills/complectens/SKILL.md` — the spell.
- `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the discipline.
- `docs/arc/2026/05/130-cache-services-pair-by-index/CALIBRATION-HOLOGRAM-SCORE.md` — the validation that the artifacts teach.
- `docs/arc/2026/05/130-cache-services-pair-by-index/FOLLOWUPS.md` — the queue this arc works through.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — worked demo (single prelude).
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — worked demo (two preludes for mixed-outcome).
