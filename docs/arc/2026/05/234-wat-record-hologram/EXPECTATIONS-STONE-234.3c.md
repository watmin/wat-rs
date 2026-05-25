# EXPECTATIONS — Stone 234.3c

Mode A: 11/11 PASS (or 10/11 if struct arm is deferred — see Row 2 note).

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **234.3c probe flips** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -5` | `6 passed; 0 failed` (or `5 passed; 1 failed` if struct arm deferred — document in SCORE) |
| 3 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

## Prediction

**Target:** 60–90 min Mode A. **Upper:** 120 min (STOP-3).

Surface: ~80-150 lines runtime + ~30-50 lines check.rs. Medium scope.

Risks:
- **T6 check.rs polymorphic-T** — may need custom inference handler (Stone 234.2a-CORRECTION's infer_record_of pattern)
- **T5 Struct arm scaffolding** — if struct fixture in probe 6 proves heavy, defer that arm + ship 5/6 with documented deferral as named follow-up stone (not "future cleanup")
- **HashMap key construction** — verify Value::wat__core__keyword's expected format (with colon per 234.2a SCORE D5)

## Out-of-scope (REJECTED)

- Check-time field narrowing (Stone 234.2c+ / future arc 232.1 lift)
- Per-class TypeDef registration
- Receivers beyond {record, struct, HashMap}
- Alternative naming verbs (`:wat::core::field-of` etc.)
- holon-rs touched (STOP-4)

## SCORE

`SCORE-STONE-234.3c.md` (NEW). 11-row verbatim + which arms shipped + cascade depth + time + honest deltas. If struct arm deferred: explicit named follow-up stone (e.g., "Stone 234.3c.struct"), NOT "future cleanup."
