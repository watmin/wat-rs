# Arc 135 Slice 2 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a27f962e6edc8e489`
**Runtime:** ~8 min (488s).

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Two-file diff | **PASS** | Only `crates/wat-telemetry/wat-tests/telemetry/Console.wat` + `Service.wat` modified. |
| 2 | Helpers added | **PASS** | 8 total (2 Console + 6 Service). |
| 3 | Each existing deftest body shrinks | **PASS** | Console: dispatcher-edn 8→4 outer logical bindings; dispatcher-json 4→2. Service: spawn-drop-join 37→1 line (97%); batch-roundtrip ~15→5 outer logical (93%); cadence-fires same shape. All within 3-7 outer logical target. |
| 4 | Per-helper deftests added | **PASS** | 7 of 8 helpers have isolated deftests. The exempted one (`tel-stdout-from-result` — RunResult thin accessor) is Level 3 taste — same exemption sonnet applied in slice 1, documented as such. RunResult cannot be constructed in isolation without hermetic infrastructure. |
| 5 | No forward references | **PASS** | Files read top-down. Layer 0 helpers → their deftests → Layer 1 → their deftests → Layer 2 → final scenarios. |
| 6 | **Outcomes preserved** | **PASS** | `cargo test --release --workspace` exit=0; 35/35 in wat-telemetry tests; no regressions; new helper deftests pass cleanly. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications. |
| 8 | Honest report | **PASS+** | Three deltas surfaced. One genuinely new (Delta 1 — embedded lambda literals); one repeat from slice 1 (Delta 2 — RunResult opacity); one design observation (Delta 3 — type unification cost). |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result |
|---|---|---|
| 9 | Helper count | PASS — 8 in band (6-15). |
| 10 | Average body shrink | PASS+ — outer logical: ~90% across the three Service tests; Console outer logical 50-75%. |
| 11 | Workspace runtime | PASS. |
| 12 | Edge-case usage | PASS — sonnet correctly applied SIX SKILL edge cases. |

**SOFT VERDICT: 4 OF 4 PASS. Clean ship.**

## Calibration insights

### New SKILL delta — embedded literals generalize the hermetic edge case

Sonnet's Delta 1: inline `MetricsCadence/new` lambda literals (gate value + tick fn) inflate visual line counts even when outer logical structure is clean. The lambda literal in a binding's RHS is irreducible — it's data passed to the cadence factory. Visual count over-flags; outer logical count is correct.

This generalizes the existing "Hermetic-program tests have inherently irreducible bodies" edge case. The SKILL note becomes: ANY embedded literal (program AST in `run-hermetic-ast`, cadence lambda, dispatcher closure, translator fn) inflates visual line count. Phase-2 judgment counts OUTER LOGICAL BINDINGS.

### Repeat from slice 1 — RunResult accessor exemption

`tel-stdout-from-result` has no isolated deftest because RunResult requires hermetic infrastructure to construct. Level 3 taste exemption — same as slice 1's `stdout-from-result`. No SKILL change needed; pattern is documented.

### Design observation — type unification cost in shared helpers

When a helper takes a generic parameter (e.g., `MetricsCadence<G>`), forcing two callers with different G types to share the helper requires choosing one G. The null-cadence test ends up passing an i64 cadence even though G is unused. Future shape: split into two specialized helpers. Deferred — current form is correct.

## Independent prediction calibration

Predicted (from EXPECTATIONS-SLICE-1 framework reused):
- 65% all-pass — FIRED ✓
- 10% new edge case surfaces — FIRED ✓ (Delta 1)

Both fired together. Same pattern as slice 1: ships clean AND surfaces refinement data. The artifacts-as-teaching record continues compounding.

Sweep timing: 8 min — half of slice 1's 16 min. The compounding sweep timing trend continues (HologramCacheService 14 min → slice 1 16 min → slice 2 8 min).

## Ship decision

**SHIP.** All hard + soft pass. The Delta 1 generalization gets baked into the SKILL before slice 3.

## Next steps

1. Refine SKILL — generalize hermetic-program edge case to "embedded literals" broadly.
2. Commit slice 2 + this SCORE + SKILL refinement + FOLLOWUPS update.
3. Spawn slice 3 (WorkUnit.wat + WorkUnitLog.wat — the largest slice; 7 deftests across 2 files).
