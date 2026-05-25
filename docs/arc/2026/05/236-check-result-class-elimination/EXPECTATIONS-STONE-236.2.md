# EXPECTATIONS — Stone 236.2

Mode A: 12/12 PASS.

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
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 822 passed; 0 failed (1-5 changes acceptable if traced to HARVEST Classification 2; > 5 = STOP-2) |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | Sibling sig flip verified | `grep -c "^fn infer_.*errors: &mut Vec<CheckError>" src/check.rs` | `0` (all 47 siblings flipped; zero retain the legacy errors param) |

**Note on Row 8:** The HARVEST is expected to surface MORE previously-silent sites than 236.1 did (per Stone 236.1's own SCORE: "the silent failures live in the sibling functions — 236.2's territory"). 1-5 lib-test changes acceptable + documented in SCORE; > 5 = STOP-2.

**Note on Row 12:** Defensive grep — confirms NO sibling retains the old `errors: &mut Vec<CheckError>` parameter. Zero matches proves uniform flip across all 47.

## Prediction

**Target:** 90 min Mode A. **Upper:** 180 min (STOP-3).

Surface:
- 47 sibling fn body translations: ~30-100 lines modified per fn (~1500-4700 total lines touched)
- ~111 sibling-call site bridge insertions: mostly mechanical `.drain_errors_into(...)` chain
- Primary fn infer's calls to siblings flip from legacy `&mut local_errors` to bridge form
- Hot spots: `infer_list` (~1273 line body; dispatch hub for 30+ siblings) and `infer_match` (~1279 line body)
- 0-5 new CheckError variants if harvest surfaces previously-silent failure semantics
- HARVEST aggregate count (Classification 1 / 2 / 3) across all 47 siblings

Cascade depth: 3-5 compile rounds expected (signature flip surfaces all sibling-call sites; each round of fixes triggers next round). Sibling↔sibling calls inside `infer_list` create the widest single-fn cascade.

Risks:
- HARVEST classification debate at edge cases (sonnet flags; orchestrator reviews aggregate)
- `infer_list`'s wide internal cascade may dominate iteration time
- Sibling param signature variance — some siblings may have additional non-standard params (T6 trap-door); sanity check before mechanical flip
- Test surfaces previously-silent diagnostic site → expected behavior shift (Classification 2 yield expected > 0)

Pre-emption evidence (rank-up vs 236.1):
- 236.1 cascade depth 2 vs predicted 3-5 (under prediction)
- 236.1 runtime ~25 min vs 60-90 band (under prediction)
- Party-comp's pre-emption + bridge-tool maturity reduces actual work below pessimistic estimates
- 236.1's success informs 236.2's 90 min target (rather than naive 47x = 1175 min projection)

## Out-of-scope (REJECTED)

- Primary `fn infer()` signature changes (already shipped by 236.1; STOP-6)
- Transitional dual-channel shim like `infer_<verb>_v2()` (D8 HARD CUT)
- Touch any file other than src/check.rs (STOP-5)
- holon-rs touched (STOP-4)
- New probe files (D6; existing scorecard regression-guards sufficient)
- HARVEST remediation arc work (that's Stone 236.3 — silent-failure sites surface in 236.2 but get remediated in 236.3 if behavior fix needed)

## SCORE

`SCORE-STONE-236.2.md` (NEW). Capture:
- 12-row scorecard verbatim
- HARVEST aggregate (D3): total Classification 1 / 2 / 3 counts across 47 siblings
- Per-fn HARVEST table (which siblings contributed which classifications + counts; dominant Classification-2 producers named)
- Any new CheckError variants minted (their names + why)
- Cascade depth + iteration rounds
- Per-classification narrative
- Lib-test changes (per Row 8 note; documented as harvest-surfaced)
- Honest deltas if any
- Rank-up evidence — was Stone 236.1's SCORE doc a useful template? Did the 1040-1206 migration-pattern docstring help? Pre-emption effectiveness?
