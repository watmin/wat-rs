# EXPECTATIONS — Arc 233 Stone 233.2.c — sweep 4 producers

Mode A target: **14/14 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed; 1 ignored |
| 3 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 4 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline match) |
| 5 | eval_holon_from_holon wraps in Tracked | `grep -A 100 "fn eval_holon_from_holon" src/runtime.rs \| grep -c "Value::Tracked"` | ≥ 1 (sonnet documents Ok-path count covered in SCORE) |
| 6 | from-holon producer string canonical | `grep -A 100 "fn eval_holon_from_holon" src/runtime.rs \| grep -c '":wat::holon::from-holon"'` | ≥ 2 (existing OP + new producer) |
| 7 | eval_edn_read wraps in Tracked | `grep -A 40 "fn eval_edn_read" src/edn_shim.rs \| grep -c "Value::Tracked"` | ≥ 1 |
| 8 | edn::read producer string canonical | `grep -A 40 "fn eval_edn_read" src/edn_shim.rs \| grep -c '":wat::edn::read"'` | ≥ 1 |
| 9 | eval_kernel_recv wraps in Tracked | `grep -A 80 "fn eval_kernel_recv" src/runtime.rs \| grep -c "Value::Tracked"` | ≥ 1 |
| 10 | recv producer string canonical | `grep -A 80 "fn eval_kernel_recv" src/runtime.rs \| grep -c '":wat::kernel::recv"'` | ≥ 2 (existing op string + new producer) |
| 11 | eval_kernel_try_recv wraps in Tracked | `grep -A 80 "fn eval_kernel_try_recv" src/runtime.rs \| grep -c "Value::Tracked"` | ≥ 1 |
| 12 | **Probe 7 (from-holon) flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_7 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 13 | **Probe 8 (edn::read) flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_8 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 14 | Full probe file 8/8 + holon-rs untouched | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` AND `git -C /home/watmin/work/holon/holon-rs/ status --short` | `8 passed; 0 failed` + empty git output |

## Independent prediction

**Target runtime:** 30-60 min Mode A
**Upper bound:** 90 min (STOP-3)
**Confidence:** high (4 producers; mechanical replication of 233.2.b pattern)

**Rationale:**
- Each producer: ~5-10 min (wrap return + handle Ok-path variants)
- eval_edn_read needs signature change to thread list_span: +5 min
- Verification cascade: ~5 min
- SCORE writing: ~10 min

**Risks:**
- from-holon has multiple Ok-return paths (sonnet wraps each; may miss one)
- edn::read signature change ripples to caller; small but mechanical
- Existing tests asserting EXACT error format may need CONTAINS-updates

## Out-of-scope rows (REJECTED)

- select, IO readers, keyword/to-string
- AST-derived provenance (233.2.d)
- Errors-as-EDN (233.3)
- holon-rs / wat-edn touch
- Aliases / deprecation shims

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** baseline regress
- **STOP-3:** 90 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning
- **STOP-6:** scope creep
- **STOP-7:** Probe 7 or 8 still FAIL
- **STOP-8:** Stone 233.1 probes regress
- **STOP-9:** Stone 233.2.a transparency tests regress

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.c.md` (new file per `feedback_inscription_immutable`).

## What this unblocks

- **233.2.d** — AST-derived provenance (the remaining variants: Literal + SymbolBound)
- **First user-visible payoff for from-holon + edn::read consumers** — runtime-built Values from these substrate sources now teach
- **Pattern is now firmly established** across 5 producers; future producer additions just replicate
