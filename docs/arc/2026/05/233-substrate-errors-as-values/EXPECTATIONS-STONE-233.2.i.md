# EXPECTATIONS — Arc 233 Stone 233.2.i — flip eval signature to TrackedValue

Mode A target: **10/10 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **eval signature probe FLIPS 0/3 → 3/3** | `cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` |
| 5 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | Stone 232.0 dynamic-keyword probes still pass | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 8 | Stone 233.2.h TrackedValue mint probe still passes | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 10 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 90-150 min Mode A
**Upper bound:** 180 min (STOP-3) — the BIG cascade per Stone 233.2.g sub-DESIGN
**Confidence:** medium — substrate-as-teacher iteration shape proven (arc 163 slice 3e + Stone 233.2.d precedents); volume is high; helper-signature changes add ripple

**Rationale:**
- Boundary wrap on eval + 3 freeze.rs surfaces: ~15 min
- Cascade through internal `eval(...)?` call sites (~319 in runtime.rs): ~40-60 min mechanical sweep
- Helper signature changes (require_X, expect_X family): ~20-30 min
- External test file cascade: ~15-30 min
- Verification cascade: ~10 min
- SCORE writing: ~15 min

**Risks:**
- Helper signature changes ripple into wat-side test code via `wat::parse_one!` / `eval_in_frozen` patterns
- Borrow checker friction on `.value()` vs `.value_owned()` choices (sonnet picks per call site)
- External integration tests in `tests/*.rs` may need adaptation
- If cascade exceeds 180 min: STOP-3 fires; orchestrator decides sub-slice or extend

## Out-of-scope rows (REJECTED)

- Producer migration (Stone 233.2.j)
- Value::Tracked variant retirement (Stone 233.2.k)
- Display impl on TrackedValue
- Eq/PartialEq/Hash on TrackedValue
- Internal eval_<name> signature changes (they keep returning Value)
- holon-rs touched (STOP-4)
- Parallel `eval_tracked` API (HARD CUT — existing eval IS new boundary)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to cascade
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (Value::Tracked variant, producer logic, internal eval_<name> signatures)
- STOP-7: probe still has failures
- STOP-8: existing arc 233 probes regress
- STOP-9: cascade exceeds time-box — surface partial state for orchestrator

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.i.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- Boundary wrap implementation (eval + 3 freeze.rs surfaces)
- Cascade count per file (eval call sites updated; .value_owned() insertions)
- Helper signature changes per family (require_X / expect_X / etc.)
- External test file adaptations
- Time breakdown by phase
- Calibration band actual vs predicted

## What this unblocks

- **Stone 233.2.j** — producer migration to TrackedValue::new (drops Value::Tracked wrapping at the 5 producer sites)
- **Stone 233.2.k** — Value::Tracked variant retirement (final structural class-elimination)
- **Stone 233.2.e** — AST-derived provenance on the new substrate

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.i.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md` — sub-DESIGN
- `tests/probe_eval_signature_returns_tracked_value.rs` — FM 2-bis probe (commit `df7dcb8`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — substrate-as-teacher cascade precedent
