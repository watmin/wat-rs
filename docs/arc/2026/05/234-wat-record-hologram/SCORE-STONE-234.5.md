# SCORE — Arc 234 Stone 234.5 — `:wat::holon::*` auto-dispatch on `Value::wat__Record`

**Status:** COMPLETE — 11/11 PASS. The hologram property is externally observable.

**Result:** 5 VSA verbs extended. Records flow through `:wat::holon::*` natively. The user-facing `(:wat::core::record->holon r)` conversion call is no longer required. `(:wat::holon::cosine r1 r2)` works end-to-end.

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **New probe FLIPS 6/6 FAIL → 6/6 PASS** (LOAD-BEARING) | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 3 | Stone 234.2b regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 4 | Stone 234.2a regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` |
| 5 | Stone 234.1.5 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 6 | Stone 234.1 regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 7 | Stone 234.0 regression guard | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 8 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s` |
| 9 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 10 | Clippy no new warnings | ≤ 54 | `54` (at ceiling; no regression) |
| 11 | holon-rs untouched | empty output | `(empty)` (STOP-4 clean) |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 19.00s

# Row 2 — LOAD-BEARING (6/6 PASS)
cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 | tail -5
test probe_4_bundle_accepts_records_as_children ... ok
test probe_6_mixed_records_and_holon_asts ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 3
cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 | tail -5
test probe_3_predicate_true_on_matching_class ... ok
test probe_4_predicate_false_on_non_matching_class ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 4
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -5
test probe_5_field_at_positional_access ... ok
test probe_7_equality_via_holon_form ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

# Row 5
cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 | tail -5
test probe_5_class_fqdn_extraction_post_rename ... ok
test probe_4_namespace_type_registration ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 6
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5
test probe_4_eq_different_field_values ... ok
test probe_6_debug_contains_class ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 7
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5
test probe_8_type_on_struct_instance ... ok
test probe_7_type_on_defrecord_instance ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 8
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s

# Row 9
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 10
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54

# Row 11
git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```

---

## Implementation Pattern Chosen

### Runtime: Centralized helper + verb threading (D1/D3)

**Pattern chosen: centralized `coerce_to_holon_ast` helper (D1).** Added at `src/runtime.rs` immediately after `require_holon` (line ~16761 in pre-edit). The helper (~16 lines) normalizes two representations:
- `Value::holon__HolonAST(h)` → `(*h).clone()` (existing case)
- `Value::wat__Record { holon_form, .. }` → `holon_form.as_ref().clone()` (NEW)
- Other → `TypeMismatch` with "HolonAST or wat::Record" diagnostic

**`to_holon_inner` arm (D2):** Added `Value::wat__Record { holon_form, .. }` arm before the `other` catch-all (~8 lines). Returns `holon_form.as_ref().clone()` directly (no Atom wrapping — the bridge exposes the pre-built holon form as-is per D2). Also updated the `other` arm's error message to mention `wat::Record`.

**Verb threadings (D3):**

| Verb | Threading approach |
|---|---|
| `eval_algebra_bind` | Replaced `require_holon` calls with `coerce_to_holon_ast` for both args. Removed `(*a).clone()` + `(*b).clone()` since helper returns `HolonAST` directly (not `Arc<HolonAST>`). 2 lines changed. |
| `eval_algebra_bundle` | Replaced `require_holon` in `list.iter().map(...)` with `coerce_to_holon_ast`. `Span::unknown()` per arc 138 discipline (we have Value, not WatAST). 2 lines changed. |
| `pair_values_to_vectors` (cosine) | Normalized `wat__Record` → `Value::holon__HolonAST(holon_form)` before the `match (a, b)` dispatch. Added 6 normalization lines; the existing HolonAST arms handle the rest. |
| `eval_algebra_cosine` | Added `Similarity::cosine(...).clamp(-1.0, 1.0)`. Cosine is mathematically bounded to `[-1, 1]`; floating-point imprecision produced `1.0000000000000002` for identical vectors. Clamp is the honest substrate-level fix (probe 2 asserts strict `(-1.0..=1.0)` range). |
| `eval_extract_classifier` | Restructured to `match arg_val` with three arms: `wat__Record { class_fqdn, .. }` → `Value::String(class_fqdn)` (not Option); `holon__HolonAST` → existing `Value::Option(Option<String>)`; `other` → TypeMismatch. |

**Runtime helper fn line count:** ~16 lines for `coerce_to_holon_ast`.
**Per-verb threading line counts:** Bind ~2, Bundle ~2, cosine (pair_values) ~6, cosine (clamp) ~1, extract-classifier ~10. Total ~21 lines.
**`to_holon_inner` arm:** ~8 lines.

### Check.rs: Custom handlers + predicate extension (D4)

**Pattern chosen: custom dispatch handlers per verb (precedent: Stone 234.2a-CORRECTION's `infer_record_of`).** Two new custom handlers added; `is_atomizable` and `is_holon_or_vector` predicates extended.

**`is_atomizable` extension (~5 lines):** Added `:wat::Record` to the `matches!` list with a comment explaining the hologram property. Fixes probe 1 (`to-holon` accepts records).

**`is_holon_or_vector` extension (~2 lines):** Added `|| p == ":wat::Record"` to the predicate. Fixes probe 2 (`cosine` accepts records via `infer_polymorphic_holon_pair_to_f64`). Also updated the error message in `infer_polymorphic_holon_pair_to_f64` to mention `:wat::Record`.

**`is_holon_or_record` predicate (new, ~6 lines):** Companion to `is_holon_or_vector`, used by `infer_holon_bind` and `infer_holon_bundle` element checks.

**`infer_holon_bind` custom handler (~50 lines):** Accepts 2 args; validates each against `is_holon_or_record`. Returns `holon_ty()`. Dispatch arm added before the cosine arm in `match k.as_str()`. TypeScheme registration at line 14311 retained as documentation (dead code at these call sites per Stone 234.2a-CORRECTION precedent).

**`infer_holon_bundle` custom handler (~70 lines):** Accepts 1 arg. For `WatAST::Vector` literals: validates each element independently against `is_holon_or_record` (mirrors `infer_record_of`'s heterogeneous-vec pattern). For non-literal args (e.g., `(:wat::core::Vector ...)` list forms): infers the arg type then validates it's `Vector<HolonAST>` or `Vector<Record>` — preserving the old TypeScheme's rejection of `Vector<i64>` and similar. Returns `Result<HolonAST, CapacityExceeded>`. Dispatch arm added before the cosine arm.

**`extract-classifier` handler update (~15 lines):** Extended the existing custom handler (Stone 232.0a) to detect `:wat::Record` arg type and return `:wat::core::String` (not `Option<String>`). Records always have a classifier; the `Option` wrapping is not needed. HolonAST args retain the existing `Option<String>` path.

**Check.rs total new/modified lines:** ~155 lines (two new handlers + predicate extensions + handler update).

---

## Cascade Depth

**Compile rounds: 2.**

**Round 1:** Full implementation — runtime helper + 5 verb threadings + check.rs extensions. 0 compile errors. Run probe → 5/6 PASS. Probe 2 fails: `cosine should be in [-1, 1]; got 1.0000000000000002` (floating-point imprecision on identical-vector cosine). **Round 1 iteration: cosine clamp** — added `.clamp(-1.0, 1.0)` to `eval_algebra_cosine`.

**Round 1.5 (after clamp):** Run probe again → 6/6 PASS. Run lib tests → 826 passed; 1 FAILED. `bundle_of_list_of_ints_rejected` regresses — my `infer_holon_bundle`'s `other` arm accepted without type-checking (which the old TypeScheme DID enforce for non-literal Vec forms). **Round 1.5 iteration: bundle `other` arm fix** — added type validation for non-literal args.

**Round 2:** Clean compile. 6/6 probe PASS. 827 lib tests PASS. 11/11 scorecard PASS.

**Net compile rounds: 2** (1 clean + 1 iteration for float precision + 1 iteration for bundle regression).

---

## Trap-Door Audit (T1-T8)

### T1 — `Value::wat__Record` field access pattern
**RESOLVED.** Used `holon_form.as_ref().clone()` throughout (in `coerce_to_holon_ast`, `to_holon_inner`, `pair_values_to_vectors` normalization). Pattern proven at Stone 234.2a `eval_record_field_at`. Arc is shared; `as_ref().clone()` is correct.

### T2 — `to_holon_inner` is the polymorphic UP body
**RESOLVED.** Added `Value::wat__Record` arm before the `other` catch-all. Returns `holon_form.as_ref().clone()` directly (not wrapped in Atom). The `to-holon` verb IS the bridge; it exposes the pre-built holon form as-is per D2.

### T3 — `pair_values_to_vectors` for cosine
**RESOLVED.** Threaded via pre-dispatch normalization: `Value::wat__Record { holon_form, .. }` → `Value::holon__HolonAST(holon_form)` before the `match (a, b)` block. The existing `(HolonAST, HolonAST)` arm then handles both operands. Clean — no modification to the existing arms.

### T4 — `eval_algebra_bind` and `eval_algebra_bundle`
**RESOLVED.** Bind: replaced `require_holon` with `coerce_to_holon_ast` for both args. Bundle: replaced `require_holon` in the children iterator. Both thread cleanly.

### T5 — `eval_extract_classifier` reads from HolonAST shape
**RESOLVED.** When called on a `wat::Record`, returns `Value::String(class_fqdn)` directly (not wrapped in Option). The `class_fqdn` is always present — it's mandatory at record construction. This matches what probe 5 expects: the return type declared is `:wat::core::String`.

### T6 — check.rs TypeScheme registration sites
**RESOLVED.** Located via grep. Strategy:
- `to-holon`: extended `is_atomizable` (used by existing custom handler)
- `cosine`: extended `is_holon_or_vector` (used by existing `infer_polymorphic_holon_pair_to_f64`)
- `Bind`: new custom handler `infer_holon_bind` + dispatch arm (supersedes TypeScheme)
- `Bundle`: new custom handler `infer_holon_bundle` + dispatch arm (supersedes TypeScheme)
- `extract-classifier`: extended existing custom handler (Stone 232.0a) for `:wat::Record` arg path

### T7 — Custom-handler precedent
**RESOLVED.** Stone 234.2a-CORRECTION's `infer_record_of` shortened authoring significantly:
- Signature shape copied verbatim
- Dispatch arm location known (before cosine arm in `match k.as_str()`)
- Return-early pattern confirmed
- TypeScheme "dead code" decision applied to Bind and Bundle registrations

Without the precedent: ~20 min to understand dispatcher structure. With it: ~5 min. Estimated 70% reduction in orientation time.

### T8 — Lib baseline + regression guards
**RESOLVED.** Lib baseline 827 maintained. All 5 arc 234 regression guards PASS. Stone 232.0a and 234.0 regression guards PASS. The `bundle_of_list_of_ints_rejected` lib test revealed the Bundle `other` arm gap; fixed in Round 1.5.

**T8-NEW — Float precision issue (probe 2):** `Similarity::cosine` for identical vectors returned `1.0000000000000002` (1 ULP above 1.0). Probe 2 asserts strict `(-1.0..=1.0)` range. Fixed via `.clamp(-1.0, 1.0)` in `eval_algebra_cosine`. Mathematically honest — cosine similarity IS defined on [-1, 1]; floating-point imprecision is the implementation artifact.

**T8-NEW2 — Bundle `other` arm regression:** `bundle_of_list_of_ints_rejected` lib test caught that non-literal Bundle args must still be type-checked. Fixed by validating inferred type is `Vector<HolonAST>` or `Vector<Record>` in the `other` arm.

---

## Time Breakdown

- Read mandatory artifacts (BRIEF + DESIGN + EXPECTATIONS + probe + SCORE-234.2a-CORRECTION): ~15 min
- Read runtime.rs key functions (to_holon_inner, eval_algebra_bind, bundle, cosine, extract-classifier, pair_values_to_vectors, require_holon): ~10 min
- Read check.rs key sites (is_atomizable, is_holon_or_vector, TypeScheme registrations, extract-classifier handler, Bind/Bundle registrations): ~10 min
- Author runtime changes (helper + 5 verb threadings + to_holon_inner arm): ~10 min
- Author check.rs changes (is_atomizable, is_holon_or_vector, infer_holon_bind, infer_holon_bundle, extract-classifier update): ~15 min
- Compile Round 1 + probe run (5/6 PASS): ~20 min
- Float precision fix + bundle `other` arm fix: ~5 min
- Compile Round 2 + full scorecard verification: ~20 min
- SCORE writing: ~15 min

**Total: ~120 min.** At the STOP-3 hard cap edge. The two secondary issues (float clamp + bundle regression) each added ~5 min; without them the stone would have been ~110 min.

---

## Calibration Delta

- Predicted: 70-90 min Mode A
- Actual: ~120 min total
- Variance drivers:
  1. **Float precision issue** (probe 2): `1.0000000000000002` outside `(-1.0..=1.0)`. Diagnosis ~3 min; fix ~2 min. Not predicted.
  2. **Bundle `other` arm regression** (lib test `bundle_of_list_of_ints_rejected`): discovered in Round 1.5. The non-literal Vec form path was not validated; old TypeScheme enforced it. Diagnosis ~3 min; fix ~3 min. Not predicted but similar to Stone 234.2a-CORRECTION's secondary probe error pattern.
  3. **Artifact reading**: 5 artifacts + 7 code sections = more reading than predicted.
  4. **SCORE writing**: consistent with prior SCOREs; ~15 min.

The implementation itself (authoring + first compile) was within band. The secondary issues added ~2 compile/test rounds.

---

## Rank-Up Evidence — Stone 234.2a-CORRECTION Precedent

Stone 234.2a-CORRECTION's `infer_record_of` pattern SUBSTANTIALLY shortened the check.rs authoring:

1. **Dispatch arm location**: known in advance (before cosine arm in `match k.as_str()`). Found in < 1 min via grep vs ~10 min orientation from scratch.

2. **Handler signature shape**: copied verbatim (`head_span, args, env, locals, fresh, subst, errors` pattern). Zero guessing.

3. **TypeScheme dead-code decision**: confirmed from SCORE — when a dispatch arm is added, the TypeScheme registration is bypassed. Applied immediately to Bind and Bundle; no investigation needed.

4. **Heterogeneous-vec element inference**: `infer_record_of`'s pattern (infer each element independently, no cross-element unification) applied directly to `infer_holon_bundle`'s Vector-literal arm.

5. **`is_holon_or_vector` extension pattern**: demonstrated that predicate-extension is the right approach for cosine (vs. a new custom handler). Extended in 2 lines.

Without the precedent: estimated ~25 min extra (dispatcher structure orientation + signature guessing). With it: ~5 min total orientation for check.rs. ~80% reduction on check.rs authoring.

---

## Working Tree State

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/check.rs
 M src/runtime.rs
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md
```

Three files: two modified (as expected), one new SCORE doc.

---

## Honest Assessment

Stone 234.5 ships **11/11 PASS**. The hologram property is now externally observable:

- `(:wat::holon::to-holon r)` returns the record's pre-built holon_form directly
- `(:wat::holon::cosine r1 r2)` computes VSA similarity end-to-end
- `(:wat::holon::Bind c r)` composes classifier + record's holon_form
- `(:wat::holon::Bundle [r1 r2 r3])` superimposes three records
- `(:wat::holon::extract-classifier r)` returns the class_fqdn String directly

Two secondary issues surfaced and were resolved within scope:
1. Float precision clamp in `eval_algebra_cosine` — mathematically honest; no semantic change
2. Bundle `other` arm regression — `bundle_of_list_of_ints_rejected` lib test enforced that the custom handler preserves the old TypeScheme's non-literal arg validation

The centralized `coerce_to_holon_ast` helper (D1) threaded cleanly across the 4 algebra verbs. The `to_holon_inner` arm (D2) added the bridge in ~8 lines. The custom-handler approach (D4) composed cleanly with the existing dispatch structure, informed by Stone 234.2a-CORRECTION's precedent.

No STOP triggers fired.

---

## Cross-References

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.5.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.5.md` — sub-DESIGN with 9 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.5.md` — paired EXPECTATIONS
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent (load-bearing reference)
- `tests/probe_arc234_stone5_holon_auto_dispatch.rs` — FM 2-bis probe (6/6 PASS post-stone)
- `src/runtime.rs::coerce_to_holon_ast` — centralized D1 helper (after `require_holon`)
- `src/runtime.rs::to_holon_inner` — D2 arm (before `other` catch-all)
- `src/runtime.rs::eval_algebra_bind` — D3 threading
- `src/runtime.rs::eval_algebra_bundle` — D3 threading
- `src/runtime.rs::pair_values_to_vectors` — D3 cosine normalization
- `src/runtime.rs::eval_algebra_cosine` — float precision clamp
- `src/runtime.rs::eval_extract_classifier` — D3 threading (String direct return for records)
- `src/check.rs::is_atomizable` — `:wat::Record` added
- `src/check.rs::is_holon_or_vector` — `:wat::Record` added
- `src/check.rs::is_holon_or_record` — new predicate
- `src/check.rs::infer_holon_bind` — new custom handler
- `src/check.rs::infer_holon_bundle` — new custom handler
- `src/check.rs::extract-classifier handler` — extended for `:wat::Record` arg path
