# Arc 212 stone δ-process-stdin-joins — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-15 min Mode A.
- **LOC changed:** ~15 (collapse List+Vector arms; preserve fn/lambda early-return)
- **New files:** 1 (SCORE)
- **Surprises expected:** 0-1 — only risk is dropping the fn/lambda early-return; the BRIEF flags this as load-bearing

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `collect_process_stdin_and_joins` uses `node.children()` for recursion | YES |
| 2 | List-head classification + fn/lambda early-return preserved verbatim | YES |
| 3 | `cargo test --release --test wat_arc202_process_join_holds_stdin` green | YES |
| 4 | `cargo build --release` clean | YES |
| 5 | SCORE file written at named path | YES |
| 6 | Zero other code edits anywhere | YES |

## Mode classification

- **Mode A:** all criteria satisfied
- **Mode B (acceptable):** test fails because early-return was dropped; REVERTED + inscribed
- **Mode C:** STOP rule broken

## Calibration metadata

- **Orchestrator confidence:** HIGH. Migration shape matches δ-bare-primitives + δ-scan-setter. Critical detail (fn/lambda early-return) flagged prominently in BRIEF.
- **Risk:** dropping the early-return is the single failure mode. BRIEF surfaces it twice.
- **Cadence:** fourth L1 migration. δ-console deferred to ζ-newtype-wall (no test gate; compiler will force-migrate when L2 ships).

## Cross-references

- Arc 212 DESIGN § "Locked stone chain"
- SCORE-212-GAMMA-1-AUDIT-CATALOG.md — sonnet's classification of this walker
- SCORE-212-DELTA-BARE-PRIMITIVES.md / SCORE-212-DELTA-REFUSE-MUTATION.md / SCORE-212-DELTA-SCAN-SETTER.md — pattern precedent
- BRIEF-212-DELTA-PROCESS-STDIN-JOINS.md — the brief itself
- `tests/wat_arc202_process_join_holds_stdin.rs` — the gate
