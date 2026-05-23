# SCORE — Arc 233 Stone 233.2.f — apply Tracked-unwrap defect fix

Mode A result: **8/8 PASS**

## Scorecard

| # | Row | Command | Expected | Actual | Result |
|---|---|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors | 0 errors (`Finished` in 16.36s) | PASS |
| 2 | **probe_diagnostic_dynamic_keyword_invocation flips 6/8 → 8/8** | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -5` | `8 passed; 0 failed` | `8 passed; 0 failed` | PASS |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -5` | ≥ 827 passed; 0 failed | `827 passed; 0 failed` | PASS |
| 4 | Stone 233.2.d substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` | `1 passed; 0 failed` | PASS |
| 5 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` | `8 passed; 0 failed` | PASS |
| 6 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` | `8 passed; 0 failed` | PASS |
| 7 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 | 54 | PASS |
| 8 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output | empty output | PASS |

## Fix applied

Two edits in `fn eval_apply` (`src/runtime.rs`):

**Step 5 — fast-path fn-valued head (line 7433):**
- BEFORE: `if let Value::wat__core__fn(ref func) = head_val {`
- AFTER: `if let Value::wat__core__fn(func) = head_val.inner() {`

**Step 6 — keyword-valued head extraction (lines 7438-7448):**
- BEFORE: `match head_val { Value::wat__core__keyword(ref k) => k.clone(), ref other => Err(TypeMismatch { got: ValueSnapshot::of(&other), ... }) }`
- AFTER: `match head_val.inner() { Value::wat__core__keyword(k) => k.clone(), other => Err(TypeMismatch { got: ValueSnapshot::of(other), ... }) }`

Borrow-checker adjustments: `ref` dropped from arm bindings in both sites (match target is now `&Value` via `.inner()` return type). `&other` → `other` in `ValueSnapshot::of(other)` since the arm already binds a `&Value`.

## Honest delta

None. Mode A 8/8 with no deviations from EXPECTATIONS.

## What this unblocks

- **Stone 233.2.e** EXPECTATIONS can assert clean 8/8 across all arc 233 probes (no standing honest delta on probe_diagnostic_dynamic_keyword_invocation).
- **Trust restored** on `:wat::core::apply` for runtime-built keyword dispatch — defprotocol (arc 232) builds on it.
- **Pattern reminder:** any Value pattern-match in src/ should use `Value::inner()` first. Task #491 audits the broader surface.

## Timing

Well within the 10-20 min Mode A target. No STOP triggers fired.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.f.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/EXPECTATIONS-STONE-233.2.f.md` — scorecard source
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — where the defect was surfaced (Row 6 honest delta)
- `tests/probe_diagnostic_dynamic_keyword_invocation.rs` — probe_2 + probe_3 were the load-bearing failures; now 8/8
- `src/runtime.rs:7432-7448` — the two fixed match sites
