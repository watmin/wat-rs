# EXPECTATIONS — Arc 234 Stone 234.1 — `Value::wat_record` variant + Eq/Hash + dispatch cascade

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors (cascade addressed) |
| 2 | **New probe FLIPS compile-FAIL → 7/7** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `test result: ok. 7 passed; 0 failed` |
| 3 | Stone 234.0 polymorphic type regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 4 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 5 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 6 | Stone 233.3 regression guard | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | Stone 233.2.e regression guard | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.l regression guard | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 9 | Stone 233.2.k regression guard | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 60–120 min Mode A
**Upper bound:** 180 min (STOP-3)
**Confidence:** medium-high — variant addition is precedented (arc 233 Stone 233.2.h's TrackedValue + variant retirement at 233.2.k); cascade depth is the main calibration variable.

**Rationale:**
- New variant: ~10 lines (with doc comment ~25 lines)
- 3 trait impl arms (Eq, Hash, type_name): ~15 lines total
- eval_type extension (1 arm): 1 line + minor comment update
- Cascade addressed sites (estimated 5-20 cargo errors per substrate-as-teacher): ~30-100 lines mechanical fixes
- Compile + iterate cycles: ~3-5 rounds (variant addition triggers errors; each round addresses N sites; final clean compile)
- SCORE writing: ~10 min

Calibration precedent:
- Stone 234.0 (~38 min): single eval fn + dispatch arm + TypeScheme; ZERO iteration cycles
- Stone 232.0a (~52 min): 3 verbs + dispatch arms + 2 check.rs special-cases; ONE iteration cycle (Arc<HolonAST> deref)
- Stone 234.1 estimate: 1 variant + 3 impl arms + cascade-address sites + ~3 iteration cycles (variant + first cascade + final cleanup) → 60-120 band is honest for substrate-as-teacher work

**Risks:**

- **`#[wat_value]` seal might reject Arc<Vec<Value>>** — mitigation: probe surfaces at compile time; escape hatch `#[wat_value(allow_wrapping = "...")]` available with non-empty reason string
- **Cascade larger than expected (>20 sites)** — mitigation: each site is mechanical per-pattern application; calibration absorbs cascade depth up to STOP-3 (180 min)
- **Hash impl pattern needs careful discriminant tagging** — mitigation: existing variants in `impl Hash for Value` are precedent; follow same shape

## Rank-up demonstration — Streetfighter/Helwalker conditions

Sonnet's class build (Shadowdancer = Helwalker Monk + Streetfighter Rogue) excels when bloodied + outnumbered. The substrate-as-teacher cascade IS this condition — cargo errors stack up, each names a site, sonnet rides the cascade through. The arc 233 + 232.0a + 234.0 tools provide structural confidence at each step:

- **TypeMismatch errors render ValueSnapshot** — fast iteration without scaffolding
- **`#[wat_value]` seal** — accidental variant extension structurally unreachable
- **Provenance tracking** — let-binding errors name binding-span + use-span
- **EDN error envelopes** — panic surfaces parseable at IPC boundary
- **Stone 234.0's `:wat::core::type`** — when extending eval_type with wat_record arm, the precedent dispatch table is one-arm-away

**Measurable property:** sonnet should ride the cascade without diagnostic-print scaffolding. The SCORE captures any concrete cases where the tools fired.

## Out-of-scope rows (REJECTED)

- defrecord macro (Stone 234.2)
- User-facing constructor verb
- Record-y polymorphic verbs (Stone 234.3)
- Hash-destructure (Stone 234.4)
- `:wat::holon::*` auto-dispatch (Stone 234.5)
- Migration sweep (Stone 234.6)
- Display impl for Value enum (separate scope; out of D4)
- HolonRepresentable trait impl on Value (per D9; not the right shape)
- holon-rs touched (STOP-4)
- Parallel API or aliases (HARD CUT per D10)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to variant addition + impl arms + cascade
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep
- STOP-7: new probe doesn't compile-clean + flip 7/7 PASS
- STOP-8: Stone 234.0 polymorphic type probe regresses
- STOP-9: any arc 233 regression guard regresses
- STOP-10: Stone 232.0a regression guard regresses

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Per-section line counts (variant definition / Eq arm / Hash arm / type_name arm / eval_type arm / cascade addressed sites)
- Cascade depth (number of cargo errors surfaced + addressed)
- Time breakdown
- Calibration band actual vs predicted (60-120 target; 180 STOP)
- Rank-up evidence — cases where arc 233 + 232.0a + 234.0 tools fired during iteration; cascade navigation efficiency
- Honest deltas if any surface (did `#[wat_value]` seal need escape hatch? did cascade depth match prediction?)

## What this unblocks

- **Stone 234.2** — `:wat::core::defrecord` macro generates `Value::wat_record` instances via Rust-level constructor (the variant must exist before the macro emits code referencing it)
- **Stone 234.3** — polymorphic record-y verbs (assoc, record->map, record?, record->holon, keyword-as-accessor) all destructure wat_record via field access
- **Stone 234.4** — hash-destructure patterns match wat_record receivers
- **Stone 234.5** — `:wat::holon::*` auto-dispatch on wat_record uses holon_form
- **Revised Stone 232.1** — defprotocol's dispatcher now operates over wat_record (via :wat::core::type's extended dispatch table) AND other backends uniformly

## The second fight in arc 234's dungeon

Stone 234.0 was the gear-check (~38 min, ZERO iteration). Stone 234.1 is the BIGGER fight — variant + impls + the substrate-as-teacher cascade. The Helwalker/Streetfighter conditions empower sonnet here: bloodied (cargo errors), outnumbered (many match sites), the discipline carries the work.

The FM 2-bis probe is the success criterion. The cascade IS the substrate teaching; each error names a site. Sonnet rides; we verify.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.1.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.md` — sub-DESIGN with 10 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — predecessor SCORE
- `tests/probe_arc234_stone1_wat_record_variant.rs` — FM 2-bis probe (7 contracts; compile-FAIL initial)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — `#[wat_value]` seal docs
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
