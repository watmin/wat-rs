# Arc 212 stone δ-refuse-mutation — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-15 min Mode A. Mechanical migration; two short cargo invocations.
- **LOC changed:** ~12 (delete List-only recursion line, add children() recursion + arc 212 comment)
- **New files:** 1 (SCORE-212-DELTA-REFUSE-MUTATION.md)
- **Surprises expected:** **MODERATE** — this walker was previously List-only with NO Vector arm. Migration EXTENDS coverage to Vector + StructPattern bracketed shapes. There may be code patterns elsewhere that buried mutation forms inside Vector RHSes assuming silent acceptance.

## Honest-delta watch (HIGH PRIORITY)

This stone differs from δ-bare-primitives: the previous walker had no Vector arm at all, so this migration ADDS NEW behavior (catching mutation forms in bracketed positions). Two scenarios to watch:

1. **Both named tests pass.** Migration is clean. No latent bugs surfaced. Mode A. Expected if test fixtures never embedded mutation forms in Vector positions.

2. **One or both named tests fail.** The substrate is teaching that previously-silent mutation forms inside bracketed shapes are now caught. This is honest substrate-as-teacher behavior, NOT a regression. STOP-trigger 1's sub-rule fires: sonnet reverts + reports the honest delta. The orchestrator then decides:
   - Test was relying on buggy silent acceptance → adjust the test (separate stone)
   - Test exposes a substrate path that genuinely needs the mutation form in Vector position → walker rule needs sharpening (separate stone)
   - Either way: the migration's coverage extension stands as the correct shape; the path to ship it requires the orchestrator's decision.

**The Mode B outcome is acceptable failure-engineering data**, not a sonnet failure.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `refuse_mutation_forms` uses `ast.children()` for recursion | YES |
| 2 | List-head mutation check preserved verbatim | YES |
| 3 | `cargo test --release --test probe_declaration_form_lift` green | YES (likely; possible Mode B trigger) |
| 4 | `cargo test --release --test wat_eval_result` green | YES (likely; possible Mode B trigger) |
| 5 | `cargo build --release` clean | YES |
| 6 | SCORE file written at named path | YES |
| 7 | Zero other code edits anywhere | YES |
| 8 | Zero test-file edits | YES |

## Mode classification

- **Mode A:** all criteria satisfied; both tests green; SCORE clean.
- **Mode B (acceptable):** migration applied; test failure traceable to extended coverage catching previously-silent mutation form; sonnet REVERTED + inscribed the honest delta with file:line of the offending mutation-in-Vector pattern.
- **Mode C:** STOP rule broken (touched test files; "improved" is_mutation_form; investigated unrelated failures; modified a test to "make it pass").

## Calibration metadata

- **Orchestrator confidence:** HIGH on the mechanical migration; MEDIUM on Mode A vs Mode B outcome. The walker has NO Vector arm so the migration's reach is genuinely new.
- **Risk factors:**
  - Extended coverage may catch previously-silent bugs in test fixtures
  - The walker is called from `eval_in_frozen` (runtime evaluator path), so failures cascade through eval-ast! callers
- **Why this matters:**
  - This walker has the LARGEST latent bug class (silently-accepted mutation forms in Vector positions across the entire eval-ast! surface)
  - Extending coverage is the CORRECT direction per L4 doctrine
  - If Mode B fires, the honest delta itself is the next stone's data — keeps the substrate-as-teacher cascade running

## Per-stone discipline note

This is the second L1 migration stone. The first (δ-bare-primitives) shipped Mode A in 97 seconds. If this one ships Mode A in similar time, the discipline scales. If it ships Mode B, the honest stop validates the STOP-trigger sub-rule — sonnet should report rather than investigate.

## Cross-references

- Arc 212 DESIGN § "Locked stone chain (L0 → L4 trajectory)" — δ-refuse-mutation slot
- SCORE-212-GAMMA-1-AUDIT-CATALOG.md — sonnet's "most incomplete" classification of this walker
- SCORE-212-DELTA-BARE-PRIMITIVES.md — the previous stone (Mode A in 97 sec; established cadence)
- BRIEF-212-DELTA-REFUSE-MUTATION.md — the brief itself
