# EXPECTATIONS — Arc 234 Stone 234.1.5 — variant rename + `:wat::record` namespace promotion

Mode A target: **13/13 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 5/5 FAIL → 5/5 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | **Stone 234.1 probe stays GREEN under rename** (LOAD-BEARING regression guard) | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 4 | Stone 234.0 polymorphic type regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 5 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 6 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 7 | Stone 233.3 regression guard | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.e regression guard | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 9 | Stone 233.2.l regression guard | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 10 | Stone 233.2.k regression guard | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |
| 13 | Zero leftover `wat_record` references (except historical Stone 234.1 comments) | `grep -rE "Value::wat_record\|\\bwat_record\\b" src/ tests/ 2>/dev/null \| grep -v "Arc 234 Stone 234.1\\b" \| wc -l` | 0 |

## Independent prediction

**Target runtime:** 15–30 min Mode A
**Upper bound:** 45 min (STOP-3)
**Confidence:** very high — purely mechanical rename + small type registration; 18 cascade sites enumerated empirically.

**Rationale:**
- 14 sites in runtime.rs: 11 pattern updates + 1 discriminant tag + 2 string literal updates (mechanical)
- 2 sites each in edn_shim.rs + closure_extract.rs: pattern updates (mechanical)
- 1 type registration in check.rs: ~5-10 lines, mirroring existing primitive type pattern
- Stone 234.1 probe update: helper signature + 7 test body patterns (mechanical)
- Compile + iterate cycles: ~1-2 rounds (variant rename triggers cascade; cargo names every leftover; final clean compile)
- SCORE: ~5 min

**Calibration precedents:**
- Stone 234.0 (~38 min): single new primitive + dispatch + TypeScheme; ZERO iteration cycles
- Stone 234.1 (~30 min): variant addition + 4 impl arms + cascade; UNDER 60-120 band
- Stone 234.1.5 estimate: SIMPLER than both — no new behavior, no new dispatch, pure rename + 1 type registration; ~20-25 min plausible

**Risks:**

- **Cascade larger than 18 sites** — possible if any test fixture references old variant via string. Mitigation: trap-door audit #3 has grep; substrate-as-teacher cascade absorbs surfaced sites
- **check.rs TypeDef registration pattern unclear** — Stone 234.2a's BRIEF carried the same audit item; sonnet investigates by reading existing registrations. Mitigation: trap-door audit #4
- **Stone 234.1 probe semantic regression** — extremely unlikely (rename preserves all observables) but would surface as test failure. Mitigation: probe IS the regression guard; STOP-8 fires hard

## Rank-up demonstration — Inquisitor/Shadowdancer party-comp

Per [[party-comp-inquisitor-shadowdancer]]: orchestrator (Inquisitor — Cipher Psion + Paladin Goldpact Knight) marked the rename target via empirical grep + sub-DESIGN + FM 2-bis probe + initial-FAIL verified clean. Sonnet (Shadowdancer — Helwalker Monk + Streetfighter Rogue) strikes-to-kill in the cascade.

For Stone 234.1.5 specifically, the cascade should be SHALLOW (18 enumerated sites; mechanical pattern updates). This is the easy fight in the dungeon — the Helwalker's bloodied condition is minimal. The party-comp's complementarity should produce a clean below-band win.

Capture in SCORE:
- Cases where the empirical grep enumeration (18 sites) matched the actual cascade
- Whether check.rs TypeDef registration was straightforward or required investigation
- Whether Stone 234.1 probe required ANY semantic adjustment (predicted: NO; purely mechanical pattern update)
- Substrate-as-teacher cascade depth (predicted: 0-2 unexpected sites; expect cargo to surface only the 18 enumerated)

## Out-of-scope rows (REJECTED)

- New substrate primitives (Stone 234.2a)
- defrecord macro (Stone 234.2b)
- Per-class type registration (Stone 234.2b)
- Polymorphic verb extensions (Stone 234.3)
- `:wat::holon::to-holon` wat__record arm (later stone)
- Hash-destructure (Stone 234.4)
- Migration sweep (Stone 234.6)
- holon-rs touched (STOP-4)
- Parallel API or aliases (HARD CUT per D10)
- Stone 234.2a in-flight artifacts revision (β.ii orchestrator paperwork)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to variant rename + cascade + type registration
- STOP-2: baseline regress below 827
- STOP-3: 45 min elapsed
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — new primitives, defrecord macro, per-class types, polymorphic verbs
- STOP-7: new probe doesn't flip 5/5 PASS
- STOP-8: Stone 234.1 regression guard regresses
- STOP-9: Stone 234.0 polymorphic type probe regresses
- STOP-10: Stone 232.0a regression guard regresses
- STOP-11: any arc 233 regression guard regresses

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.5.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 13-row scorecard with verbatim verification command outputs
- Per-section line counts (runtime.rs / edn_shim.rs / closure_extract.rs / check.rs / Stone 234.1 probe updates)
- Cascade depth (cargo errors surfaced + addressed; predicted 0-2 unexpected)
- Time breakdown
- Calibration band actual vs predicted (15-30 target; 45 STOP)
- Rank-up evidence — predecessor stones' tools (Stone 234.1 probe template / arc 109 `__` convention / check.rs primitive type registration precedents) shortening iteration
- Honest deltas if any surface (did the cascade match the 18-site prediction? did check.rs registration mirror an existing primitive cleanly? did Stone 234.1 probe stay semantically green?)

## What this unblocks

- **β.ii (orchestrator paperwork)** — revise the now-superseded Stone 234.2a artifacts at `db39ebd` + `7113c51` to use `:wat::record::*` shape (sub-DESIGN, BRIEF, EXPECTATIONS, probe; rewritten under new namespace + `to-map` family)
- **β.iii (revised Stone 234.2a)** — mint `:wat::record::of` + `:wat::record::field-at` substrate primitives + TypeSchemes on settled foundation
- **Stone 234.2b** — defrecord macro (`:wat::record::def`) consumes settled `:wat::record::*` primitives + emits per-class types under `:wat::record::*` umbrella
- **Stone 234.3** — polymorphic record-y verbs at `:wat::record::*` operate on settled wat__record variant
- **Stones 234.4-6** — all operate on the settled foundation Stone 234.1.5 lands

Six future stones inherit clean settled substrate. The interstitial ceremony cost (~15-30 min for β.i) is recouped multiple times over.

## The corrective stone

Stone 234.0 was the gear-check (~38 min, ZERO iteration). Stone 234.1 was the BIGGER fight (~30 min UNDER 60-120 band). Stone 234.1.5 is the FOUNDATION-CORRECTION — the rename that honors intueri's twin verdicts + the user's composed-from principle.

This stone is qualitatively different from its predecessors: it's NOT a behavior addition; it's NAME HONESTY. The Inquisitor/Shadowdancer build's strength here is precision — every site enumerated, every pattern mechanical, every regression guard load-bearing. No new substrate; just the renamed substrate, settled clean, ready for the macro work to follow.

Per the stepping-stone discipline: simple steps enable complex steps. β.i is the simple step. β.iii + 234.2b inherit a settled foundation.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.1.5.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.5.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (post-pivot)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — variant-minting predecessor SCORE
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — type-primitive predecessor SCORE
- `tests/probe_arc234_stone15_namespace_promotion.rs` — FM 2-bis probe (5 contracts; 5/5 FAIL initial verified)
- `tests/probe_arc234_stone1_wat_record_variant.rs` — Stone 234.1's regression guard (β.i updates variant pattern; stays GREEN)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
