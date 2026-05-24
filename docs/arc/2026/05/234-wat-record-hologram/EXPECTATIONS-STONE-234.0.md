# EXPECTATIONS — Arc 234 Stone 234.0 — `:wat::core::type` polymorphic primitive

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 0/8 → 8/8** (LOAD-BEARING) | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `test result: ok. 8 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 5 | Stone 233.3 regression guard | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | Stone 233.2.e regression guard | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | Stone 233.2.l regression guard | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 8 | Stone 233.2.k regression guard | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 9 | Stone 233.1 ValueSnapshot guard | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 30–60 min Mode A
**Upper bound:** 90 min (STOP-3)
**Confidence:** high — smallest substrate addition in arc 234; one eval fn + one dispatch arm + one TypeScheme entry. Apply primitive (Stone 232.0; ~30 min) is the precedent for "single polymorphic substrate verb."

**Rationale:**
- `eval_type` Rust fn: ~30 lines (match on Value variants; return Value::String)
- Dispatch arm in `dispatch_keyword_head_value`: 1 line
- `register_builtins` entry in `src/check.rs`: ~10 lines (TypeScheme registration)
- Possibly: `infer_list` special-case for polymorphic T inference: ~5-10 lines if needed
- Compile + iterate cycles: ~5 min (substrate is well-precedented)
- SCORE writing: ~10 min

**Risks:**
- **Polymorphic TypeScheme inference** — the `TypeExpr::Var("T")` may or may not propagate correctly through ordinary inference. If not, add an `infer_list` special-case mirroring `infer_apply`'s pattern (Stone 232.0 precedent).
- **HolonAST extract_classifier fallback** — straightforward `unwrap_or_else`; precedent verified during sub-DESIGN.
- **Struct type_name leading colon stripping** — `trim_start_matches(':')` per D2; verified during sub-DESIGN.

## Rank-up demonstration

**Arc 233 + Stone 232.0a substrate is in place.** Stone 234.0's iteration cycles should be informative without diagnostic-print scaffolding:

- **If a probe fails with TypeMismatch on the new TypeScheme** — error names the unifier mismatch + provenance of the offending value.
- **If a probe fails with UnknownFunction post-dispatch-arm-addition** — the arm isn't routing; verify the keyword string matches verbatim.
- **If a probe surfaces an unexpected type string** — error message renders the actual `String` value via ValueSnapshot; sonnet sees "got 'wat::core::Struct' expected 'myapp::Point'" instantly without println scaffolding.

The substrate teaches. Trust the cargo loop.

## Out-of-scope rows (REJECTED)

- `Value::wat_record` variant (Stone 234.1)
- defrecord macro (Stone 234.2)
- Record-y polymorphic verbs (Stone 234.3)
- Hash-destructure (Stone 234.4)
- `:wat::holon::*` auto-dispatch (Stone 234.5)
- Migration sweep (Stone 234.6)
- holon-rs touched (STOP-4)
- Parallel API or aliases (HARD CUT per D5)
- Any Rust changes outside `src/runtime.rs` + `src/check.rs` (STOP-1, STOP-6)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to new eval_type / dispatch arm / TypeScheme entry
- STOP-2: baseline regress below 827
- STOP-3: 90 min elapsed
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep
- STOP-7: new probe doesn't PASS 8/8 (the load-bearing row)
- STOP-8: any arc 233 regression guard regresses
- STOP-9: Stone 232.0a probe regresses

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Per-section line counts (eval_type / dispatch arm / TypeScheme / any infer_list special-case)
- Time breakdown
- Calibration band actual vs predicted (30-60 target; 90 STOP)
- Rank-up evidence — any cases during iteration where arc 233 + Stone 232.0a tools (ValueSnapshot rendering, provenance display, EDN parseability, reflection primitives) saved time or surfaced honest diagnostic context. Even absence-of-need-for-scaffolding worth noting.
- Honest deltas if any surface (e.g., did `infer_list` need a special-case? did the polymorphic TypeScheme propagate as expected?)

## What this unblocks

- **Revised Stone 232.1** — `:wat::core::defprotocol` + `:wat::core::extend-type` polymorphic via `:wat::core::type` (no longer at `:wat::holon::*` per arc 234 doctrine + DR-branched superseded ship at `dr/stone-232.1-holon-only`)
- **Stone 234.1** — `Value::wat_record` variant (type primitive extends with one arm for the new variant)
- **Stone 234.3** — polymorphic record-y verbs (all consume `:wat::core::type` for dispatch routing)
- **All subsequent arc 234.x stones**

## The first fight in arc 234's dungeon

This is sonnet's first step into the wat-record hologram arc. Stone 234.0 is the smallest known piece — well-precedented (apply primitive shape), small surface (~50 lines total), narrow scope (one primitive). It serves as the gear-check before the bigger stones (234.1 Value variant; 234.2 defrecord macro; 234.3 polymorphic family) consume it.

The FM 2-bis probe is the success criterion. The probe IS what the dungeon asks; sonnet's substrate IS the answer.

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.0.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md` — sub-DESIGN with 6 locked decisions + dispatch table
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `tests/probe_diagnostic_polymorphic_type.rs` — FM 2-bis probe (8 contracts; commit `529760b`)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
