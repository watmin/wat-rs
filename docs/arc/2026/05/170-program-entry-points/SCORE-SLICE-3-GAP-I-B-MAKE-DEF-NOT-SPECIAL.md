# Arc 170 slice 3 Gap I-B SCORE — make `def` not special

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2243 passed / 0 failed (+5 probes over Gap I-A baseline of 2238)

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::core::def` arm in `validate_def_position_with_wrapper` DELETED (def now falls through `_ =>` like the 7 siblings); check-time validator silent for def | grep + read | PASS — arm deleted at `src/check.rs:7095`; replaced by a retirement comment explaining the symmetry. `def` falls through to `_ =>` arm like the other 7 declaration forms. |
| B | `:wat::core::def` arm in eval dispatch (`runtime.rs:3518-3532`) emits position-class error (chosen variant per Phase 2); does NOT silently return Unit | grep + read | PASS — the permissive arm (evaluate RHS, return Unit) is replaced by `Err(RuntimeError::DeclarationInExpressionPosition(":wat::core::def".into(), list_span.clone()))` at `src/runtime.rs:3529`. |
| C | Error variant minted/renamed per Phase 2 decision (α: mint `DeclarationInExpressionPosition` carrying head + span; route both def + define through; retire `DefineInExpressionPosition` via sweep); Display renders the correct head | grep + read + Display test | PASS — `DeclarationInExpressionPosition(String, Span)` minted at `src/runtime.rs:1061`. Display renders `"{head} is a declaration form, not an expression — declaration forms are top-level registration forms and cannot appear in expression position"`. `DefineInExpressionPosition` variant retired. Both def and define arm route through the new variant. Probe 4 verifies define still rejects with `DeclarationInExpressionPosition` carrying `":wat::core::define"` as head. |
| D | 5+ new probes in `tests/probe_def_not_special.rs` pass: end-to-end spawn lift; runtime position error; top-level regression; define regression; mixed 8-form prelude | cargo test | PASS — `5 passed; 0 failed` (`probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end`, `probe_def_at_expression_position_emits_position_error_at_runtime`, `probe_def_at_top_level_still_works`, `probe_define_at_expression_position_still_emits_error`, `probe_mixed_declaration_prelude_now_includes_def`) |
| E | All 6 Gap I-A probes + all 5 Gap H probes + all 11 prior substrate probes still pass | cargo test | PASS — 6/6 Gap I-A probes pass; 5/5 Gap H probes pass; all 11 prior substrate probe test files pass. No regressions. |
| F | `cargo check --release` green; workspace at 2238 + N (- M_updated) / 0 failed; M is the test sweep size (Phase 1 audit reveals) | full test run | PASS — `2243 passed; 0 failed` (+5 probes over baseline 2238; M_updated = 3 tests changed assertion shape, same count). |

**All 6 rows PASS.**

---

## Phase 1 audit inventory

### DefNotTopLevel sites

| Location | Kind |
|----------|------|
| `src/check.rs:475` | variant definition (RETAINED — orphaned, no emitter after I-B) |
| `src/check.rs:736` | Display impl arm |
| `src/check.rs:1041-1042` | Diagnostic emission |
| `src/check.rs:1786` | comment in `check_program` |
| `src/check.rs:7042` | docstring comment |
| `src/check.rs:7097` (pre-I-B) | **the only emitter** — RETIRED (arm deleted in Phase 3) |
| `tests/probe_declaration_form_lift.rs:14,108,309` | comments only; document the Gap I-A/I-B dependency |
| `tests/wat_arc166_defn.rs:16,196,213,214` | test assertions — 1 live test (test 6) changed |
| `tests/wat_arc157_def.rs:9,207,218,219,224,231,241,242` | test assertions — 2 live tests (tests 9, 10) changed |

**DefNotTopLevel emitter count:** 0 after retirement (was 1). Variant remains as orphaned scaffolding (arc 113 precedent).

### DefineInExpressionPosition sites

| Location | Kind |
|----------|------|
| `src/runtime.rs:1065` (pre-I-B) | variant definition — **RETIRED** |
| `src/runtime.rs:1319` (pre-I-B) | Display impl arm — **RETIRED** |
| `src/runtime.rs:3539` (pre-I-B) | the only emitter — **RETIRED** (define arm now uses `DeclarationInExpressionPosition`) |
| `src/closure_extract.rs:292,1785,1822` | comments only — unchanged |
| `tests/probe_closure_body_prelude_lift.rs:8,25,36,108,259` | comments only — unchanged |
| `tests/probe_deftest_hermetic_isolation.rs:35` | comment only — unchanged |

**DefineInExpressionPosition emitter count:** 0 (variant retired). No test pattern-matched on it — all references were comments.

### def-at-expression-position test assertions (live)

| Test | File | Pre-I-B assertion | Post-I-B assertion | Reason |
|------|------|-------------------|--------------------|--------|
| `def_position_illegal_inside_if` | `tests/wat_arc157_def.rs` | `startup_err` + `contains("DefNotTopLevel")` | `startup_ok` (with corrected `if` syntax using `-> :T`) | Check-time validator retired; `if`-branch defs silently pass startup |
| `def_position_illegal_inside_define_body` | `tests/wat_arc157_def.rs` | `startup_err` + `contains("DefNotTopLevel")` | `startup_ok` | Same: function body defs pass startup after I-B |
| `defn_rejected_inside_if_branch` | `tests/wat_arc166_defn.rs` | `startup_err` + `contains("DefNotTopLevel")` | `startup_ok` (with corrected `if` syntax) | `defn` expands to `def`; same retirement applies |

**Test sweep size:** 3 live tests updated. Well within the 20-site STOP threshold.

---

## Phase 2 variant naming choice — (α) chosen

**Decision:** Mint `DeclarationInExpressionPosition(String, Span)` carrying the offending form's head. Route both `":wat::core::def"` and `":wat::core::define"` through it. Retire `DefineInExpressionPosition`.

**Rationale — four questions:**

- **Obvious?** YES — the name `DeclarationInExpressionPosition` is symmetric with `is_declaration_form` (Gap I-A's predicate name). A reader who knows the predicate names the variant by the same vocabulary.
- **Simple?** YES — one variant for all 8 forms; the `String` head carries identity. No proliferation of per-form variants.
- **Honest?** YES — `DefineInExpressionPosition` LIED when applied to `def` (the variant name referenced "define" but would now name "def"). The new variant is honest for any declaration form.
- **Good UX?** BETTER — one error class, consistent Display, head visible in the message. Users see `":wat::core::def" is a declaration form, not an expression...` vs the old define-specific message.

**(β)** and **(γ)** rejected: β leaves two near-identical variants; γ has a lying variant name ("DefineInExpressionPosition" for def). Both fail Honest.

---

## Test sweep enumeration

### `tests/wat_arc157_def.rs::def_position_illegal_inside_if` (Test 9)

**Before:** `startup_err` → assert `contains("DefNotTopLevel")` AND `contains(":wat::core::if")`

**After:** `startup_ok` with corrected `if` form syntax (`-> :wat::core::nil` return type, `true` bool literal).

**Why:** After retiring the validator's def arm, `def` inside `if` passes check-time. The `if`-branch is not processed by `register_runtime_defs_form` (only `do`/`let`/`def` arms exist there). Startup succeeds.

**If-syntax fix needed:** The original source used `:wat::core::true` (a keyword, not a bool) and the old `if` shape without `-> :T`. Both of these caused independent errors; after retiring `DefNotTopLevel`, only the form-shape errors remained visible. The test was updated to use `true` (bool literal) and `-> :wat::core::nil` to satisfy the `if` form's shape requirements.

### `tests/wat_arc157_def.rs::def_position_illegal_inside_define_body` (Test 10)

**Before:** `startup_err` → assert `contains("DefNotTopLevel")`

**After:** `startup_ok` (source unchanged — `define` body with `def` inside).

**Why:** After retiring the validator's def arm, the function-body check (`validate_def_position_with_wrapper` with NonTopLevel context on function bodies, lines 1793-1800 in check.rs) also no longer emits `DefNotTopLevel` for `def`. Startup succeeds.

### `tests/wat_arc166_defn.rs::defn_rejected_inside_if_branch` (Test 6)

**Before:** `startup_err` → assert `contains("DefNotTopLevel")`

**After:** `startup_ok` with corrected `if` form syntax (`-> :wat::core::nil`, `true` bool literal).

**Why:** `defn` expands to `def` before the position check runs. The expanded `def` no longer triggers `DefNotTopLevel`. Same `if`-syntax fix as Test 9 above.

---

## Public API impact assessment

`RuntimeError` is re-exported via `src/lib.rs:149-152`:
```rust
pub use runtime::{
    eval, register_defines, register_struct_methods, EncodingCtx, EnvBuilder, Environment,
    Function, RuntimeError, StructValue, SymbolTable, Value,
};
```

`RuntimeError::DefineInExpressionPosition(Span)` was a public variant. It is **retired** in this slice.

`RuntimeError::DeclarationInExpressionPosition(String, Span)` is **minted** as its replacement.

**Impact:** Any external code that pattern-matched on `RuntimeError::DefineInExpressionPosition` will fail to compile after upgrading to post-I-B substrate. Audit of the in-tree codebase found zero pattern-match uses of `DefineInExpressionPosition` in tests or src — all 11 references in test files were comments. The variant was not used in any external crate within the workspace.

**Severity:** API-breaking for downstream consumers. The variant name was define-specific; the replacement is more accurate (covers all 8 declaration forms). A consumer searching for `DefineInExpressionPosition` would update to `DeclarationInExpressionPosition` matching on the `String` head if they need define-specific filtering.

---

## Honest deltas

### Delta 1 — `if`-syntax fix required for test sweep (unexpected)

The three tests being swept (test 9, test 6) used invalid `if` syntax — both `:wat::core::true` (a keyword, not a bool literal) and the old `if` shape without `-> :T`. Before I-B, these tests failed with MULTIPLE check errors, and `DefNotTopLevel` was among them (emitted first by the position-validator pass), so `err.contains("DefNotTopLevel")` was true and the tests passed.

After I-B, `DefNotTopLevel` is no longer emitted. The remaining errors were form-shape errors for the `if`. Changing the tests to `startup_ok` without fixing the `if` syntax caused the test to fail with `MalformedForm` (if missing `-> :T`) and `TypeMismatch` (`:wat::core::true` is a keyword, not a bool).

Fix: updated both tests to use `true` (bool literal) and `-> :wat::core::nil` (the proper nil return type, since `def` infers as nil). These syntax corrections reflect the current `if` form requirements (not introduced by I-B — they were always required; the old test was depending on `DefNotTopLevel` being emitted before the `if`-shape check could fire, obscuring the pre-existing invalid syntax).

### Delta 2 — `CheckError::DefNotTopLevel` orphaned variant remains

After retiring the sole emitter (the `":wat::core::def"` arm), `DefNotTopLevel` has no live emitter sites. The variant definition, Display impl, and Diagnostic impl all remain in `src/check.rs`.

**Per BRIEF constraint:** do NOT delete `CheckError::DefNotTopLevel` in this slice. The variant cleanup (definition + Display + Diagnostic arm) belongs in a dedicated retirement sweep after a consumer audit. This is affirmative scope-bounding (a separate arc closes it), not a deferral.

**What needs to happen:** a follow-up arc should grep `CheckError::DefNotTopLevel` across the codebase, verify no live emitters exist, and delete the variant + all matching arms. The variant name should not be repurposed (it was specific to def's position check, which is now runtime-enforced).

### Delta 3 — runtime test approach: `eval_in_frozen` cannot test def directly (must call wrapper fn)

Probe 2 (def at runtime expression position) cannot call `(:wat::core::def :x 1)` directly via `eval_in_frozen`. The `freeze::refuse_mutation_forms` check runs before `eval` and catches `def` as a mutation form, returning `EvalForbidsMutationForm` instead of `DeclarationInExpressionPosition`.

The correct approach (used in Probe 2): define a function `(:my::bad -> :wat::core::nil)` whose body is `(:wat::core::def :x 1)`, then call `(:my::bad)` via `eval_in_frozen`. The function-call AST is not a mutation form; `refuse_mutation_forms` passes. The function body evaluates and hits the tightened def arm in `dispatch_keyword_head`, emitting `DeclarationInExpressionPosition`.

This is architecturally correct: `refuse_mutation_forms` catches defs at the dynamic-eval boundary (protecting against code injection); `DeclarationInExpressionPosition` catches defs that reach eval through a function body at expression position. The two mechanisms are complementary, not redundant.

### Delta 4 — test sweep discovered pre-existing invalid `if` syntax in arc 157 + arc 166 test files

The `if` form used in Test 9 (`wat_arc157_def.rs`) and Test 6 (`wat_arc166_defn.rs`) used `:wat::core::true` (a keyword, not a bool literal) and omitted the required `-> :T` return-type annotation. These were pre-existing syntax violations that were masked by the `DefNotTopLevel` error being emitted before the `if`-shape check. The sweep corrected both to use `true` (bool literal) and `-> :wat::core::nil`. The corrections are independent of Gap I-B's semantic changes.

### Delta 5 — Gap I-A probe 6 docstring now stale

`tests/probe_declaration_form_lift.rs::probe_mixed_declaration_prelude_all_lift` has this in its docstring:

> `def` is intentionally omitted from this end-to-end probe...

After Gap I-B ships, Probe 5 in `probe_def_not_special.rs` covers the 8-form mixed prelude including `def`. The Gap I-A probe 6 docstring's explanation of `def`'s absence is now historically accurate (it correctly documents why `def` was omitted AT THAT TIME) but could be confusing to future readers. A follow-up arc may annotate it "this probe predates Gap I-B; see `probe_mixed_declaration_prelude_now_includes_def` in `probe_def_not_special.rs` for the full 8-form proof."

---

## Files changed

| File | Change |
|------|--------|
| `src/check.rs` | Deleted `:wat::core::def` arm in `validate_def_position_with_wrapper` (lines 7094-7111 pre-I-B); replaced with retirement comment. `def` now falls through to `_ =>`. |
| `src/runtime.rs` | (1) Retired `DefineInExpressionPosition(Span)` variant; replaced with `DeclarationInExpressionPosition(String, Span)` with updated Display. (2) Replaced permissive `":wat::core::def"` eval arm with `Err(RuntimeError::DeclarationInExpressionPosition(":wat::core::def".into(), ...))`. (3) Updated `":wat::core::define"` eval arm to use `DeclarationInExpressionPosition` instead of `DefineInExpressionPosition`. |
| `tests/wat_arc157_def.rs` | Test 9 (`def_position_illegal_inside_if`): changed from `startup_err` + `DefNotTopLevel` assertion to `startup_ok` with corrected `if` syntax. Test 10 (`def_position_illegal_inside_define_body`): changed from `startup_err` + `DefNotTopLevel` assertion to `startup_ok`. |
| `tests/wat_arc166_defn.rs` | Test 6 (`defn_rejected_inside_if_branch`): changed from `startup_err` + `DefNotTopLevel` assertion to `startup_ok` with corrected `if` syntax. |
| `tests/probe_def_not_special.rs` | CREATED — 5 Gap I-B probes. |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-I-B-MAKE-DEF-NOT-SPECIAL.md` | CREATED — this file. |

---

## Workspace delta

- Baseline (post-Gap-I-A, commit `8c13631`): 2238 passed / 0 failed
- Post-Gap-I-B: 2243 passed / 0 failed (+5 new probes; 3 tests changed assertion shape, same count)

---

## Verification commands run

```
# New Gap I-B probes
cargo test --release --test probe_def_not_special
# → 5 passed; 0 failed

# Gap I-A regression (CRITICAL)
cargo test --release --test probe_declaration_form_lift
# → 6 passed; 0 failed

# Gap H regression
cargo test --release --test probe_closure_body_prelude_lift
# → 5 passed; 0 failed

# All 11 prior substrate probes
cargo test --release \
  --test probe_do_splice_def --test probe_let_splice_def \
  --test probe_do_splice_define --test probe_let_splice_define \
  --test probe_do_splice_struct --test probe_do_splice_enum \
  --test probe_let_splice_struct --test probe_let_splice_enum \
  --test probe_spawn_process_parent_type \
  --test probe_resolver_quote_awareness \
  --test probe_deftest_hermetic_isolation
# → all 11 files pass; no regressions

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result:" | \
    awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# → passed:2243 failed:0
```

---

## Cross-references

- `8c13631` Gap I-A (the slice that surfaced Gap I-B's necessity; is_declaration_form predicate)
- `36030c3` Gap H (lift mechanism that Gap I-B's def-fix unblocks end-to-end)
- arc 157 (the arc that minted def's check-time validator AND the permissive runtime arm assumption Gap I-B corrects)
- arc 113 (orphaned-scaffolding precedent for leaving `CheckError::DefNotTopLevel` without emitters)
- After Gap I-B: Phase 2a complete (F-1, F-3, F-2, G, H, I-A, I-B all shipped); deftest-hermetic Path E macro shape is the next small slice
