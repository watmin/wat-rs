# Arc 212 stone δ-def-restricted — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-15 min Mode A. Mechanical migration; one short cargo invocation.
- **LOC changed:** ~12 (collapse List+Vector arms into children() recursion)
- **New files:** 1 (SCORE)
- **Surprises expected:** 0-1 (StructPattern coverage extension may surface a previously-silent restriction violation — unlikely)

## Honest-delta watch

Migration extends coverage to StructPattern. Two scenarios:
1. **Test passes.** No latent bugs. Mode A. Expected — restriction violations live in call-head positions which only appear inside List.
2. **Test fails.** Substrate teaching; revert + report. Mode B acceptable.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `walk_for_def_restricted_call` uses `node.children()` for recursion | YES |
| 2 | List-head restriction check preserved verbatim | YES |
| 3 | `cargo test --release --test wat_arc198_def_restricted` green | YES |
| 4 | `cargo build --release` clean | YES |
| 5 | SCORE file written | YES |
| 6 | Zero other code edits | YES |

## Mode classification

- **Mode A:** all criteria satisfied
- **Mode B (acceptable):** test failure traceable to extended coverage; REVERTED + inscribed
- **Mode C:** STOP rule broken

## Calibration metadata

- **Orchestrator confidence:** VERY HIGH. Fifth L1 migration with this shape; pattern set in stone (pun intended) across δ-bare-primitives + δ-scan-setter + δ-process-stdin-joins.
- **Risk factors:** minimal — call heads only appear in List position, so StructPattern extension is purely structural.
- **Cadence:** 97s + 78s + 155s + 74s established. Expected ≤120s.

## Cross-references

- Arc 212 DESIGN § "Locked stone chain"
- SCORE-212-GAMMA-1-AUDIT-CATALOG.md — classified this walker as pending
- SCORE-212-DELTA-BARE-PRIMITIVES.md / SCORE-212-DELTA-SCAN-SETTER.md / SCORE-212-DELTA-PROCESS-STDIN-JOINS.md — pattern precedent
- BRIEF-212-DELTA-DEF-RESTRICTED.md — the brief itself
- `tests/wat_arc198_def_restricted.rs` — the gate
