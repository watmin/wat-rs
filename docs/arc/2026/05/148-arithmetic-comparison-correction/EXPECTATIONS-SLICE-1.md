# Arc 148 Slice 1 — Pre-handoff expectations

**Drafted 2026-05-03.** Pure-audit slice — no code changes.
Sonnet enumerates the existing surfaces of all 7
`infer_polymorphic_*` handlers and produces `AUDIT-SLICE-1.md`.
Predicted MEDIUM-LIGHT slice (Mode A ~80%; Mode B-architecture-
mismatch ~15%; Mode C ~5%).

**Brief:** `BRIEF-SLICE-1.md`
**Output:** 1 NEW Markdown file (`AUDIT-SLICE-1.md`) in this
arc's directory + report. NO `src/` edits. NO `wat/` edits.
NO tests.

## Setup — workspace state pre-spawn

- Arc 148 DESIGN locked 2026-05-03 (this session) after multi-turn
  debate; architecture: Type-as-namespace for same-type arithmetic
  leaves; verb-comma-pair for mixed-type leaves; substrate-primitive
  + selective-mixed-arms for comparison; Category A non-numeric
  eq/ord SOLVED by universal same-type delegation (not deferred).
- 1 in-flight uncommitted file (`crates/wat-lru/wat-tests/lru/CacheService.wat`
  — arc 130 noise; ignore).
- Workspace baseline (per FM 9 baseline check 2026-05-03):
  reflection-layer baselines all green (45/45 across 5 test files);
  workspace failure profile is the documented CacheService.wat noise.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | NEW `docs/arc/2026/05/148-arithmetic-comparison-correction/AUDIT-SLICE-1.md`. NO other files modified or added. NO Rust changes. NO wat changes. NO test changes. |
| 2 | All 7 handlers enumerated | `infer_polymorphic_compare` + `infer_polymorphic_arith` + `infer_polymorphic_time_arith` + `infer_polymorphic_holon_pair_to_f64` + `infer_polymorphic_holon_pair_to_bool` + `infer_polymorphic_holon_pair_to_path` + `infer_polymorphic_holon_to_i64`. Each has its own section. |
| 3 | User-facing ops per handler | Each handler section enumerates the user-facing op keywords that route to it. Source: `src/check.rs:3285-3360` dispatch site. Cited file:line. |
| 4 | Argument acceptance per handler | Each handler section documents the argument types accepted, the unification logic, the result type. Cited from handler body in `src/check.rs:6567+`. |
| 5 | Mixed-type signatures enumerated | For handlers that explicitly support cross-type combinations: enumerate them. arith: numeric promotion. time-arith: 3 signatures. holon-pair: HolonAST OR Vector cross-acceptance. |
| 6 | Runtime impl references | For each user-facing op: which `eval_*` function handles it at runtime. Sourced from `src/runtime.rs:2593-2631`. |
| 7 | Category mapping | Section maps handlers to arc 148's categories: NUMERIC ARITHMETIC (slice 2), NUMERIC COMPARISON (slice 3), CATEGORY A UNIVERSAL (handled by slice 3's substrate primitive), CATEGORY B time-arith (parallel track), CATEGORY C holon-pair (parallel track). |
| 8 | Open questions surfaced | Section listing factual unknowns / discrepancies between DESIGN's assumptions and the substrate's current state. Empty section is OK if no discrepancies; explicitly states "no discrepancies found" if so. |
| 9 | All file:line citations valid | Spot-check 5 random citations in the audit doc — each must point to actually-existing code at that line in the current working tree. |
| 10 | Honest report | ~200-word report covers all required sections from the brief; predicted complexity of slices 2-3 named. |

**Hard verdict:** all 10 must pass. Rows 2 + 7 + 8 are the
load-bearing rows (the audit's actual content).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | AUDIT-SLICE-1.md: 300-700 lines. >1000 lines = re-evaluate scope (audit became implementation planning). |
| 12 | Style consistency | New audit doc mirrors structure of `docs/arc/2026/05/146-container-method-correction/SCORE-SLICE-2.md` for section headers + table style. |
| 13 | No phantom citations | Every file:line reference is verifiable. No "as documented in X" without a specific path. |
| 14 | Audit-first discipline | If sonnet finds DESIGN.md's architectural assumptions don't match reality, surface as open-question (not silent edit; not "I fixed it"). |

## Independent prediction

- **Most likely (~80%) — Mode A clean ship.** Audit is mechanical;
  brief is detailed; pre-flight crawled the load-bearing files. The
  7 handlers are all in `src/check.rs` in adjacent line ranges.
  ~30-50 min wall-clock.
- **Mode B-architecture-mismatch (~15%)** — sonnet finds the
  substrate doesn't quite match DESIGN's assumptions (e.g., the
  `infer_polymorphic_compare` handler accepts Types DESIGN didn't
  list; per-Type leaves already exist in `register_builtins` that
  DESIGN didn't catalog). Surfaces as open-questions; orchestrator
  reconciles before slice 2 spawn.
- **Mode C (~5%)** — sonnet hits an unforeseen substrate edge
  (parsing fails on something; reflection doesn't surface a handler
  via lookup_form; etc.). Surface as STOP-at-first-red.

## Time-box

60 min wall-clock (2× the predicted upper-bound of 30-50 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

Slice 2 (numeric arithmetic migration — 32 names) and slice 3
(numeric comparison migration — 18 names) both depend on the
audit. If the audit surfaces architectural surprises, slices 2-3
briefs are revised before spawn.

The parallel user-track work on Category B (time-arith) and
Category C (holon-pair algebra) also benefits from the audit's
enumeration of those handlers' surfaces.

## After sonnet completes

- Re-read DESIGN's assumptions against the audit's findings
- Score the 10 hard rows + 4 soft rows
- Write `SCORE-SLICE-1.md` documenting the audit's discoveries +
  any open-question reconciliation needed before slice 2
- Commit the SCORE before drafting slice 2's BRIEF (so calibration
  preserved across compactions)
