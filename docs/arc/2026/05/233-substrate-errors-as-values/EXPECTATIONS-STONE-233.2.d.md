# EXPECTATIONS — Arc 233 Stone 233.2.d — substrate-symmetry uniform `list_span` threading

Mode A target: **13/13 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Substrate-symmetry probe FLIPS to PASS** | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -5` | `test result: ok. 1 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 5 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | Stone 232.0 dynamic-keyword probes still pass | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline match) |
| 8 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |
| 9 | All 133 dispatch arms now thread list_span | probe output | `Counts: 376 compliant; 6 exempt; 0 violations` |
| 10 | No fn body refactor (scope discipline) | diff review | only signature additions + dispatch arm updates (no eval body refactors) |
| 11 | Canonical ordering used for NEW threading | sample check of 5 modified arms | `list_span` at position 2 (after args, before env) on newly-modified arms |
| 12 | Non-dispatch caller ripples handled | compile clean implies ripple completion | if any eval_* called from another fn besides dispatch_keyword_head, that caller also passes list_span |
| 13 | SCORE doc lists actual sweep + ripple counts | SCORE file present | clear breakdown: # dispatch arms updated, # eval_* signatures updated, # ripple-caller updates |

## Independent prediction

**Target runtime:** 60-90 min Mode A
**Upper bound:** 150 min (STOP-3)
**Confidence:** medium-high — mechanical replication of Stone 233.2.c pattern across 133 sites; substrate-as-teacher iteration shape proven via arcs 111/112/113/114/115/117 / 163 slice 3e precedent

**Rationale:**
- 133 dispatch arm updates: ~30-45 min mechanical edits
- Signature ripples via cargo errors: ~20-40 min (depends on call graph fan-in)
- Verification cascade: ~10 min
- SCORE writing: ~10 min

**Risks:**
- Some `eval_*` fns may have many non-dispatch callers (recursive eval, helpers); ripple cost grows. SCORE captures actual count.
- Generic / lifetime parameters on eval fns may add friction at signature update.
- A few arms might call wrapper helpers that internally invoke eval_* — those wrappers may not yet take `list_span`. If the wrapper is shallow, plumb through; if it's deep (multi-layer), surface as honest delta and bound scope to wrapper's immediate caller layer only.
- Closure-extraction walker (arc 170 area) interacts with eval_* signatures; if it breaks, surface and STOP.

## Out-of-scope rows (REJECTED)

- AST-derived provenance (233.2.e)
- Errors-as-EDN (233.3)
- holon-rs touched (STOP-4)
- Renaming `list_span` parameter
- Eval fn body refactors
- Touching the 6 exempt arms (inline non-eval bodies)
- Touching the 243 already-compliant arms (unless ripple-forced by signature changes)
- Repositioning `list_span` in already-compliant arms (e.g., `eval_apply`'s 4th-position usage)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors (errors NOT tracing to substrate-symmetry plumbing)
- **STOP-2:** baseline regress below 827
- **STOP-3:** 150 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 52 baseline
- **STOP-6:** scope creep (body refactor / parameter rename / already-compliant-arm refactor)
- **STOP-7:** substrate-symmetry probe still FAILS
- **STOP-8:** existing arc 233 probes (Stones 233.1 / 233.2.a / 233.2.b / 233.2.c / 232.0a) regress

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` (new file per `feedback_inscription_immutable`).

SCORE structure expected:
- 13 row verdicts (each PASS / Honest Delta / FAIL with details)
- Actual numbers: # dispatch arms updated; # eval_* signatures updated; # non-dispatch caller ripples per fn
- Time breakdown by phase (dispatch arms / signatures / ripples / verification / SCORE)
- Calibration band actual vs predicted
- Any honest deltas surfaced (e.g., wrappers that needed special treatment, position discrepancies left in place per scope)

## What this unblocks

- **Stone 233.2.e** — AST-derived provenance (SymbolBound's `head_span` + Literal's `span` populate on enriched substrate)
- **Any future producer addition** — uniform plumbing convention is now structural; new producers thread `list_span` automatically per template
- **Arc 232 resume** — defprotocol dispatcher operates on cleaner substrate; call-by-name path benefits from uniform call-site coordinates
- **Stone 233.4 INSCRIPTION** — arc 233 INSCRIPTION can read "errors teach uniformly" instead of "errors teach... mostly"

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.d.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.d.md` — sub-DESIGN
- `tests/probe_substrate_symmetry_list_span_threading.rs` — FM 2-bis probe (commit `2ff3d56`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.c.md` — precedent (eval_edn_read)
