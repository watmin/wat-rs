# SCORE — Arc 234 Stone 234.2a — forward-correction (TypeScheme heterogeneous struct_form)

**Status:** PARTIAL — STOP-7 fires. The TypeMismatch is eliminated; probe 5 still FAILs for a different, secondary reason exposed by the fix.

**Result:** check.rs custom inference handler `infer_record_of` is correct and complete. STOP-7 fires because the probe's test code uses runtime functions that don't exist (`:wat::core::i64/to-string`, `:wat::core::bool/to-string`). These were masked by the original TypeMismatch, which fired before runtime evaluation was reached.

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **Stone 234.2b probe FLIPS 5/6 → 6/6** (LOAD-BEARING) | `6 passed; 0 failed` | `5 passed; 1 failed` — STOP-7 FIRES (see diagnostic below) |
| 3 | Stone 234.2a regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 4 | Stone 234.1.5 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 5 | Stone 234.1 regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 6 | Stone 234.0 regression guard | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed` |
| 7 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 8 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 9 | `:wat::holon::defrecord` regression guard | `35 passed; 0 failed` | `test result: ok. 35 passed; 0 failed` |
| 10 | Clippy no new warnings | ≤ 54 | `54` (at ceiling; no regression) |
| 11 | holon-rs untouched | empty output | empty output (STOP-4 clean) |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 21.77s

# Row 2 — LOAD-BEARING (STOP-7 FIRES)
cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 | tail -5
thread 'probe_5_multi_field_accessors_in_order' (440840) panicked at tests/probe_arc234_stone2b_defrecord_macro.rs:231:19:
Probe 5 FAILED: eval: UnknownFunction(":wat::core::i64/to-string", Span { file: "<entry>", line: 26, col: 7 })
failures:
    probe_5_multi_field_accessors_in_order
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 3
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -5
test probe_4_multi_field_construction ... ok
test probe_7_equality_via_holon_form ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 4
cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 | tail -5
test probe_5_class_fqdn_extraction_post_rename ... ok
test probe_4_namespace_type_registration ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 5
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5
test probe_6_debug_contains_class ... ok
test probe_7_type_name_returns_generic_kind ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 6
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5
test probe_2_type_on_string ... ok
test probe_6_type_on_hashmap ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 7
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s

# Row 8
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 9
cargo test --release --test probe_arc227_stone2_defrecord 2>&1 | tail -5
test probe_zero_field_instance_uses_empty_bundle ... ok
test probe_predicate_works_for_n0_n1_n2_n3 ... ok
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

# Row 10
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54

# Row 11
git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```

---

## STOP-7 Diagnostic — Secondary Probe Error Exposed by Fix

**STOP-7 fires on probe 5.** The original TypeMismatch is GONE. The new error is:

```
eval: UnknownFunction(":wat::core::i64/to-string", Span { file: "<entry>", line: 26, col: 7 })
```

### What changed

**Before correction:** probe 5 failed at `startup_from_source` (type-check phase) with:
```
TypeMismatch { callee: ":wat::core::vec", param: "#3", expected: ":i64", got: ":String" }
TypeMismatch { callee: ":wat::core::vec", param: "#4", expected: ":i64", got: ":bool" }
```

**After correction:** probe 5 passes type-check (startup_from_source returns Ok). Then `eval_in_frozen` fails at runtime with:
```
UnknownFunction(":wat::core::i64/to-string", ...)
```

The error prefix is `eval:` not `startup:`. Type-check succeeded; runtime evaluation fails.

### Root cause

Probe 5's test body (not the macro under test — the verification code written by the probe author) calls:
```wat
(:wat::core::string::concat
  (:wat::core::i64/to-string a)
  "|"
  b
  "|"
  (:wat::core::bool/to-string c))
```

The substrate has these functions under `::` form (double-colon, path syntax):
- `:wat::core::i64::to-string` (registered at runtime.rs line 5154)
- `:wat::core::bool::to-string` (registered at runtime.rs line 5161)

The probe was authored with `/` form (slash, method syntax):
- `:wat::core::i64/to-string` — DOES NOT EXIST in runtime
- `:wat::core::bool/to-string` — DOES NOT EXIST in runtime

Note: `:wat::core::keyword/to-string` (slash form) DOES exist (runtime.rs line 5164) because keyword's to-string uses method syntax. The probe author applied the keyword pattern to i64 and bool, but those primitives use path syntax.

### Why it was masked

The TypeMismatch error fired BEFORE runtime evaluation reached `(:wat::core::i64/to-string a)`. The type-checker caught the heterogeneous vec at an earlier point in the source form, and `startup_from_source` returned Err without reaching the `i64/to-string` call. Once the TypeMismatch is eliminated, startup succeeds, and runtime eval proceeds to the concatenation code — where it hits the missing function.

### Why I cannot fix this within scope

- Fixing this requires adding `:wat::core::i64/to-string` and `:wat::core::bool/to-string` to the runtime dispatch table in `src/runtime.rs`
- STOP-5: "Rust changes outside `src/check.rs`" is REJECTION criteria
- Adding a TypeScheme for these in check.rs would make type-checking accept them, but runtime would still fail
- Modifying the probe is forbidden ("DO NOT modify the existing probe files")

### Orchestrator options

**(a)** Add `:wat::core::i64/to-string` and `:wat::core::bool/to-string` to `src/runtime.rs` (and `src/check.rs` for type-scheme registration). Small addition — model after `eval_i64_to_string` / `eval_bool_to_string`. Requires a new stone or an expansion of this stone's scope.

**(b)** Correct the probe (modify `tests/probe_arc234_stone2b_defrecord_macro.rs`) to use `::` forms: `:wat::core::i64::to-string` and `:wat::core::bool::to-string`. These exist in both runtime and check.rs. The probe tests the same behavior; only the helper function names change. This is a probe authoring correction analogous to the 234.2a TypeScheme correction.

**(c)** Expand THIS stone's scope to cover runtime.rs (lift STOP-5) — the check.rs fix is complete; adding the two runtime dispatch entries is trivial.

---

## Implementation surface

### What was authored

**`src/check.rs` — two changes:**

1. **`infer_record_of` function** (~80 lines) added after `infer_arithmetic` ends (after line 10986 in pre-edit numbering). Modeled exactly after `infer_arithmetic`'s signature and dispatch pattern.

   The handler:
   - Checks arity = 3 (ArityMismatch if not)
   - Type-checks arg #1 against `:wat::core::keyword` (unify; TypeMismatch on fail)
   - Accepts arg #2 as `WatAST::Vector` (literal) OR `WatAST::List` with `:wat::core::Vector` head — in BOTH cases infers each element independently WITHOUT unification
   - Falls back to `infer` + MalformedForm for non-Vec arg #2 shapes
   - Type-checks arg #3 against `:wat::holon::HolonAST` (unify; TypeMismatch on fail)
   - Returns `Some(TypeExpr::Path(":wat::Record"))` always

2. **Dispatch arm** in `infer_list`'s `match k.as_str()` block, inserted before the `":wat::core::fn"` arm:
   ```rust
   ":wat::Record::of" => {
       return infer_record_of(head_span, args, env, locals, fresh, subst, errors);
   }
   ```

3. **TypeScheme registration REMOVED** for `:wat::Record::of` (lines 16993-17001 in pre-edit). Comment block updated to explain the removal: custom handler in the dispatch takes precedence (returns early BEFORE the generic `env.get` fallback path). Keeping both would be dead code + honest confusion (the TypeScheme had the wrong `Vector<T>` contract).

### T2 investigation outcome (dispatch-hook vs TypeScheme order)

Finding: **custom dispatch arms run FIRST; TypeScheme path never reached for registered arms.**

In `infer_list`, the `match k.as_str() { ... _ => {} }` block at line 4944 runs before the generic `env.get` scheme lookup at line 5848. Every `return` inside a match arm exits `infer_list` immediately. The `_ => {}` fallthrough falls to the scheme lookup. So:

- If `:wat::Record::of` has a match arm → handler runs + returns → TypeScheme registration is bypassed
- If no match arm → `_ => {}` → scheme lookup runs → TypeScheme registration is used

**Resolution: remove the TypeScheme registration.** It was dead code once the dispatch arm was added, and it had the wrong contract (Vector<T> uniform). Removing it is honest.

---

## Dispatch-hook investigation finding (T1)

Found at `src/check.rs` line 4944: `match k.as_str()` inside `fn infer_list`. The arithmetic arm at line 5589:

```rust
":wat::core::+"
| ":wat::core::-"
| ":wat::core::*"
| ":wat::core::/" => {
    return infer_arithmetic(k, head_span, args, env, locals, fresh, subst, errors);
}
```

Mirror insertion for `:wat::Record::of` added at the same dispatch block level, before `":wat::core::fn"`. The generic scheme fallback starts at line 5840 (`// Normal call: look up scheme, instantiate, unify args.`). All custom arms return before reaching line 5840.

---

## Cascade depth

**Compile rounds: 2.**

- **Round 1:** Add `infer_record_of` + dispatch arm + remove TypeScheme. Clean compile. Run probe → probe 5 FAIL with `UnknownFunction(":wat::core::i64/to-string")` — TypeMismatch gone; new error surfaced.
- **Round 2:** Investigate the new error. Traced to runtime.rs line 5154/5161 (functions exist under `::` not `/`). Confirmed STOP-5 applies (runtime.rs is off-limits). STOP-7 confirmed.

No Rust changes beyond what was authored in Round 1.

---

## Time breakdown

- Read mandatory artifacts (BRIEF + DESIGN + EXPECTATIONS + probes + check.rs lines 10885-10980 + 16985-17001 + infer_list dispatch structure): ~15 min
- Author `infer_record_of` + dispatch arm + TypeScheme removal: ~8 min
- Compile Round 1 + initial probe run: ~3 min (clean compile)
- Probe 5 new error investigation (trace `UnknownFunction` to runtime.rs; confirm STOP-5 blocks fix): ~10 min
- All 11 scorecard rows verification: ~10 min
- SCORE writing: ~15 min

**Total: ~61 min.** Outside the 20-35 min target band; under the 40 min STOP-3 hard cap.

Wait — re-reading STOP-3: "40 min elapsed (small change; tight cap)." This is 61 min wall-clock. STOP-3 fires by wall-clock.

However, the check.rs change itself (authoring + compile) took ~26 min. The excess is investigation of the secondary probe error (10 min) + scorecard verification (10 min) + SCORE writing (15 min). The TYPE-CHECK FIX itself was within the 40 min band. The STOP-7 investigation is mandatory (per discipline: diagnose before surfacing).

Noting: STOP-3 fires by wall-clock if the session exceeded 40 min. Surfacing honestly.

---

## Calibration delta

- Predicted: 20-35 min Mode A
- Actual: ~61 min total; ~26 min for the check.rs change itself
- Variance drivers:
  1. The secondary probe error (`:wat::core::i64/to-string` missing at runtime) was invisible until my fix removed the earlier TypeMismatch. Discovery + diagnosis took ~10 min.
  2. The BRIEF/DESIGN assumed the TypeMismatch was the ONLY failure mode in probe 5. This was incorrect — the probe had a second bug that was always masked.
  3. SCORE writing takes longer than predicted.

---

## Trap-door audit (T1-T8)

### T1 — Dispatch-hook location
**RESOLVED.** Found at `infer_list` line 4944. Match arm added before `":wat::core::fn"` arm, well before the `_ => {}` fallthrough and the generic scheme path at line 5840. Pattern mirrors arithmetic arms exactly.

### T2 — Coexistence: custom handler + TypeScheme
**RESOLVED.** Custom handler takes precedence (returns early before scheme fallback). TypeScheme registration REMOVED. Dead code eliminated; honest comment added explaining the removal.

### T3 — Vec-shape recognition for arg #2
**RESOLVED.** Handler branches on `WatAST::Vector` (literal `[...]`) and `WatAST::List` with `:wat::core::Vector` head. In both cases, elements inferred independently with no cross-element unification. Third arm accepts general expressions (no shape validation — runtime already accepts any Vec).

### T4 — Empty struct_form `[]`
**CLEAN.** Probe 6 (zero-field) passes. `WatAST::Vector([], _)` → zero iterations through the element loop → returns `:wat::Record`. Handler handles this correctly.

### T5 — Single-field uniform struct_form
**CLEAN.** Probes 1, 2, 3, 4, 6 all pass (5/6 in the probe). Single-field uniform cases work. Handler infers the one element independently; result is `:wat::Record`.

### T6 — Stone 234.2a regression guard
**CLEAN.** 6/6 PASS confirmed. Uniform-vec cases (234.2a used only uniform-type struct_forms) continue to pass; the handler doesn't reject them.

### T7 — `:wat::Record/field-at` polymorphic-T inference
**CLEAN.** TypeScheme for `/field-at` unchanged. Accessor probes 2, 5 (accessor calls) pass in 234.2a probe (6/6). The correction doesn't touch field-at's registration.

### T8 — Macros that use `:wat::Record::of`
**CLEAN for macro type-check.** The 234.2b macro (`wat/Record.wat`) generates `:wat::Record::of` calls; they now pass type-checking (probe 5's constructor type-checks correctly). The RUNTIME failure in probe 5 is in the TEST CODE around the macro invocation, not in the macro itself. The macro's generated code is correct.

### T-NEW — Secondary probe error (not in T1-T8; novel)
**STOP-7 FIRES.** Probe 5's test body (the verification code, not the macro under test) calls `:wat::core::i64/to-string` and `:wat::core::bool/to-string`. These functions exist in the substrate ONLY under `::` form. The probe was authored with `/` form (matching `keyword/to-string` pattern). The TypeMismatch masked this error until the check.rs fix removed it. Now the runtime fails at the test code.

---

## Rank-up evidence — `infer_arithmetic` precedent

The `infer_arithmetic` pattern (arc 148 slice 4) SUBSTANTIALLY shortened authoring:

1. **Signature shape** was verbatim copied (`op: &str` → `head_span`, same `env`/`locals`/`fresh`/`subst`/`errors` parameters). Zero guessing about the function signature.

2. **Dispatch-hook location** was found in one grep (`grep -n "infer_arithmetic" check.rs`) → line 5593 → scroll to see the `match k.as_str()` block structure → insertion point obvious.

3. **Return-early pattern** was confirmed by reading the arithmetic arm: `return infer_arithmetic(...)`. Same `return` in my arm.

4. **T2 resolution** was fast: seeing that every arm uses `return` and the `_ => {}` fallthrough leads to the scheme lookup made the "remove the TypeScheme" decision obvious within 2 min of investigation.

Without the `infer_arithmetic` precedent: ~20 min to understand the dispatcher structure from scratch. With it: ~5 min. The precedent cut investigation by ~75% on the dispatcher-orientation subproblem.

The pattern didn't prevent STOP-7 (the secondary probe error is a different layer), but the check.rs authoring itself was fast and accurate.

---

## Working tree state

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/check.rs
 M src/stdlib.rs
?? docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2a-CORRECTION.md
?? docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a-CORRECTION.md
?? docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2a-CORRECTION.md
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md
?? wat/Record.wat
```

`src/check.rs` is modified (this stone). `src/stdlib.rs` and `wat/Record.wat` are 234.2b's earlier work. Both SCORE docs are new.

---

## Honest assessment

The check.rs correction is complete, correct, and clean:
- TypeMismatch eliminated from probe 5's error output
- `infer_record_of` handler correctly accepts heterogeneous Vec elements
- TypeScheme registration removed (dead code; wrong contract)
- All 10 non-STOP-7 scorecard rows PASS
- Clippy stays at 54 (no regression)
- 6/6 PASS for 234.2a, 234.1.5, 234.1, 234.0 regression guards

STOP-7 fires for a reason beyond check.rs scope: probe 5's test code uses non-existent runtime functions. The DESIGN assumed the TypeMismatch was probe 5's only failure; it was not. The secondary error was always present but invisible.

The orchestrator decides the path forward: add the two runtime functions, or correct the probe's function names.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2a-CORRECTION.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a-CORRECTION.md` — sub-DESIGN with 8 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2a-CORRECTION.md` — paired EXPECTATIONS
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — predecessor SCORE (immutable; unchanged)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — sibling SCORE (234.2b sonnet's earlier work)
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — load-bearing test (probe 5 STILL FAILS; secondary error)
- `tests/probe_arc234_stone2a_record_primitives.rs` — regression guard (6/6 PASS; unchanged)
- `src/check.rs` — the ONLY modified Rust file
- `src/runtime.rs` line 5154 — `:wat::core::i64::to-string` (exists; `::` form)
- `src/runtime.rs` line 5161 — `:wat::core::bool::to-string` (exists; `::` form)
- `src/runtime.rs` line 5164 — `:wat::core::keyword/to-string` (exists; `/` form — the pattern the probe author generalized from)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_inscription_immutable.md` — SCORE-STONE-234.2a.md stays unchanged
- `feedback_no_broken_commits.md` — do not commit broken state; STOP-7 prevents atomic commit
