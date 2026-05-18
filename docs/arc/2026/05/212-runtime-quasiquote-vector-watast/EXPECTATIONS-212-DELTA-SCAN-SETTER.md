# Arc 212 stone δ-scan-setter — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-15 min Mode A. Mechanical migration; two short cargo invocations.
- **LOC changed:** ~12 (collapse List + Vector arms into children() recursion)
- **New files:** 1 (SCORE-212-DELTA-SCAN-SETTER.md)
- **Surprises expected:** 0-1 (StructPattern coverage extension may surface a previously-silent setter — unlikely since setters typically appear in top-level List position in loaded files)

## Honest-delta watch

Migration extends coverage to StructPattern (List + Vector already covered). Two scenarios:

1. **Both tests pass.** No latent bugs. Mode A. Expected — setters don't naturally appear inside StructPattern contexts.

2. **A test fails.** Substrate is teaching about a latent silent-acceptance bug. STOP-trigger 1 sub-rule fires: revert + report. Mode B acceptable.

Mode A is the strong expectation.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `scan_for_setter` uses `form.children()` for recursion | YES |
| 2 | List-head setter check preserved verbatim | YES |
| 3 | `cargo test --release --lib setter_in_loaded_file_halts` green | YES |
| 4 | `cargo test --release --test probe_declaration_form_lift` green | YES |
| 5 | `cargo build --release` clean | YES |
| 6 | SCORE file written at named path | YES |
| 7 | Zero other code edits anywhere | YES |
| 8 | Zero test-file edits | YES |

## Mode classification

- **Mode A:** all criteria satisfied; both tests green; SCORE clean
- **Mode B (acceptable):** test failure traceable to extended coverage; sonnet REVERTED + inscribed delta
- **Mode C:** STOP rule broken

## Calibration metadata

- **Orchestrator confidence:** VERY HIGH. Migration shape is identical to δ-bare-primitives (both had explicit Vector arm already; both collapse to children()). δ-bare-primitives shipped Mode A in 97 sec.
- **Risk factors:** minimal — StructPattern extension is structurally additive
- **Why this matters:** third L1 migration validates cadence on a load.rs walker (different file from previous stones; proves discipline scales beyond src/check.rs + src/freeze.rs)

## Cross-references

- Arc 212 DESIGN § "Locked stone chain (L0 → L4 trajectory)"
- SCORE-212-GAMMA-1-AUDIT-CATALOG.md — sonnet's classification of this walker
- SCORE-212-DELTA-BARE-PRIMITIVES.md — pattern precedent (List+Vector → children())
- SCORE-212-DELTA-REFUSE-MUTATION.md — sibling stone shipped Mode A in 78 sec
- BRIEF-212-DELTA-SCAN-SETTER.md — the brief itself
