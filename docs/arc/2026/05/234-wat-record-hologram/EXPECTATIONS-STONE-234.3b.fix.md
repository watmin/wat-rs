# EXPECTATIONS — Stone 234.3b.fix

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **234.3b probe stays GREEN** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

Note: Row 10 specifically checks the errors-as-EDN regression guard since this stone touches `runtime_error_edn.rs`. Critical that EDN serialization stays correct for all variants.

## Prediction

**Target:** 15–30 min Mode A. **Upper:** 45 min (STOP-3).

Surface: ~20-40 lines across 2-3+ files. Variant decl + 2 EDN-rs arms + eval site migration + any other exhaustive matches.

Risks:
- **Other exhaustive sites** — beyond runtime_error_edn.rs, may be Display impls, panic-EDN encoders, or test helpers. Substrate-as-teacher surfaces them via compile errors; sonnet adds arms uniformly.
- **EDN map shape** — UnknownField has 4 fields; the EDN serializer map needs to accept 4 entries. Existing helper may only have `map3` etc.; sonnet may need a `map4` helper or inline construction.
- **`available` vector population during walk** — eval_record_assoc's existing holon_form walk needs minor extension to collect names. ~3-5 extra lines.

## SCORE

`SCORE-STONE-234.3b.fix.md` (NEW). 11-row outputs + implementation surface + cascade depth (which files surfaced compile errors) + time + honest deltas.
