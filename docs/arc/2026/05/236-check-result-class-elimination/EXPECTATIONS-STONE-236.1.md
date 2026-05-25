# EXPECTATIONS — Stone 236.1

Mode A: 11/11 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Stone 236.0 probe still PASSES** (regression check) | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 5 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed (or document drops as harvest-surfaced behavior shifts) |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |

**Note on Row 8:** The HARVEST may surface previously-silent failure sites; these may cause 1-2 lib test changes if a test was relying on silent failure. Per BRIEF: 1-2 changes acceptable + documented in SCORE; > 5 = STOP-2.

## Prediction

**Target:** 60-90 min. **Upper:** 120 min (STOP-3).

Surface:
- Primary infer body translation: 50-80 lines modified in 125-line body
- ~126 call-site bridge insertions: mostly mechanical `.drain_errors_into(errors)` chain
- 0-5 new CheckError variants if harvest surfaces previously-silent failure semantics

Cascade depth: 3-5 compile rounds expected (signature flip surfaces all call sites; each round of fixes triggers next round).

Risks:
- HARVEST classification debate at edge cases (sonnet flags; orchestrator reviews)
- Call sites without `errors: &mut Vec<CheckError>` in scope (rare; restructuring needed)
- Test surfaces previously-silent diagnostic site → expected behavior shift

## Out-of-scope (REJECTED)

- Sibling infer_* function flips (Stone 236.2)
- Transitional dual-channel shim like `infer_v2()` (D8)
- Touch any file other than src/check.rs (STOP-5)
- holon-rs touched (STOP-4)

## SCORE

`SCORE-STONE-236.1.md` (NEW). Capture:
- 11-row scorecard verbatim
- HARVEST classification counts (D3: 1/2/3 per type)
- Any new CheckError variants (their names + why)
- Cascade depth + iteration cycles
- Lib-test changes (per row 8 note; documented as harvest-surfaced)
- Honest deltas if any
- Rank-up evidence — predecessor pattern (arc 233 failure-engineering cascade) effectiveness
