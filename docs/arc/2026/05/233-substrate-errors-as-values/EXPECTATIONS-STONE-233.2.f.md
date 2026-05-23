# EXPECTATIONS — Arc 233 Stone 233.2.f — apply Tracked-unwrap defect fix

Mode A target: **8/8 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **probe_diagnostic_dynamic_keyword_invocation flips 6/8 → 8/8** | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 233.2.d substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` |
| 5 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 (post-233.2.d baseline) |
| 8 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 10-20 min Mode A
**Upper bound:** 30 min (STOP-3)
**Confidence:** high — defect localized; fix shape locked; Stone 233.2.a transparency contract is the load-bearing API; 2 match-site edits + minor borrow-checker adjustments

**Rationale:**
- 2 match-site `.inner()` insertions: ~3 min
- `ref` → `&` borrow-checker adjustments: ~3 min
- Compile + iterate: ~5 min
- Verification cascade: ~5 min
- SCORE writing: ~5 min

**Risks:**
- The fast-path `if let` (line 7433) may bind `func` differently when match target is `&Value` — sonnet adjusts per compiler guidance.
- `apply_function(func.clone(), ...)` — when `func` comes from `&Value::wat__core__fn(func)` it's `&Arc<Function>` (or similar); `func.clone()` should still work but if Arc semantics shift, sonnet adjusts.
- If a fix introduces unintended additional probe failures elsewhere — STOP-8 fires; surface as delta.

## Out-of-scope rows (REJECTED)

- Wider audit of Value-match sites without `.inner()` (task #491 follow-up)
- apply body refactor beyond the two match sites
- Renaming `head_val` or related identifiers
- AST-derived provenance (Stone 233.2.e)
- holon-rs touched (STOP-4)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors
- STOP-2: baseline regress below 827
- STOP-3: 30 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (other sites, body refactor)
- STOP-7: probe_diagnostic_dynamic_keyword_invocation still has failures
- STOP-8: existing arc 233 probes regress

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.f.md` (new file per `feedback_inscription_immutable`).

## What this unblocks

- **Stone 233.2.e** EXPECTATIONS can assert clean 8/8 across all arc 233 probes (no standing honest delta on probe_diagnostic_dynamic_keyword_invocation)
- **Trust restored** on `:wat::core::apply` for runtime-built keyword dispatch — defprotocol (arc 232) builds on it
- **Pattern reminder:** any Value pattern-match in src/ should use `Value::inner()` first. Task #491 audits the broader surface.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.f.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — where the defect was surfaced
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — Shape C + transparency contracts
