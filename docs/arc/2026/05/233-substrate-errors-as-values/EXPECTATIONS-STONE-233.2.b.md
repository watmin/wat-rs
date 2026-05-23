# EXPECTATIONS — Arc 233 Stone 233.2.b — keyword/from-string producer tag

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests baseline maintained | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed; 1 ignored |
| 3 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 4 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline match) |
| 5 | eval_keyword_from_string wraps return in Tracked | `grep -A 30 "fn eval_keyword_from_string" src/runtime.rs \| grep -c "Value::Tracked"` | ≥ 1 |
| 6 | Provenance::RuntimeBuilt used at the wrap site | `grep -A 30 "fn eval_keyword_from_string" src/runtime.rs \| grep -c "Provenance::RuntimeBuilt"` | ≥ 1 |
| 7 | Producer string is canonical | `grep -A 30 "fn eval_keyword_from_string" src/runtime.rs \| grep -c '":wat::core::keyword/from-string"'` | ≥ 2 (existing op string + new producer string) |
| 8 | ValueSnapshot::Display extended for Provenance | `grep -A 30 "impl std::fmt::Display for ValueSnapshot" src/runtime.rs \| grep -c "Provenance::"` | ≥ 1 |
| 9 | Display covers all 4 Provenance variants | `grep -A 40 "impl std::fmt::Display for ValueSnapshot" src/runtime.rs \| grep -cE "Provenance::(Unknown\|Literal\|SymbolBound\|RuntimeBuilt)"` | ≥ 4 |
| 10 | **Probe 6 flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_6 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 11 | All other 233.1 probes still PASS | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `6 passed; 0 failed` (was 5 in 233.1; +1 from Probe 6) |
| 12 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction (calibration record)

**Target runtime:** 30-60 min Mode A
**Upper bound:** 60 min (STOP-3 fires)
**Confidence:** high (small, focused scope; precedent in 233.2.a)

**Rationale:**
- eval_keyword_from_string wrap: ~5 min (add 3-4 lines)
- ValueSnapshot::Display extension: ~10-15 min (4 Provenance arms)
- Verification cascade: ~5 min
- Probe runs + SCORE: ~10 min
- Buffer for any existing-test fixes: ~20 min

**Risks:**
- Existing tests asserting EXACT error message format may break (audit + fix; in scope per "Specific trap" in BRIEF)
- Tracked-wrapping may affect downstream code paths that pattern-match the Value (unlikely — Stone 233.2.a's inner() discipline should have established transparency throughout)

**Calibration check:**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows (REJECTED if attempted)

- Other producers (233.2.c)
- AST-derived provenance (233.2.d)
- Errors-as-EDN (233.3)
- holon-rs touch
- wat-edn touch
- Any aliases / deprecation shims
- "Stub", "future arc", "deferred to" language in SCORE

## Honesty deltas accepted

- Existing test assertions on RuntimeError message format may need CONTAINS-not-EXACT updates (small fix; documents the new richer format)
- Exact span coordinates in Display output may use different formatting (e.g., `<file>:<line>:<col>` vs `<file>(<line>,<col>)`); sonnet picks readable

## Honesty deltas NOT accepted (STOP triggers fire)

- Baseline lib tests regress — STOP-2
- Probe 6 still FAILS — STOP-7
- 233.2.a transparency tests regress — STOP-8
- Sonnet tags producers beyond keyword/from-string — STOP-6
- Sonnet edits holon-rs — STOP-4

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** baseline tests regress
- **STOP-3:** 60 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning
- **STOP-6:** scope creep
- **STOP-7:** Probe 6 still FAILS
- **STOP-8:** transparency tests regress
- **STOP-9:** Display impl breaks existing assertions beyond CONTAINS-update fixes

## SCORE doc

SCORE will live at `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.b.md`. Per `feedback_inscription_immutable`, that's a NEW file. Body cites each row's verification command + result + any honest delta.

## What this unblocks

- **233.2.c** — pattern established; sweep additional producers using same template (eval_X wraps return in Tracked with appropriate RuntimeBuilt producer)
- **233.2.d** — AST-derived provenance for let-bindings + literals
- **First user-visible proof** that the diagnostic-richness work pays off — the runtime-built case from INVENTORY § O three-case table now teaches
