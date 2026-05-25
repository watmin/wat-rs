# EXPECTATIONS — Stone 234.4

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **234.4 probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

## Prediction

**Target:** 90–120 min. **Upper:** 150 min (STOP-3).

Surface: ~80-120 parser + ~60-100 check + ~80-120 runtime = ~220-340 lines across 3 files. Multi-file change.

Risks:
- Parser 2nd-item peek ambiguity edge cases
- Binding-scope extension type propagation for polymorphic-T
- HashMap-vs-record check-time-type discrimination (the receiver-type RHS may be polymorphic until eval — handle gracefully)

## Out-of-scope (REJECTED)

- Match-arm hash-destructure (Stone 234.4.match named follow-up)
- Per-class TypeDef registration (arc 232.1 future-lift)
- Receivers beyond {record, struct, HashMap}
- Positional-bind via brace (vec-form is positional per arc 169)
- holon-rs touched (STOP-4)

## SCORE

`SCORE-STONE-234.4.md` (NEW). 11-row verbatim + which receivers shipped + cascade depth + parser/check/runtime line counts + time + honest deltas. Per-arm deferrals MUST name successor stones (not "future cleanup").
