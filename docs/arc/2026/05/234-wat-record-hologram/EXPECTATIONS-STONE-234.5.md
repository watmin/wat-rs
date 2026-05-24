# EXPECTATIONS — Arc 234 Stone 234.5 — `:wat::holon::*` auto-dispatch on `Value::wat__Record`

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 6/6 FAIL → 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 3 | Stone 234.2b regression guard | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 4 | Stone 234.2a regression guard | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 5 | Stone 234.1.5 regression guard | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 6 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 7 | Stone 234.0 regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 8 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 60–90 min Mode A
**Upper bound:** 120 min (STOP-3 hard cap)
**Confidence:** medium — 5 verbs touched; centralized helper pattern reduces per-verb work; precedent (Stone 234.2a-CORRECTION's `infer_record_of`) is established.

**Rationale:**
- Runtime: helper fn (~15 lines) + 4 verb threadings (~30-40 lines combined) + `to_holon_inner` arm (~5 lines) = ~50-60 lines
- Check.rs: 4-5 TypeScheme broadenings or custom handlers = ~30-60 lines
- Probe already committed: no probe-authoring time
- Compile cycles: 2-4 rounds (each verb's threading is independent; can iterate)
- SCORE writing: ~10-15 min

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): one custom handler
- Stone 234.2a (~58 min): substrate primitives + 2 TypeSchemes
- Stone 232.0a (~52 min): 3 verbs + dispatch arms + check.rs special-cases
- Stone 234.5 estimate: ~70-90 min predicted; band's upper-middle (5 verbs is more than 3; centralization may shorten total)

**Risks:**
- **Bundle's vec-of-children threading (probe 4)** — splicing records inside `[r1 r2 r3]` for `Bundle` requires check.rs to allow `:wat::Record` as Vec element where HolonAST is expected. This is structurally similar to Stone 234.2a-CORRECTION's heterogeneous-vec fix; may require a similar custom-handler approach OR a Vec-element accept-helper.
- **Mixed-args composition (probe 6)** — if any verb's threading is incomplete, the composition breaks. The probe explicitly tests this; sonnet validates all 4 algebra verbs hold together.
- **`pair_values_to_vectors` indirection** — cosine routes through this helper; sonnet investigates whether the threading goes inside the helper OR before the call.

## Rank-up demonstration — Inquisitor/Shadowdancer party-comp

Per `project_party_comp_inquisitor_shadowdancer`: the orchestrator marked the integration target (sub-DESIGN + 6-contract probe + initial-FAIL verification); sonnet strikes-to-kill in the substrate-as-teacher cascade.

For Stone 234.5 specifically:
- 5-verb scope: each verb's threading is the unit of work; cascade may iterate per verb as compile errors surface
- The centralized helper pattern (D1) is the architectural lever; if it composes cleanly, threading the verbs is mechanical
- Custom-handler precedent (Stone 234.2a-CORRECTION) shortens check.rs work significantly
- Mixed-args composition (probe 6) is the proof that the helper threads uniformly across composition layers

Capture in SCORE:
- How the centralized helper pattern reduced per-verb work
- Which verbs threaded cleanly vs which required iteration
- Cascade depth per verb
- Was the custom-handler-per-verb or centralized-accept-helper approach chosen for check.rs

## Out-of-scope rows (REJECTED)

- Additional `:wat::holon::*` verbs beyond the 5 named (Permute, Thermometer, Blend, Atom, is?, is-Map?, presence?, etc.) — deferred to Stone 234.6 migration sweep if needed
- Class_fqdn validation inside VSA verb bodies (Stone 234.2c — D5 explicitly rejects)
- Changes to the 234.2b macro
- Changes to any prior arc 234 SCORE doc (INSCRIPTION-immutable)
- Changes to existing probes
- holon-rs touched (STOP-4)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1** — unexpected compile errors not tracing to the 5 verbs' updates
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 120 min elapsed (hard cap; medium-sized stone)
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes outside `src/runtime.rs` + `src/check.rs`
- **STOP-6** — scope creep: additional `:wat::holon::*` verbs beyond the 5 named; class_fqdn validation inside VSA verbs
- **STOP-7** — the new probe doesn't flip 6/6 PASS
- **STOP-8** — Stone 234.2b regression guard regresses
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION. None is a permission slot. If hit: report; surface; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Implementation pattern chosen (centralized helper vs per-verb threading; custom handler vs broadened TypeScheme)
- Runtime: helper fn name + line count + per-verb threading line counts + `to_holon_inner` arm
- Check.rs: TypeScheme/handler approach + line count
- Cascade depth: compile rounds + iteration cycles per verb
- Time breakdown
- Calibration delta (60-90 target; 120 STOP)
- Rank-up evidence — Stone 234.2a-CORRECTION precedent effectiveness
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface

## What this completes

When 234.5 ships:
- The hologram property is EXTERNALLY OBSERVABLE — `(:wat::holon::cosine r1 r2)` works without conversion
- Records flow through VSA composition naturally: `(:wat::holon::Bind c (:wat::holon::Bundle [r1 r2]))`
- Arc 234's user-facing surface is feature-complete for v1 (Stones 234.3, 234.4 are UX-improvers but not load-bearing for the hologram thesis)
- Stone 234.6 (migration sweep) becomes ergonomic — callers migrating off `:wat::holon::defrecord` retain VSA capability

## The fifth fight in arc 234's dungeon

Stone 234.0 gear-check (~38 min). Stone 234.1 bigger fight (~30 min). Stone 234.1.5 rename (~25 min). Stone 234.2a substrate primitives (~58 min) + atomic CORRECTION (~ ?? min). Stone 234.2b macro (~78 min, with two honest deltas surfaced).

**Stone 234.5 is the HOLOGRAM PROOF** — the stone that demonstrates the dual-form claim isn't just storage; it's externally observable. After this, "wat-records flow through VSA verbs natively" goes from DESIGN claim to substrate reality.

The party-comp continues. Five wins on arc 234 this session; this is the sixth.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.5.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.5.md` — sub-DESIGN with 9 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (line 290: VSA integration intent)
- `tests/probe_arc234_stone5_holon_auto_dispatch.rs` — FM 2-bis probe (6 contracts; 6/6 FAIL verified)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent (`infer_record_of`)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — macro consumer SCORE
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
