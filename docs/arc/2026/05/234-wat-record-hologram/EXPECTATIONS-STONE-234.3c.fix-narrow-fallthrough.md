# EXPECTATIONS — Stone 234.3c.fix-narrow-fallthrough

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe 4/4 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -5` | `4 passed; 0 failed` (or `3 passed; 1 failed` if probe 4 deferred with NAMED stone) |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

## Prediction

**Target:** 20-40 min Mode A. **Upper:** 60 min (STOP-3).

Surface: ~20-40 lines check.rs.

Risks:
- Lib tests may surface consumer reliance on over-permissive behavior — REPORT honestly; do not auto-fix
- `apply_subst` helper name / `env.types.is_struct` predicate may be different than sketched — sonnet investigates and adjusts

## SCORE

`SCORE-STONE-234.3c.fix-narrow-fallthrough.md` (NEW). Capture line counts + receiver-discrimination predicate used + cascade depth + any consumer-reliance surface.
