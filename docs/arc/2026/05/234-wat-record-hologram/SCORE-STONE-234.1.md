# SCORE — Arc 234 Stone 234.1 — `Value::wat_record` variant + Eq/Hash + dispatch cascade

**Status:** COMPLETE (2026-05-24)
**Result:** 11/11 PASS — Mode A target met

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **New probe 7/7 PASS** (LOAD-BEARING) | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 3 | Stone 234.0 regression guard | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed` |
| 4 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 5 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 6 | Stone 233.3 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 7 | Stone 233.2.e regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 8 | Stone 233.2.l regression guard | `3 passed; 0 failed` | `test result: ok. 3 passed; 0 failed` |
| 9 | Stone 233.2.k regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 10 | Clippy no new warnings | ≤ 54 | `54` (exactly at ceiling; no regression) |
| 11 | holon-rs untouched | empty | empty output (STOP-4 clean) |

### Verbatim verification command outputs

```
# Row 1
warning: `wat` (lib) generated 107 warnings
    Finished `release` profile [optimized] target(s) in 0.04s

# Row 2
test probe_2_eq_same_class_same_holon_form ... ok
test probe_3_eq_different_class ... ok
test probe_1_variant_construction_compiles ... ok
test probe_4_eq_different_field_values ... ok
test probe_5_hash_eq_consistency ... ok
test probe_6_debug_contains_class ... ok
test probe_7_type_name_returns_generic_kind ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 3
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 4
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s

# Row 5
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 6
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 7
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 8
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 9
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 10
54

# Row 11
(empty)
```

---

## Per-Section Line Counts

| Section | File | Lines added |
|---|---|---|
| Variant definition (with doc comment) | `src/runtime.rs` | ~29 |
| PartialEq arm (D2) | `src/runtime.rs` | ~7 |
| Hash arm (D3) | `src/runtime.rs` | ~6 |
| type_name arm (D5) | `src/runtime.rs` | ~2 |
| eval_type arm (D6) + doc update | `src/runtime.rs` | ~5 |
| render_value arm (cascade) | `src/runtime.rs` | ~19 |
| closure_extract cascade arm | `src/closure_extract.rs` | ~7 |
| edn_shim cascade arm | `src/edn_shim.rs` | ~17 |
| **Total** | 3 files | **92 added, 1 removed** |

---

## Cascade Depth

**3 cargo errors surfaced; 3 addressed.**

Sites enumerated by substrate-as-teacher cascade:

1. `src/closure_extract.rs:1465` — `encode_value_with_path` — added `Value::wat_record { .. }` arm returning `ExtractionError::Internal` (no closure-encode form yet; Stone 234.2 ships constructor)
2. `src/edn_shim.rs:1555` — `value_to_edn_with` — added `Value::wat_record` arm rendering as tagged map (`class_fqdn` tag + positional field map)
3. `src/runtime.rs:18044` — `render_value` — added `Value::wat_record` arm rendering as `<class_fqdn{field0, field1, ...}>`

Predicted depth: 5-20 errors. Actual: 3. Low end of prediction — the codebase has fewer exhaustive-match sites than anticipated. All three were mechanical per-pattern applications; zero design decisions required.

---

## Time Breakdown

- Read planning docs + source locations: ~8 min
- Implement 5 mandatory arms (variant, Eq, Hash, type_name, eval_type): ~5 min
- First cargo build — cascade surfaces 3 sites: immediate
- Address 3 cascade sites: ~5 min
- Second cargo build — clean: immediate
- Run all 11 verification commands: ~4 min
- Write SCORE: ~8 min
- **Total: ~30 min** (well inside 60-120 band; STOP-3 at 180 never approached)

---

## Calibration Band

**Predicted:** 60–120 min Mode A
**Actual:** ~30 min
**Delta:** Under-target by ~50%. Cascade depth of 3 (vs predicted 5-20) was the primary driver. The pattern at each cascade site was identical to prior arcs; no iteration cycles required.

Precedent comparison:
- Stone 234.0 (~38 min): single eval fn + dispatch arm + TypeScheme; ZERO iteration
- Stone 234.1 (~30 min): variant + 5 impls + 3 cascade sites; ZERO iteration cycles (2 build passes total: variant addition + one cascade sweep)

The cascade was shallower than predicted but structurally identical in kind. This is honest calibration data for future arc 234 stones.

---

## Rank-Up Evidence

### Substrate-as-teacher cascade (Helwalker/Streetfighter conditions)

The cascade fired exactly as designed. After adding the `Value::wat_record` variant and the 5 mandatory impl arms, cargo enumerated precisely 3 exhaustive-match sites in a single build pass. Each site was mechanically addressable per the existing per-pattern precedent:
- `closure_extract.rs`: mirror of the existing "non-portable capture" arm shape
- `edn_shim.rs`: mirror of the existing `Value::Struct` tagged-map arm shape
- `runtime.rs::render_value`: mirror of the existing `Value::Struct` rendering arm shape

Zero iteration cycles. The substrate taught; the cascade was rideable.

### `#[wat_value]` seal — no escape hatch needed

`Value::wat_record` with three Arc'd field types (`Arc<String>`, `Arc<Vec<Value>>`, `Arc<HolonAST>`) passed the `#[wat_value]` proc-macro seal naturally. The container-variant rule (BRIEF trap-door 1) held: `Arc<Vec<Value>>` is a container wrapping, not a direct `Arc<Self>` single-field wrapping. No `allow_wrapping` escape hatch needed. Verified empirically — first build pass was the proof.

### Stone 234.0 eval_type TODO marker closes cleanly

The `eval_type` TODO marker at `src/runtime.rs:14420` (placed by Stone 234.0) was exactly the right surgery point. The arm dropped in as the third explicit case in the match, before the `other => other.type_name()` fallthrough. One line of code; the doc comment updated in the same pass. This is the "scaffolded insertion point" pattern working exactly as intended.

### Arc 233 tools shortened iteration

- `ValueSnapshot::of` in `require_*` helpers: not needed directly in this stone (cascade sites didn't need diagnostic output), but their presence means error messages for the new variant are already human-readable via Rust's auto-derived `Debug` — zero additional work needed.
- `#[wat_value]` seal structural confidence: knowing the proc-macro would have rejected a wrapping-style variant meant the variant shape could be written confidently without experimental iteration.

---

## Honest Deltas

### D4 — Debug-only (no Display): probe contract change honored

The probe was authored expecting `format!("{}", r)` (Display) in probe 6, but probe 6 in the committed probe file actually uses `format!("{:?}", r)` (Debug). The DESIGN.md D4 section was already honest about this: Value has no Display impl; auto-derived Debug is the correct path. The probe contract (probe 6) checks that `{:?}` output contains the class_fqdn. Verified: `probe_6_debug_contains_class` passes because the auto-derived Debug renders all struct fields including `class_fqdn`.

### Cascade depth lower than predicted

Predicted 5-20 errors; actual 3. The three sites that fired were the only exhaustive-match-on-Value sites outside of `impl PartialEq`, `impl Hash`, and `type_name()`. No check.rs exhaustive match fired (confirmed: check.rs has no exhaustive match on Value). No other files had exhaustive match sites. The substrate is more localized than the prediction assumed.

### Clippy ceiling: exactly 54 (no regression, no headroom)

Clippy count is exactly 54 — at the boundary. No new warnings introduced by the new variant + cascade arms. This is consistent with the `#[allow(non_camel_case_types)]` attribute on the enum (the `wat_record` variant name is snake_case, but the enum has `#[allow(non_camel_case_types)]` applied globally, so no new warning fires).

---

## What This Unblocks

- **Stone 234.2** — `:wat::core::defrecord` macro can now generate `Value::wat_record { ... }` instances; the variant exists with the correct field shape
- **Stone 234.3** — polymorphic verbs (`record->map`, `record?`, etc.) can destructure `Value::wat_record` via `{ class_fqdn, struct_form, holon_form }` pattern
- **Stone 234.4** — hash-destructure patterns match `Value::wat_record` receivers
- **Stone 234.5** — `:wat::holon::*` auto-dispatch on `Value::wat_record` receivers uses `holon_form` directly
- **Revised Stone 232.1** — defprotocol's dispatcher can now extend to `Value::wat_record` via `:wat::core::type`'s updated dispatch table

---

## Cross-References

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.1.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.1.md` — paired EXPECTATIONS (11-row scorecard)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.md` — sub-DESIGN with locked decisions D1-D10
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — predecessor SCORE
- `tests/probe_arc234_stone1_wat_record_variant.rs` — FM 2-bis probe (7 contracts; PASS)
- `src/runtime.rs` — variant + 5 impl arms (PartialEq, Hash, type_name, eval_type, render_value)
- `src/closure_extract.rs` — cascade arm (Internal error for unimplemented closure encode)
- `src/edn_shim.rs` — cascade arm (tagged map EDN rendering)
