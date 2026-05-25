# EXPECTATIONS — Stone 236.0

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **CheckResult probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

## Prediction

**Target:** 25-45 min Mode A. **Upper:** 60 min (STOP-3).

Surface: ~80-150 lines net in src/check.rs (type + 5 constructors + 5 accessors + 4 combinators + module doc).

Confidence: HIGH. Pure additive; no migration; clear API contract.

Risks:
- Probe `CheckError` construction shape (the test instantiates CheckError to test the constructors; sonnet picks easy variants like `CheckError::UnknownFunction` if extant; or whatever's simplest)
- Combinators preserve invariant (map/and_then on err must preserve err state); test coverage critical

## Out-of-scope (REJECTED)

- Migration of `fn infer` (Stone 236.1)
- Migration of sibling inference helpers (Stone 236.2)
- `From<Option<T>>` back-door conversions (T7)
- New module files outside check.rs scope
- holon-rs touched (STOP-4)

## SCORE

`SCORE-STONE-236.0.md` (NEW). 11-row verbatim + final API shape (any naming adjustments from sketch) + line count + module-doc text + cascade depth (likely zero for foundation) + honest deltas.
