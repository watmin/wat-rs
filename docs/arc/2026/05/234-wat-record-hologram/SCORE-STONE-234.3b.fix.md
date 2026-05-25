# SCORE — Stone 234.3b.fix — `RuntimeError::UnknownField` variant

**Status:** 11/11 PASS.
**Date:** 2026-05-24.

---

## Scorecard

| # | Row | Command | Verbatim tail | Result |
|---|-----|---------|---------------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | `warning: \`wat\` (lib) generated 107 warnings`<br>`Finished \`release\` profile [optimized] target(s) in 0.08s` | PASS (0 errors) |
| 2 | **234.3b probe stays GREEN** | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 3 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 4 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 5 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 6 | 234.5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 7 | 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` | PASS |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.18s` | PASS (827 ≥ 827) |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | PASS |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` | PASS (54 ≤ 54) |

---

## Implementation surface

### Files modified

1. **`src/runtime.rs`**
   - Added `UnknownField { record_class: String, field: String, available: Vec<String>, span: Span }` variant to `RuntimeError` enum (after `EdnCoerceMismatch`)
   - Added `Display` impl arm for `UnknownField` (message: `"unknown field '<field>' on record <class>; available: [<list>]"`)
   - Migrated `eval_record_assoc` missing-field `Err(RuntimeError::MalformedForm {...})` → `Err(RuntimeError::UnknownField {...})` — HARD CUT, no shim

2. **`src/runtime_error_edn.rs`**
   - Added EDN serializer arm in `runtime_error_to_edn` for `UnknownField` (4-field map: `record-class`, `field`, `available` as EDN vector of strings, `span`)
   - Added variant-name arm in `variant_name`: `RuntimeError::UnknownField { .. } => "UnknownField"`

### Cascade depth

**Zero cascade.** The compiler surfaced no other exhaustive match sites beyond the two already in scope (`runtime.rs` Display impl, `runtime_error_edn.rs` serializer + name lookup). The `freeze.rs`, `io.rs`, `rust_deps/marshal.rs` references are all non-exhaustive (individual arm matches or `matches!()` predicates) — no new arms needed there.

Total exhaustive match sites patched: 3 (Display impl + EDN serializer + variant-name lookup).

### Line delta

- `src/runtime.rs`: +30 lines (variant doc comment + struct + Display arm + use-site migration replacing 8 lines with 6 lines)
- `src/runtime_error_edn.rs`: +13 lines (serializer arm + variant-name arm)
- Total: +43 lines across 2 files

---

## Deltas from SCORE-STONE-234.3b.md

Stone 234.3b SCORE noted: *"UnknownField error uses MalformedForm with reason string... Future cleanup arc could mint RuntimeError::UnknownField as proper variant."* That deferral is now closed.

The `available: Vec<String>` collection was already present in `eval_record_assoc` from Stone 234.3b (the walk was already collecting field names for the error message string). Migration was mechanical: extract the already-collected vec, pass it directly to the new variant. Zero structural change to the walk logic.

Probe 3 (`probe_3_unknown_field_errors`) continues to pass — its lenient assert (`msg.contains("unknown") || msg.contains("nonexistent")`) accepts both the old Display rendering and the new one (`"unknown field 'nonexistent' on record ..."` satisfies `contains("unknown")`).

---

## Time breakdown

- Read docs (BRIEF + DESIGN + EXPECTATIONS): ~3 min
- Read source files (enum, Display impl, eval_record_assoc, runtime_error_edn.rs): ~4 min
- Edits (5 changes across 2 files): ~4 min
- Build + 11-row scorecard: ~8 min
- SCORE write: ~3 min

**Total: ~22 min** (within 15-30 min Mode A target; well under 45 min STOP-3)
