# EXPECTATIONS — Arc 234 Stone 234.2b — `:wat::Record::def` macro

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 6/6 FAIL → 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 3 | Stone 234.2a regression guard | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 4 | Stone 234.1.5 regression guard | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 5 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 6 | Stone 234.0 regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 7 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 8 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 9 | `:wat::holon::defrecord` not regressed (co-exists) | `cargo test --release --test wat_arc227_defrecord_macro 2>&1 \| tail -5 \|\| echo "no test exists; check via lib baseline"` | tests pass OR lib baseline covers it |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 45–75 min Mode A
**Upper bound:** 90 min (STOP-3 hard cap at 60 wall-clock per BRIEF discipline; if T3 trips badly, replan and respawn)
**Confidence:** medium — wat-side macro work; 227 v3 provides ~70% of the pattern; 234.2b adds one new splice site (per-field accessor splice into `do`).

**Rationale:**
- Macro body length: ~70-90 lines (vs 227 v3's ~50 lines)
- Stdlib.rs WatSource entry: 4-7 lines (include comment header)
- Probe already committed at `676e861`; no probe authoring time
- Compile cycles: 2-4 expected (T3's accessor splice surfacing time is the unknown)
- SCORE writing: ~10 min

**Calibration precedents:**
- Stone 227.2 v3 (after probe-resolution): ~55 min
- Stone 234.2a (substrate primitives): ~58 min (band's upper edge; one trap-door investigation)
- Stone 234.1 (variant + cascade): ~30 min (band's lower edge; cascade shallower than predicted)
- Stone 234.2b estimate: ~50-65 min predicted; band's middle

**Risks:**

- **T3 per-field accessor splice into `do` body** — `~@(:wat::core::map ...)` splicing a vector of `defn` ASTs into the top-level `(:wat::core::do ...)` body has not been empirically proven for THIS specific composition (defn ASTs at the splice point). If it fails: surface immediately; do NOT workaround via Vector wrapping or alternative pattern. The substrate either supports it or a substrate-extension stone is needed. The 227 v3 pattern splices Bind ASTs into Bundle — similar but at a different level. **Trust the substrate-as-teacher cascade.**

- **`:wat::core::keyword/from-string` returning a usable head for `defn` accessor name** — the predicate name uses this pattern (227 v3); the accessor name is similar. Should work uniformly; surface if it doesn't.

- **`:wat::holon::to-wat` round-trip on type-keyword for accessor signature** — produces a WatAST::Keyword usable in the `-> :T` position of a `defn` signature. The 227 v3 macro does this for the field-name `var-w` (line 137) which appears in vector position; 234.2b uses it for type position. The vector-vs-type distinction might matter; surface if it does.

## Rank-up demonstration — Inquisitor/Shadowdancer party-comp

Per `project_party_comp_inquisitor_shadowdancer` (the build inscribed 2026-05-24 via Stone 234.1's UNDER-band ship): the orchestrator (Inquisitor — Cipher Psion + Paladin Goldpact Knight) marked the target via sub-DESIGN + FM 2-bis probe + initial-FAIL verification; sonnet (Shadowdancer — Helwalker Monk + Streetfighter Rogue) strikes-to-kill in the cascade.

For Stone 234.2b specifically:
- The dungeon-mapping is wat-side (NEW territory: per-field accessor splice — T3). The pre-emption is intentionally CONSERVATIVE — the sub-DESIGN names T3 as empirical risk; the BRIEF directs "trust the substrate, surface the diagnostic if it fails." Sonnet's discipline: do NOT workaround; report the diagnostic if T3 fires badly.
- The bloodied condition is the macro-expand-time error surface — wat macro errors are notoriously terse; sonnet may need 2-3 iteration cycles to localize the cause.
- The complementarity continues: orchestrator focused (marked the gear; spec'd the contract); sonnet bloodied-in-cascade (executes against the wat-macro surface).

Capture in SCORE:
- Cases where 227 v3's macro body shortened authoring (expected: the holon_form construction is verbatim-mirrored)
- The expand-time cascade depth (predicted: 2-4 cycles; 0 cycles would vindicate the conservative pre-emption)
- Any trap-door items that fired (T1-T8) with concrete diagnostics
- Did T3 (accessor splice into `do`) work cleanly OR require alternate composition

## Out-of-scope rows (REJECTED)

- Runtime class-safety check in accessor bodies (Stone 234.2c — named follow-up)
- Field-type constraint enforcement at macro-expand time (Stone 234.2c+ — named follow-up)
- Per-class TypeDef registration (Stone 234.2c — named follow-up)
- `:wat::holon::defrecord` migration / retirement (Stone 234.6 — named follow-up)
- Polymorphic record-y verbs (Stone 234.3)
- Hash-destructure (Stone 234.4)
- `:wat::holon::*` auto-dispatch on records (Stone 234.5)
- holon-rs touched (STOP-4)
- Aliases or single-arg form (D14 HARD CUT)
- Macro registered anywhere other than `WAT_SOURCES` (D8 file location)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1** — unexpected compile errors not tracing to the new macro file or stdlib.rs entry
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 60 min elapsed (hard cap; replan if hit)
- **STOP-4** — holon-rs touched
- **STOP-5** — Rust changes outside `src/stdlib.rs`
- **STOP-6** — scope creep: per-class TypeDef registration, runtime class-safety check, field-type constraint enforcement, predicate-arity variants
- **STOP-7** — the new probe doesn't flip 6/6 PASS
- **STOP-8** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a)
- **STOP-9** — `:wat::holon::defrecord` macro behavior regresses (lib tests catch this)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION. None is a permission slot. If hit: report the diagnostic; surface to orchestrator; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Macro implementation surface: `wat/Record.wat` line count (predicted: 80-130 lines including header comments); stdlib.rs WatSource entry (4-7 lines)
- Cascade depth: compile rounds; any expand-time iteration cycles
- Time breakdown: read + author + compile + scorecard + SCORE writing
- Calibration band actual vs predicted (45-75 target; 90 STOP)
- Rank-up evidence — predecessor tools (227 v3 holon_form pattern; 234.2a substrate primitives; 234.1.5 namespace registration) shortening authoring
- T1-T8 trap-door audit: which fired with concrete diagnostics
- Honest deltas if any surface

## What this unblocks

- **Stone 234.2c** — runtime class-safety check (D10 named follow-up)
- **Stone 234.3** — polymorphic record-y verbs (assoc / record->map / record? / keyword-as-accessor); consumes `Value::wat__Record` instances that 234.2b makes ergonomic to construct
- **Stone 234.6** — migration sweep + `:wat::holon::defrecord` retirement
- **Stone 232.1 revised** — `:wat::core::defprotocol` polymorphic via `:wat::core::type`; wat-records participate naturally

## The fourth fight in arc 234's dungeon

Stone 234.0 was the gear-check (~38 min, ZERO iteration). Stone 234.1 was the BIGGER fight (~30 min UNDER 60-120 band). Stone 234.1.5 was the rename cascade (~25 min). Stone 234.2a was the FOLLOW-THROUGH (~58 min). Stone 234.2b is **the SURFACE** — the macro that turns the substrate primitives into the user-facing experience the user has been building toward.

The party-comp shipped 4/4 wins this session. We press the advantage on the fourth fight — the surface that makes the hologram visible to users.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2b.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2b.md` — sub-DESIGN with 14 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — substrate primitive predecessor SCORE
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — variant-minting predecessor SCORE
- `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2-v3.md` — macro pattern predecessor SCORE
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — FM 2-bis probe (6 contracts; 6/6 FAIL verified)
- `wat/holon/defrecord.wat` — the v3 macro source (expansion pattern template)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
