# EXPECTATIONS — Stone 236.3

Mode A: 12/12 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Stone 236.0 probe still PASSES** (regression check; Contract 6 doc sharpened) | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 5 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 825 passed; 0 failed (0-2 changes acceptable if traced to struct-pattern-match test updates; > 2 = STOP-2) |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 (current baseline 52 from Stone 236.2; refactor may eliminate more warnings as pattern-match clarity often reduces lints) |
| 12 | Enum-shape verified | `grep -c "^pub enum CheckResult" src/check.rs` | `1` (the new shape) AND `grep -c "^pub struct CheckResult" src/check.rs` returns `0` (old shape gone per D5 HARD CUT) |

**Note on Row 8:** Behavior is preserved; lib baseline should hold at 827. The 0-2 tolerance is for any test code that pattern-matched the OLD struct shape directly (e.g., `match result { CheckResult { value, errors } => ... }`); such tests update to enum pattern-match form. > 2 drops = migration error or unexpected behavior change.

**Note on Row 12:** Defensive grep — confirms the type definition swap completed (enum exists; struct does not).

## Prediction

**Target:** 30-45 min Mode A. **Upper:** 60 min (STOP-3).

Surface:
- Type definition swap: ~5 lines (struct → enum)
- Constructor function body updates: ~15-25 lines (5 fns; each ~3-5 lines)
- Accessor body updates: ~30 lines (5 accessors via pattern-match)
- Combinator body updates: ~50 lines (4 combinators via pattern-match)
- drain_errors_into body update: ~10 lines (pattern-match)
- Probe Contract 6 documentation sharpen: ~5 lines
- 1040-1206 docstring update: ~20-40 lines

Net: ~150-200 line touch in src/check.rs, all localized to the CheckResult definition + impl block + module docstring.

**Body construction sites at the 151 HARVEST points + ~267 drain_errors_into call sites: ZERO RENAME** (smart constructors + bridge signature preserved). This is the load-bearing property — the refactor's value comes from internal-representation honesty without consumer disruption.

Cascade depth: 1-2 compile rounds expected. The type-definition-level changes propagate through the impl block; consumer surface unchanged.

Risks:
- Test rot revealed (existing tests that pattern-matched the OLD struct shape): expected 0-2; sonnet updates to enum pattern-match form inline
- Combinator implementations need pattern-match exhaustiveness (3 variants × multiple operations); sonnet writes carefully
- `#[derive(...)]` macros on the new enum (if needed; struct may not have had any beyond default)

Pre-emption evidence (rank-up vs predecessor stones):
- 236.0 shipped ~25 min (60-90 band)
- 236.1 shipped ~25 min (60-90 band; cascade 2 vs 3-5)
- 236.2 shipped ~57 min (90-180 band; cascade 1 vs 3-5)
- 236.3 target ~30-45 min (refactor is type-definition-level; smallest cascade footprint)
- Pattern-matching boilerplate is the main code volume; mechanically derivable from the locked variant set

## Out-of-scope (REJECTED)

- Touch any body construction site at the 151 HARVEST points (smart constructors preserve compatibility)
- Touch any of the ~267 `drain_errors_into` call sites (signature preserved)
- Modify DESIGN-STONE-236.0.md / BRIEF-STONE-236.0.md / EXPECTATIONS-STONE-236.0.md / SCORE-STONE-236.0.md (D10 — inscription-immutable)
- Modify Stone 236.1 / 236.2 paperwork artifacts (D10)
- Transitional struct-and-enum coexistence (D5 HARD CUT)
- Touch any file other than src/check.rs (probe file allowed for Contract 6 doc update per D6)
- holon-rs touched (STOP-4)

## SCORE

`SCORE-STONE-236.3.md` (NEW). Capture:
- 12-row scorecard verbatim
- Type-definition + constructor + accessor + combinator + bridge code-diff summary
- Cascade depth: compile rounds + iteration cycles
- Test rot revealed (if any)
- Honest deltas if any
- Rank-up evidence — did the ZERO-RENAME body-construction property hold? Did the predecessor SCORE doc template work? Was the dialogue-as-PERCEIVE recognition-source vindicated by the empirical refactor outcome?
- Closing note: the ✅✅✅ structural impossibility shipped for arc 236's failure class. Stone 236.4 (INSCRIPTION) is the next move.
