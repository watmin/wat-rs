# EXPECTATIONS — Arc 234 Stone 234.2c — runtime class-safety in per-field accessor bodies

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Probe FLIPS 2/5 PASS → 5/5 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | Stone 234.2b regression guard | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 4 | Stone 234.5 regression guard | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 5 | Stone 234.2a regression guard | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 6 | Stone 234.1.5 regression guard | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 7 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 8 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 20–40 min Mode A
**Upper bound:** 60 min (STOP-3 hard cap)
**Confidence:** high — well-localized wat-side macro extension; well-precedented patterns (Option/expect + if + string::concat all used by 234.2b's predecessor); probe already committed.

**Rationale:**
- Macro extension: ~10-15 lines inside the per-field accessor body
- Probe authoring time: 0 (already on disk)
- Compile cycles: 1-2 rounds expected
- Full scorecard run: ~3 min
- SCORE writing: ~10 min

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): focused single-file change
- Stone 234.2b (~78 min): full macro authoring (much larger)
- Stone 234.2c estimate: ~25-35 min predicted; band's middle

**Risks:**
- **`:wat::core::None` FQDN form** — confirm the exact form (constructor `:wat::core::None` vs `:wat::core::Option/None`) by checking `wat/holon/defrecord.wat` predecessor's usage
- **Runtime concat for Option/expect msg** — should work per `expect_panic` analysis (it evals msg_ast as String); first empirical use
- **Macro expand-time string concat for accessor-name in message prefix** — `~(:wat::core::string::concat ...)` at expand time produces the literal accessor name string; mirror 234.2b's predicate-name pattern

## Rank-up demonstration

Per `project_party_comp_inquisitor_shadowdancer`: orchestrator marked the contract (sub-DESIGN + probe + initial-state verification); sonnet strikes-to-kill via focused macro extension.

For Stone 234.2c specifically:
- 234.2b's macro patterns (the field-walking loop; the inner-let bindings; the runtime quasiquote shape) are mostly mirrored
- The new addition: wrap `field-at` arg in `Option/expect + if + string::concat` for the class check
- Mechanical work — pattern is well-established; sonnet's main task is integration into the existing accessor-emit loop without breaking the loop's other obligations

Capture in SCORE:
- Macro line count delta (estimated ~10-15 added per accessor)
- Did `:wat::core::None` use the right FQDN form first try
- Cascade depth (predicted: 1-2 cycles)
- Honest deltas if any surface

## Out-of-scope rows (REJECTED)

- Substrate (Rust) changes (D8; STOP-5)
- Constructor body changes (D7)
- Predicate body changes (D6)
- Zero-field record changes (D5)
- Unchecked-accessor escape hatch (D9 HARD CUT)
- New probe authoring (probe committed pre-spawn)
- holon-rs touched (STOP-4)
- Substrate-level type narrowing (D10 — future lift when arc 232.1 ships)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1** — unexpected compile errors not tracing to macro extension
- **STOP-2** — lib baseline < 827
- **STOP-3** — 60 min elapsed (small stone; tight cap)
- **STOP-4** — holon-rs touched
- **STOP-5** — Rust changes (pure macro extension)
- **STOP-6** — scope creep
- **STOP-7** — probe doesn't flip 2/5 → 5/5
- **STOP-8** — 234.2b regression guard regresses
- **STOP-9** — any prior arc 234 regression guard regresses
- **STOP-10** — clippy > 54

Each STOP is REJECTION. None is permission-to-defer. If hit: report; surface; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2c.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Macro line-count delta + per-accessor body shape
- Cascade depth: compile rounds
- Time breakdown
- Calibration delta (20-40 target; 60 STOP)
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface
- Rank-up evidence — 234.2b pattern reuse effectiveness

## What this completes

When 234.2c ships:
- The silent-wrong-field-returned gap is closed
- All per-field accessors are class-safe at runtime
- The accessor + predicate together form a complete safety surface
- Arc 234's user-facing API surface is feature-complete + safe for v1

Stones 234.3 (record-y verbs) + 234.4 (hash-destructure) + 234.6 (migration sweep) + 234.7 (INSCRIPTION) remain.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2c.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2c.md` — sub-DESIGN
- `tests/probe_arc234_stone2c_accessor_class_safety.rs` — the FM 2-bis probe (2/5 PASS verified)
- `wat/Record.wat` — the macro file (target)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — predecessor SCORE
- `wat/holon/defrecord.wat` — pattern reference for predicate-name computation
- `feedback_sonnet_writes_substrate.md` — discipline
- `feedback_inscription_immutable.md` — predecessor SCOREs stay unchanged
