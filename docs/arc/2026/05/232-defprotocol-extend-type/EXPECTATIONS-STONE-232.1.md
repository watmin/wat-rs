# EXPECTATIONS — Arc 232 Stone 232.1 — defprotocol + extend-type macros (BUNDLED)

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS initial-FAIL → 3/3** (the LOAD-BEARING row) | `cargo test --release --test probe_arc232_stone1_defprotocol_macros 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` |
| 3 | **FM 2-bis probe STAYS GREEN** (substrate-composition regression guard) | `cargo test --release --test probe_diagnostic_defprotocol_dispatch 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` |
| 4 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 5 | **Stone 232.0a probe** (typed-entities reflection regression guard) | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 6 | Stone 233.3 probe | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | Stone 233.2.e probe | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.l probe | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 9 | Stone 233.2.k probe | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 10 | Stone 233.1 ValueSnapshot probes | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 90–150 min Mode A
**Upper bound:** 180 min (STOP-3)
**Confidence:** medium-high — both macros follow `wat/holon/defrecord.wat` precedent; the FM 2-bis probe proves the substrate sufficiency; sonnet's calibration trend post-arc-233 stays in-band.

**Rationale:**
- `wat/holon/defprotocol.wat` — defmacro emitting N dispatchers (one per method declaration). Per-N iteration via `:wat::core::map` (defrecord precedent). ~50-80 lines.
- `wat/holon/extend-type.wat` — defmacro emitting M defns at mangled names (one per method-body). Per-M iteration + macro-time string concat. ~50-80 lines.
- `src/stdlib.rs` — two new WatSource entries. ~10 lines.
- `tests/probe_arc232_stone1_defprotocol_macros.rs` — 3 contracts mirroring FM 2-bis probe shape. ~150 lines.
- SCORE writing: ~15 min.

**Risks:**
- **Mangled name construction at macro-expand time** — verify the defrecord pattern (`keyword/to-string` + `string::concat` + `keyword/from-string`) extends to per-protocol-method suffix. Probe substrate already proves the runtime side; expand-time same primitives expected to work.
- **Per-N iteration with quasiquote+splice** — arc 227 v3 lineage proven; sonnet should mirror probe_diagnostic_macro_splice_from_let.rs verbatim for any iteration questions.
- **Method-name validation at expand time (D7)** — most complex piece; if scope creeps, DEFER per BRIEF Out-of-scope note; runtime UnknownFunction is honest.

## Rank-up demonstration

**Arc 233 + Stone 232.0a substrate is in place.** Stone 232.1's iteration cycles should be informative without diagnostic-print scaffolding:

- **If a macro-expansion error surfaces** — wat's macro errors should name the form + span. Arc 138's error coordinates + arc 233's ValueSnapshot apply at the macro layer too.

- **If a runtime test fails** — TypeMismatch errors render the actual value + provenance. defrecord instances created at let-bindings carry SymbolBound provenance.

- **If the dispatcher routes wrong** — `extract-classifier` returning the wrong name surfaces as `apply` failing with `UnknownFunction(":wrong/mangled-name", ...)` naming the precise miss.

- **If sonnet's Rust changes accidentally extend Value** — `#[wat_value]` seal compile-errors before tests run.

**Measurable property:** sonnet should iterate to green WITHOUT adding println debugging. The SCORE captures any concrete cases where the diagnostic substrate fired.

## Out-of-scope rows (REJECTED)

- Default implementations (D4 — v2 if needed)
- Multi-arg dispatch (multimethods cover this; arc 146/147)
- `satisfies?` predicate
- Protocol inheritance
- Built-in-type extension proof (Stone 232.3)
- defrecord accessor synthesis (NOT IN ARC 232)
- Method-name validation at expand time (D7 — may defer to runtime per BRIEF)
- Any Rust changes outside src/stdlib.rs (STOP-1, STOP-6)
- holon-rs touched (STOP-4)
- Parallel API or aliases (HARD CUT per D5)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to new files
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warnings above 54
- STOP-6: scope creep
- STOP-7: new macro probe doesn't PASS 3/3 (the load-bearing row)
- STOP-8: FM 2-bis probe regresses (substrate composition broke)
- STOP-9: any arc 233 regression guard regresses
- STOP-10: Stone 232.0a probe regresses
- STOP-9b (renumbered): cascade exceeds time-box — apply partial-state-grading

## SCORE doc

`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.1.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 12-row scorecard with verbatim verification command outputs
- Per-file line counts (defprotocol.wat / extend-type.wat / stdlib.rs / probe file)
- Time breakdown
- Calibration band actual vs predicted (90-150 target; 180 STOP)
- **Rank-up evidence** — any cases during iteration where arc 233 + Stone 232.0a tools (ValueSnapshot rendering, provenance display, EDN parseability, structural seals, reflection primitives) saved sonnet time or surfaced honest diagnostic context. Even small examples valuable.
- D7 deferral decision documented — whether compile-time method-name validation was implemented or deferred to runtime + WHY
- Honest deltas if any surface

## What this unblocks

- **Stone 232.3** — built-in-type extension proof (extend `:wat::holon::Vector` or similar with a sample protocol). Mostly integration-test work on the macros that Stone 232.1 ships.
- **Stone 232.5** — INSCRIPTION + USER-GUIDE chapter (arc 232 closure).
- **defrecord accessor synthesis** (separate stone OUTSIDE arc 232) — composes `Bind/right` + `Bundle/children` + name-match. Stone 232.1 method bodies use these primitives directly.

## The rank-up confirmation (the load-bearing point)

The trajectory: arc 232.0 → 232.0a → 233 (substrate enrichment) → 233 INSCRIPTION → 232 RESUME → 232.0a SHIPPED → 232.1 FM 2-bis probe SHIPPED → 232.1 (this stone).

The arc 233 detour was strategic: build the diagnostic-richness substrate BEFORE the defprotocol consumer ships, so the consumer's iteration validates the substrate. The FM 2-bis probe already showed it firing (probe 3's precise UnknownFunction message).

Stone 232.1 is the second-order rank-up confirmation: the macros themselves are AUTHORED with the diagnostic substrate available; sonnet's flight pattern post-arc-233 should mirror pre-arc-233 calibration (no regression from the enrichment), AND surface any cases where the enrichment paid off in real consumer-side iteration.

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/BRIEF-STONE-232.1.md` — paired BRIEF
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN-STONE-232.1.md` — sub-DESIGN with 8 locked decisions + canonical expansion templates
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — arc umbrella
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — predecessor SCORE
- `tests/probe_diagnostic_defprotocol_dispatch.rs` — FM 2-bis probe (3/3 PASS at `f38e120`)
- `wat/holon/defrecord.wat` — defmacro precedent
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up arc
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
