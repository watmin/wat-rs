# EXPECTATIONS — Arc 234 Stone 234.2a — `:wat::Record::of` + `/field-at` substrate primitives

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 7/7 FAIL → 7/7 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2a_wat_record_primitives 2>&1 \| tail -5` | `test result: ok. 7 passed; 0 failed` |
| 3 | Stone 234.1 variant regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 4 | Stone 234.0 polymorphic type regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 5 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 6 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 7 | Stone 233.3 regression guard | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.e regression guard | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 9 | Stone 233.2.l regression guard | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 10 | Stone 233.2.k regression guard | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 30–60 min Mode A
**Upper bound:** 90 min (STOP-3)
**Confidence:** high — mirrors Stone 234.0 shape (substrate primitives + TypeSchemes + probe).

**Rationale:**
- 2 eval fns: ~60-80 lines total (each ~30-40)
- 2 dispatch arms in dispatcher: 2 lines + small comment block
- 2 TypeScheme registrations in check.rs: ~20 lines
- 1 TypeDef registration for `:wat::Record`: ~5-10 lines
- ~150 lines probe (already committed at `db39ebd`; no probe authoring time)
- Compile + iterate cycles: ~2-4 rounds (substrate-as-teacher cascade SHALLOW because no variant addition)
- SCORE writing: ~10 min

**Calibration precedents:**
- Stone 234.0 (~38 min): single eval fn + single dispatch arm + single TypeScheme; ZERO iteration cycles
- Stone 234.1 (~30 min): variant addition + 4 impl arms + cascade; 60-120 band shipment UNDER expected
- Stone 232.0a (~52 min): 3 verbs + dispatch arms + 2 check.rs special-cases; ONE iteration cycle
- Stone 234.2a estimate: 2 primitives + 1 type registration → ~40-50 min predicted; band's middle

**Risks:**

- **`Value::Vec` Arc-ownership confusion** — `struct_form` arg arrives as `Value::Vec(Arc<Vec<Value>>)`; cloning the existing Arc is correct; re-wrapping with `Arc::new(...)` is wrong. Mitigation: trap-door audit #1 in BRIEF
- **Polymorphic `field-at` generic-T return inference** — TypeScheme uses `ret: t_var()`; probe relies on recipient inference (defn return-type) to drive T's unification. If inference fails in probe 5, address by reading existing polymorphic primitives' check.rs pattern (e.g., how `Vec/get` returns T). Mitigation: trap-door audit #4 in BRIEF
- **TypeDef registration mechanism unclear** — verify the registration approach for opaque primitive types in check.rs before authoring. Mitigation: trap-door audit #5 in BRIEF; investigate existing `:wat::core::String` / `:wat::holon::HolonAST` registrations

## Rank-up demonstration — Inquisitor/Shadowdancer party-comp

Per [[party-comp-inquisitor-shadowdancer]]: the orchestrator (Inquisitor — Cipher Psion + Paladin Goldpact Knight) marked the target via sub-DESIGN + FM 2-bis probe; sonnet (Shadowdancer — Helwalker Monk + Streetfighter Rogue) strikes-to-kill.

For Stone 234.2a specifically, the cascade should be SHALLOW (no variant addition; just new fns + new dispatch arms + new TypeSchemes). This is the gear-check rhythm of Stone 234.0 — strike-to-kill applies but the bloodied condition is less acute. The party-comp's complementarity should produce a clean ~30-60 min win.

Capture in SCORE:
- Cases where Stone 234.0's eval_type / Stone 234.1's variant fields / Stone 232.0's apply primitive shortened authoring
- The substrate-as-teacher cascade depth (predicted shallow: 0-5 sites)
- Any trap-door audit items that fired with concrete diagnostics
- Did `record/field-at` generic-T inference work cleanly OR need a check.rs adjustment

## Out-of-scope rows (REJECTED)

- defrecord macro (Stone 234.2b)
- Per-class type registration (`:myapp::Voltage` as alias of `:wat::Record`)
- User-facing constructor verbs (`:myapp::Voltage`)
- Predicates (`:myapp::is-Voltage?`)
- Named per-field accessors (`:myapp::Voltage/magnitude`)
- Record-y polymorphic verbs (Stone 234.3)
- Hash-destructure (Stone 234.4)
- `:wat::holon::*` auto-dispatch (Stone 234.5)
- Migration sweep (Stone 234.6)
- holon-rs touched (STOP-4)
- Parallel API or aliases (HARD CUT per D10)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to new primitives + type registration
- STOP-2: baseline regress below 827
- STOP-3: 90 min elapsed
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — defrecord macro, per-class types, user-facing constructors, predicates, named accessors
- STOP-7: new probe doesn't flip 7/7 PASS
- STOP-8: Stone 234.1 wat_record variant probe regresses
- STOP-9: Stone 234.0 polymorphic type probe regresses
- STOP-10: Stone 232.0a regression guard regresses
- STOP-11: any arc 233 regression guard regresses

## SCORE doc

`docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.2a.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 12-row scorecard with verbatim verification command outputs
- Per-section line counts (eval_wat_record_of / eval_wat_record_field_at / dispatch arms / TypeSchemes / TypeDef registration)
- Cascade depth (cargo errors surfaced + addressed; predicted shallow)
- Time breakdown
- Calibration band actual vs predicted (30-60 target; 90 STOP)
- Rank-up evidence — predecessor stones' tools (Stone 234.0's eval_type / Stone 234.1's variant fields / Stone 232.0's apply primitive) shortening iteration
- Honest deltas if any surface (did `Value::Vec` Arc-ownership trip? did generic-T inference work cleanly? did TypeDef registration follow precedent or need a new pattern?)

## What this unblocks

- **Stone 234.2b** — `:wat::core::defrecord` macro consumes these primitives. Macro emits:
  - Per-class constructor as `(defn :myapp::Voltage [<args>] (:wat::Record::of "myapp::Voltage" [<args>] <holon-form>))`
  - Per-field accessor as `(defn :myapp::Voltage/<field> [v <- :myapp::Voltage] -> :T (:wat::Record/field-at v <index>))`
- **Stone 234.3** — record-y polymorphic verbs (assoc, record->map, record?, keyword-as-accessor) destructure wat_record via field access + use field-at + use of for reconstruction
- **Per-class type registration in 234.2b** — `:myapp::Voltage` registered as alias of `:wat::Record` with class_fqdn invariant

## The third fight in arc 234's dungeon

Stone 234.0 was the gear-check (~38 min, ZERO iteration). Stone 234.1 was the BIGGER fight (~30 min UNDER 60-120 band; cascade depth 3 vs 5-20 predicted). Stone 234.2a is the FOLLOW-THROUGH — two new substrate primitives that the macro will consume. The hologram now has BOTH storage (234.1) AND substrate verbs to construct/access it (234.2a). The macro (234.2b) is the visible-to-user layer; everything before it is the load-bearing infrastructure.

The party-comp's complementarity (Inquisitor mark + Shadowdancer strike) has shipped 2/2 wins this session. We press the advantage.

## Cross-references

- `docs/arc/2026/05/234-record-hologram/BRIEF-STONE-234.2a.md` — paired BRIEF
- `docs/arc/2026/05/234-record-hologram/DESIGN-STONE-234.2a.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.1.md` — variant-minting predecessor SCORE
- `docs/arc/2026/05/234-record-hologram/SCORE-STONE-234.0.md` — type-primitive predecessor SCORE
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent
- `tests/probe_arc234_stone2a_wat_record_primitives.rs` — FM 2-bis probe (7 contracts; 7/7 FAIL verified at `db39ebd`)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
