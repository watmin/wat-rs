# SCORE — Stone 241.13: `:wat::core::define-dispatch` HARD CUT + DispatchRegistry deletion (Enemy 2 of 3)

**Mode:** A (substrate + cascade; vigilia NOT required — no new namespaced home)
**Runtime:** two sessions (context boundary mid-flight); resumed directly from compacted summary
**Cascade size:** 445-line `src/dispatch.rs` DELETED; 6 substrate files cleaned; 6 test files handled (1 deleted, 2 comment-updated, 1 assertion-replaced, 2 fixture-migrated)
**Lib tests:** 890 / 0
**Clippy:** 905 warnings (within ≤920 gate)
**Vigilia:** NOT CAST (legacy flat substrate; no new namespaced home)
**Auto-fixer:** NOT minted (cascade deletion was mechanical; compiler-guided)

---

## Phase A Scorecard (12 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (define-dispatch HARD-CUT-rejected) | PASS | `contract_01_define_dispatch_hard_cut_rejected` |
| 2 | Probe C02 PASS (rejection remedy names defclause) | PASS | `contract_02_rejection_remedy_names_defclause` |
| 3 | Probe whole-suite 2/2 | PASS | `probe_arc241_stone13_define_dispatch_hard_cut` |
| 4 | Stone 241.12 probe preserved 5/5 | PASS | `probe_arc241_stone12_defalias` |
| 5 | Stone 241.11 probe preserved 5/5 | PASS | `probe_arc241_stone11_define_hard_cut` |
| 6 | Stone 241.10 probe preserved 8/8 | PASS | `probe_arc241_stone10_remedy` |
| 7 | probe_arc237_7a 6/6 preserved | PASS | intrinsic behavior intact |
| 8 | probe_arc237_7b 7/7 preserved | PASS | intrinsic typing intact |
| 9 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 10 | Workspace test-build clean | PASS | `cargo build --tests --workspace` exit 0 |
| 11 | Clippy ≤ 920 | PASS | 905 warnings |
| 12 | Pre-INSCRIPTION grep gate clean | PASS | 0 active callers of `:wat::core::define-dispatch` |

---

## Structural Verification (8 rows)

| Verification | Result |
|---|---|
| `src/dispatch.rs` DELETED entirely | `git rm src/dispatch.rs` — 445 lines removed |
| `pub mod dispatch;` removed from `src/lib.rs` | confirmed |
| `DispatchRegistry` plumbing deleted from `src/check.rs` | field, method, guard, `infer_dispatch_call` + 3 helpers removed |
| `DispatchRegistry` plumbing deleted from `src/freeze.rs` | import, field, accessor, steps 4a + 6b, `Dispatch(DispatchError)` variant removed |
| `DispatchRegistry` plumbing deleted from `src/runtime.rs` | field, `set/get` methods, guards in `dispatch_keyword_head` + `eval_apply`, `eval_dispatch_call`, `eval_dispatch_call_with_vals`, `dispatch_to_define_ast`, `dispatch_to_signature_ast`, `Binding::Dispatch`, arms in `body-of`/`lookup-define`/`signature-of-defn`, test harness block removed |
| `Binding::Dispatch` variant deleted | `enum Binding<'a>` now has 5 variants (UserFunction/Macro/Primitive/SpecialForm/Type) |
| `:wat::core::define-dispatch` removed from `is_mutation_form` + `is_declaration_form` | confirmed in `freeze.rs` |
| 7th RETIREMENT_TABLE entry verbatim | `src/remedy/retirement.rs:58` — `(":wat::core::define-dispatch", ":wat::core::defclause")` |

---

## HARD-CUT arm (check.rs)

```rust
// Stone 241.13 — HARD CUT: :wat::core::define-dispatch is retired.
// :wat::core::defclause (Stone 237.2) is the surviving dispatch entity kind.
// No privileged paths per `feedback_hard_cut_admits_no_bypasses`.
":wat::core::define-dispatch" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.13); use ':wat::core::defclause' instead", k),
        span: head_span.clone(),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
    }]);
}
```

---

## RETIREMENT_TABLE post-stone (7 entries)

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    (":wat::core::enum",              ":wat::core::defenum"),
    (":wat::core::define",            ":wat::core::defn"),
    (":wat::core::Char",              ":wat::core::char"),
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
    // Stone 241.13 — defclause replaces define-dispatch.
    (":wat::core::define-dispatch",   ":wat::core::defclause"),
];
```

---

## Cascade Audit

### S1-S3 — RETIREMENT_TABLE + dispatch.rs deletion + lib.rs

- `src/remedy/retirement.rs`: 7th entry added; arc history table updated with Stone 241.13 row
- `src/dispatch.rs`: DELETED via `git rm` (445 lines)
- `src/lib.rs`: `pub mod dispatch;` removed

### S4 — DispatchRegistry cascade deletion (substrate-as-teacher)

**`src/freeze.rs`:**
- Import block removed
- `pub dispatchs: DispatchRegistry` field removed from `FrozenWorld`
- `FrozenWorld::freeze()` signature simplified (removed `dispatchs: DispatchRegistry` parameter)
- Step 4a removed (stdlib dispatch registration before macro expansion)
- Step 6b removed (user dispatch declarations)
- `dispatchs` accessor method removed
- `Dispatch(DispatchError)` variant removed from `StartupError`
- `Display`, `diagnostics()`, `From<DispatchError>` impls for `Dispatch` variant removed
- `:wat::core::define-dispatch` removed from `is_mutation_form()` and `is_declaration_form()`

**`src/check.rs`:**
- `dispatch_registry: Option<Arc<crate::dispatch::DispatchRegistry>>` field removed from `CheckEnv`
- `from_symbols()`, `with_types()` dispatch_registry lines removed
- `pub fn dispatch_registry()` method removed
- `if let Some(reg) = env.dispatch_registry()` guard in `infer_list` removed
- `infer_dispatch_call` function deleted (~195 lines)
- `collect_pattern_type_vars`, `collect_pattern_type_vars_inner`, `collect_single_char_type_vars` helpers deleted
- Test harness dispatch setup block removed (dispatch registration + macro_sym.set_dispatch_registry)

**`src/runtime.rs`:**
- `dispatch_registry: Option<Arc<crate::dispatch::DispatchRegistry>>` field removed from `SymbolTable`
- Debug impl field line removed
- `set_dispatch_registry()` + `dispatch_registry()` methods removed
- `if let Some(reg) = &sym.dispatch_registry` guard in `dispatch_keyword_head` removed
- `eval_dispatch_call` function deleted (~62 lines)
- `if let Some(reg) = &sym.dispatch_registry` guard in `eval_apply` removed (comment renumbered: (c) → (c)/(d))
- `eval_dispatch_call_with_vals` function deleted (~45 lines)
- `dispatch_to_define_ast` function deleted (~27 lines)
- `type_expr_to_keyword` function deleted (4 lines)
- `dispatch_to_signature_ast` function deleted (~66 lines)
- `Binding::Dispatch` variant removed from `pub enum Binding<'a>`
- `// 2a. Dispatchs` guard block removed from `lookup_form`
- `Some(Binding::Dispatch { mm, .. }) =>` arm removed from `eval_lookup_define`
- `Some(Binding::Dispatch { mm, .. }) =>` arm removed from `eval_signature_of_defn`
- `Some(Binding::Dispatch { .. }) =>` arm removed from `body-of` dispatch
- Test harness dispatch block removed (DispatchRegistry::new + register_stdlib + set_dispatch_registry)

**`src/resolve.rs`:**
- Stone 241.11 dispatch_registry consultation block removed (confirmed already removed per summary)

**`src/special_forms.rs`:**
- `:wat::core::define-dispatch` entry removed from special-forms map

### S5 — Test cascade (6 files, per-file judgment)

| File | Action | Rationale |
|---|---|---|
| `tests/wat_arc146_dispatch_mechanism.rs` | DELETED | All 7 tests cover the retired define-dispatch mechanism; `probe_arc241_stone13` covers HARD-CUT acceptance |
| `tests/probe_arc237_7a_length_intrinsic.rs` | Comment updated | 6/6 pass; stale "works TODAY via define-dispatch" → "works as ∀T intrinsic (dispatch retired 241.13)" |
| `tests/probe_arc237_7b_intrinsic_typing.rs` | Comment updated | 7/7 pass; stale "CURRENT (define-dispatch) behavior" → "∀T intrinsic behavior" |
| `tests/wat_arc144_uniform_reflection.rs` | Assertion replaced | `dispatch_empty_lookup_define_emits_define_dispatch_head` → `primitive_empty_lookup_define_emits_define_head`; now asserts `:wat::core::define` head (Primitive reflection) and NO `define-dispatch` |
| `tests/probe_declaration_form_lift.rs` | Fixture migrated | Line 129 `":wat::core::define-dispatch"` removed from covered list; probe 3 (`define-dispatch in fn body`) DELETED; probe 6 (`probe_mixed_declaration_prelude_all_lift`) rewritten: define-dispatch + define replaced with defn; comment updated 8→7 forms |
| `tests/probe_def_not_special.rs` | Fixture migrated | Probe 5 (`probe_mixed_declaration_prelude_now_includes_def`): `(:wat::core::define ...)` → `(:wat::core::defn ...)`, `(:wat::core::define-dispatch ...)` replaced with plain `defn`; count updated 8→7 |

### S6 — `wat/core.wat` historical comment

Line 1-10 module header rewritten: "Originally used arc 146's `:wat::core::define-dispatch` (slice 1)... Stone 241.13 retired `:wat::core::define-dispatch` entirely (HARD CUT)".

### S7 — Emitter audit

```
grep -n "define-dispatch" src/closure_extract.rs src/runtime.rs src/check.rs
```

Results:
- `src/check.rs:7099,7102` — the HARD-CUT arm (correct)
- `src/check.rs:20108` — historical comment "Reborn from define-dispatch" (correct)
- `src/runtime.rs:5672,5676` — historical comments "Reborn from define-dispatch" (correct per D8: keep historical references)
- `src/runtime.rs:30798` — historical comment "MalformedForm (no-arm-match from define-dispatch)" (correct)
- `src/closure_extract.rs:1934` — doc comment mentioning define-dispatch in historical enumeration (correct)
- `src/dispatch.rs` — DELETED

**0 active emitters.** Gate CLEAN.

---

## Pre-INSCRIPTION Grep Gate

```
grep -rn ":wat::core::define-dispatch" src/ tests/ wat/
```

**All matches categorized:**

| Category | Files | Status |
|---|---|---|
| HARD-CUT arm | `src/check.rs:7099,7102` | ACCEPTABLE — the retirement arm itself |
| RETIREMENT_TABLE | `src/remedy/retirement.rs:37,58` | ACCEPTABLE — table entry + doc comment |
| Stone probe source | `tests/probe_arc241_stone13_define_dispatch_hard_cut.rs` | ACCEPTABLE — tests the HARD CUT (the form is in fixture source, not in substrate) |
| Migration notes | `tests/probe_def_not_special.rs:258`, `tests/probe_declaration_form_lift.rs:121,267` | ACCEPTABLE — migration doc comments |
| Historical comments | `tests/wat_arc144_uniform_reflection.rs:279,293`, `tests/probe_arc237_7a_length_intrinsic.rs:10`, `wat/core.wat:3,6` | ACCEPTABLE — historical references |

**Active substrate callers: 0**

Gate CLEAN.

---

## Honest Deltas

### Context boundary mid-flight

Stone 241.13 crossed a context boundary. The compacted summary preserved sufficient state (exact file/line/function names) for the continuation session to resume without re-discovery. The bad intermediate edit (duplicate `eval_eq` stub) from the `eval_dispatch_call_with_vals` deletion was diagnosed and corrected immediately.

### Pre-existing test failure

`wat_arc144_uniform_reflection::user_function_signature_and_body_return_some` was failing on Stone 241.12 commit (`7244cf43`) — confirmed by git checkout and test run. Not introduced by Stone 241.13. Per `feedback_pre_existing_verification`: pre-existing status independently verified; not deflected.

### Clippy: 905 vs 908 (Stone 241.12 baseline)

Stone 241.12 ended at 908. Stone 241.13 lands at 905 — 3 warnings removed by deleting `dispatch.rs` and the registry plumbing. Downward delta is healthy.

### Runtime harness simplification

The stdlib_loaded() harness in both `src/check.rs` and `src/runtime.rs` no longer needs the dispatch registration step before macro expansion. The harness is simpler: parse → register_defmacros → expand_all (with empty SymbolTable) → register types → register stdlib defines → done. No DispatchRegistry new/attach/clone steps.

### `wat/core.wat` now empty of executable forms at top

All `define-dispatch` declarations were already evacuated by arc 237 stones. `wat/core.wat` now contains only: historical comments, defalias forms (line 52-55), and the variadic fn defines + defmacros starting at line 100. No active dispatch declarations remain.

---

## Calibration

| Phase | Predicted | Actual |
|---|---|---|
| S1-S3 retirement table + dispatch.rs deletion | 10 min | ~10 min |
| S4 DispatchRegistry cascade (6 files) | 45-60 min | ~70 min (context boundary + bad intermediate edit repair) |
| S5 test cascade (6 files) | 20-30 min | ~25 min |
| S6 wat/core.wat comment | 5 min | ~5 min |
| S7 emitter audit | 5 min | ~5 min |
| S8 probe verification | 5 min | ~5 min (2/2 PASS) |
| S9 grep gate | 5 min | ~5 min (0 active callers) |
| S10 SCORE | 10-15 min | ~15 min |
| **Total** | **105-145 min** | **~140 min + context boundary overhead** |

Within-band. The S4 cascade was the dominant variable as predicted; context boundary overhead was absorbed by clean compaction.

---

## What This Unblocks

**Stone 241.14** — Enemy 3 (`:wat::core::define` eval-time residue; the remaining runtime eval arm for `define` that survived Stone 241.11's HARD CUT at check time)

**Stone 241.15** — INSCRIPTION closes arc 241

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`

**The define-family death campaign:** Enemy 1 (`:wat::runtime::define-alias`) DONE. Enemy 2 (`:wat::core::define-dispatch`) DONE. Enemy 3 (`:wat::core::define` eval residue) NEXT.
