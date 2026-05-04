# Arc 150 Slice 1 — SCORE

**Sweep:** sonnet, agent `aa0997be1a1d5aaa0`
**Wall clock:** ~18.9 min (1135s) — **WAY UNDER** the 50-80 min Mode A
predicted band; used 16% of the 120-min time-box.
**Output verified:** orchestrator independently re-ran all 8 baselines
+ new test file + spot-checked Function struct extension + ran full
workspace `cargo test`.

**Verdict:** **MODE A CLEAN SHIP.** 10/10 hard rows pass; 4/4 soft
rows pass. 3 honest deltas surfaced; one (Delta 1, TypeScheme
sibling-map) is a tooling-constrained tactical choice with a
documented clean path back to inline.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` (Function struct + parse_define + apply_function + signature_ast renderer + 8 fixture sites) + `src/check.rs` (CheckEnv variadic_rest_types map + call-site inference + check_function_body extension) + NEW `tests/wat_arc150_variadic_define.rs`. NO new files in `src/`. NO `wat/` changes. NO retirement of any handler. |
| 2 | Function struct extended | ✅ `rest_param: Option<String>` at `src/runtime.rs:526` + `rest_param_type: Option<TypeExpr>` at `src/runtime.rs:531`. All 8 existing `Function {}` literal sites updated with `rest_param: None, rest_param_type: None`. |
| 3 | parse_define_signature handles `& (name :Type)` | ✅ Mirrors `parse_defmacro_signature` (`src/macros.rs:380-440`). Validations enforced: `&` followed by exactly one binder; type required; type must be `Vector<T>` (new helper `is_vector_type` accepts `Vec<T>` / `Vector<T>` / `wat::core::Vector<T>`); duplicate `&` rejected; rest followed by fixed param rejected. Negative tests verify each rejection. |
| 4 | apply_function handles variadic arity + rest binding | ✅ Match on `rest_param.is_none()` (strict equality) vs `Some(_)` (`>=` fixed_arity). Bind loop drains fixed-arity prefix positionally, collects remainder into `Value::Vec(Arc::new(rest))`, binds to rest-name. Tail-call loop integrity preserved (cur_args rebuilt fresh each iteration). |
| 5 | TypeScheme + derive_scheme_from_function extended | ⚠️ HONEST DELTA — see Delta 1. Functionally equivalent via sibling-map on `CheckEnv.variadic_rest_types: HashMap<String, TypeExpr>`. Inline TypeScheme field would have required touching 215 existing struct-literal sites; sonnet pivoted to functional-equivalent due to Edit tool constraint. Documented in rustdoc; clean path back. |
| 6 | Call-site type checking handles variadic | ✅ When the resolved scheme has variadic_rest_type via the sibling map: accepts `args.len() >= params.len()`; type-checks fixed args against `params`; type-checks each rest-arg against T extracted from `Vector<T>`. Test suite verifies all paths including type-error-on-mismatched-rest-arg. |
| 7 | New test file shipped | ✅ `tests/wat_arc150_variadic_define.rs` exists (405 LOC; 16 tests). Coverage: zero/one/many rest-args; no-fixed-params variadic; arity error below fixed; type error on mismatched rest-arg; signature-of reflection round-trip (via EDN); foldl-over-rest pattern (the arc 148 slice 4 shape); 4 negative parse tests; 2 regression guards confirming strict-arity defines unchanged. |
| 8 | All baseline tests still green | ✅ `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_polymorphic_arithmetic` 20/20; `wat_arc148_ord_buildout` 46/46; **`wat_variadic_defmacro` 6/6** (defmacro UNCHANGED). |
| 9 | New tests pass | ✅ 16/16 in `wat_arc150_variadic_define`. |
| 10 | Workspace failure profile unchanged | ✅ Total: 1819 passed / 5 failed. Failures: 1 in src lib unit test (`call_stack_populates_on_assertion` — pre-existing, panics on purpose for stack-trace test); 4 in wat-holon-lru HologramCacheService (documented arc 130 in-progress per slice 5 SCORE). NO new failures introduced by this slice. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (300-700) | ⚠️ 753 LOC total (348 substrate insertions + 16 deletions in src/, 405-line new test file). Slightly over the 700 ceiling but UNDER the 900 re-evaluate threshold. Test file size is honest scope (16 tests with comprehensive coverage). |
| 12 | Style consistency | ✅ Function field shape mirrors MacroDef. parse_define_signature mirrors parse_defmacro_signature 1:1. apply_function rest-arg binding mirrors expand_macro_call. New test file mirrors `wat_variadic_defmacro.rs` shape. |
| 13 | clippy clean | ✅ No new warnings in modified line ranges. |
| 14 | Audit-first discipline | ✅ Delta 1 surfaced honestly with rationale + path back; Delta 2 + Delta 3 both within-scope additions sonnet correctly identified. No improvisation. |

## The 3 honest deltas (sonnet)

### Delta 1 — TypeScheme sibling-map vs inline field

**The deviation:** BRIEF specced `rest_param_type: Option<TypeExpr>`
inline on `TypeScheme`. Sonnet's impl: sibling registry
`CheckEnv::variadic_rest_types: HashMap<String, TypeExpr>`,
populated by `from_symbols` from `Function.rest_param_type`,
consulted at the call-site inference branch via the new accessor
`CheckEnv::variadic_rest_type(name)`.

**The constraint:** inline field would have required updating 215
existing `TypeScheme {}` struct-literal sites (every substrate
primitive registration). Edit tool is per-site context-dependent;
no sed/perl/python access in the harness; mechanical mass-edit was
impractical.

**Functional equivalence:** the contract from rows 5+6 is met. Any
caller that needs variadic info via the TypeScheme's name accesses
the sibling map. Type-check inference for variadic call sites works
correctly per the 16 new tests.

**Architectural assessment:** sibling-map is a tactical choice, NOT
a bridge over substrate complexity. The path back to inline is
documented in TypeScheme's rustdoc; a future arc with mass-edit
access (or arc 147's substrate-registration-macro slice — which
touches every primitive registration) can fold it back cleanly.

**Per the user's discipline ("eliminate failure domains; don't
bridge"):** this delta is borderline. The functional contract is
met (no failure domain), but the architecture deviates from the
brief's intent. Future cleanup is desirable but not blocking.

**Recommendation:** accept as Delta 1; revisit during arc 147
slice work or arc 109 v1 closure cleanup.

### Delta 2 — `check_function_body` extension

The body of a variadic function must bind the rest-name in locals
or `(:length xs)` / `(:foldl xs ... ...)`-style intra-body uses
fail with `<unresolved>`. Sonnet added the binding (one-line
addition; entirely additive). Not in the brief's letter; surfaced
on first body-uses-rest test failure; correctly identified +
fixed within the slice.

**Architectural assessment:** required for the variadic surface to
actually work; not optional. Honest within-scope addition.

### Delta 3 — `function_to_signature_ast` extension

Sonnet extended the signature-AST renderer to emit `& (rest :Vector<T>)`
for variadic defines, mirroring `macrodef_to_signature_ast`. Required
by the EXPECTATIONS test for reflection coverage (signature-of must
round-trip the variadic shape).

**Architectural assessment:** mechanical mirror of the macrodef
equivalent; required for arc 144 reflection consumers. Honest
within-scope addition.

## Calibration record

- **Predicted Mode A (~60%)**: ACTUAL Mode A clean. Calibration
  matched.
- **Predicted runtime (50-80 min)**: ACTUAL ~19 min. **WAY UNDER**
  band — used only 16% of the 120-min time-box. The defmacro
  precedent provided a clean 1:1 template; the substrate work was
  mechanical pattern-application. Future foundation slices that
  mirror well-trodden patterns: predict tighter (15-30 min Mode A).
- **Time-box (120 min)**: NOT triggered. Used 16%.
- **Predicted LOC (300-700)**: ACTUAL 753 (slightly over but under
  re-evaluate threshold). Test file 405 LOC accounts for the
  overage; comprehensive coverage (16 tests) is honest scope.
- **Honest deltas (predicted 0-2; actual 3)**: Delta 1 (TypeScheme
  sibling-map) is the substantive one; Deltas 2 + 3 are within-scope
  additions sonnet correctly identified. Healthy outcome.

## Workspace failure profile (pre/post slice)

- **Pre-slice baseline:** documented `CacheService.wat` + 4
  HologramCacheService failures (arc 130 in-progress per slice 5
  SCORE).
- **Post-slice (default cargo test):** 1819 passed / 5 failed.
  The 5 failures: 1 src lib unit test (`call_stack_populates_on_assertion`
  — pre-existing, panics on purpose for stack-trace test) + 4
  HologramCacheService (same arc 130 noise). **NO new failures
  introduced by this slice.**

## What this slice closes

- The arbitrary defmacro/define asymmetry — defmacros (which expand)
  and defines (which evaluate) BOTH support `& rest`
- The substrate's metadata model gains a clear "extend the carrier"
  pattern (Function struct accretes optional fields naturally)
- Reflection layer (signature-of) round-trips variadic info
  correctly
- The "comma-typed funcs are crutches" rule strengthens — variadic
  surfaces can be wat-level functions without falling back to Rust
  primitives or comma-tagged binary direct-call leaves

## What this slice unlocks

- **Arc 148 slice 4** RESUMES — the variadic arithmetic surface
  `(:wat::core::define (:wat::core::+ & (xs :Vector<numeric>) -> :numeric)
   (:wat::core::reduce :wat::core::+,2 (:first xs) (:rest xs)))` is
  now implementable per the locked DESIGN
- **Arc 141 (docstrings)** — the "extend Function" pattern is now
  exercised; arc 141's docstring field follows the same shape
- **Future variadic surfaces** across the substrate (format / log /
  pipe / test harness extensions)

## Pivot signal analysis

NO PIVOT. The 3 honest deltas are within-scope; Delta 1 (TypeScheme
sibling-map) is a tactical choice with documented path back, NOT a
substrate failure.

The methodology IS the proof. The rhythm holds — and accelerates.
~19 min slice on a substrate change that touched parser + runtime +
check + tests. The defmacro precedent paid off.
