# Arc 150 Slice 1 — Pre-handoff expectations

**Drafted 2026-05-03.** MEDIUM substrate slice — Function struct
extension + parser + runtime + check-side type inference + new test
file. Predicted MEDIUM-LARGE slice (Mode A ~60%; Mode B-check-side-
coupling ~20%; Mode B-tail-call-rest-binding ~10%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-1.md`
**Output:** EDITS to `src/runtime.rs` (Function + parse_define +
apply_function) + `src/check.rs` (TypeScheme + derive_scheme +
call-site inference) + NEW `tests/wat_arc150_variadic_define.rs`.

## Setup — workspace state pre-spawn

- Arc 148 slice 5 shipped (`SCORE-SLICE-5.md`); comparison surface
  reached final shape.
- Arc 132 amend just shipped (`commit 0a8d6e5`); default deftest
  time-limit raised from 200ms to 1000ms; workspace failure profile
  cleaned up to documented arc 130 in-progress noise only.
- Arc 148 slice 4 PAUSED pending arc 150 closure.
- Workspace baseline (per FM 9, post-slice-5): reflection-layer
  baselines all green (45/45 across 5 test files); `wat_polymorphic_arithmetic`
  20/20; `wat_arc148_ord_buildout` 46/46.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (`Function` struct + `parse_define_signature` + `apply_function`) + `src/check.rs` (`TypeScheme` + `derive_scheme_from_function` + call-site inference) + NEW `tests/wat_arc150_variadic_define.rs`. NO new files in `src/`. NO `wat/` changes. NO retirement of any handler. |
| 2 | Function struct extended | `pub rest_param: Option<String>` + `pub rest_param_type: Option<TypeExpr>` fields added at `src/runtime.rs:499+`. Both default to `None` for existing strict-arity defines. |
| 3 | parse_define_signature handles `& (name :Type)` | `(:wat::core::define (verb (a :i64) & (rest :Vector<i64>) -> :Vector<i64>) ...)` parses cleanly. Validations enforced: `&` followed by exactly one binder; type required; type must be `Vector<T>`; multiple `&` rejected; rest followed by fixed param rejected. Mirrors `src/macros.rs:380-450` shape. |
| 4 | apply_function handles variadic arity + rest binding | When `rest_param.is_some()`: accepts `args.len() >= params.len()`; collects extras into `Value::Vec(Arc::new(rest))`; binds to rest-name. When `rest_param.is_none()`: existing strict-arity behavior unchanged (no regression for existing defines). |
| 5 | TypeScheme + derive_scheme_from_function extended | TypeScheme gains `rest_param_type: Option<TypeExpr>`. derive_scheme_from_function populates it from `Function.rest_param_type`. |
| 6 | Call-site type checking handles variadic | When the resolved scheme has `rest_param_type.is_some()`: accepts `args.len() >= params.len()`; type-checks fixed args against `params`; type-checks each rest-arg against T extracted from `Vector<T>`; type errors surface cleanly. |
| 7 | New test file shipped | `tests/wat_arc150_variadic_define.rs` exists with the coverage listed in BRIEF (variadic + rest-args 0/1/many; no-fixed-params; arity error; type error; signature-of reflection; reduce-over-rest pattern; parse-error negative tests). |
| 8 | All baseline tests still green | `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_polymorphic_arithmetic` 20/20; `wat_arc148_ord_buildout` 46/46; `wat_variadic_defmacro` (existing — must still pass since defmacro is untouched). |
| 9 | New tests pass | All test cases in `tests/wat_arc150_variadic_define.rs` pass. |
| 10 | Workspace failure profile unchanged | Pre-slice + post-slice both: only documented `CacheService.wat` + `HologramCacheService` arc 130 in-progress noise. |

**Hard verdict:** all 10 must pass. Rows 3 + 4 + 6 are the load-
bearing rows (parser + runtime + check-side all working end-to-end).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 300-700 LOC (substrate edits ~150-300 + new test file ~150-400). >900 LOC = re-evaluate scope. |
| 12 | Style consistency | Function field shape mirrors MacroDef. parse_define_signature mirrors parse_defmacro_signature. apply_function rest-arg binding mirrors expand_macro_call's rest-binding. New test file mirrors `wat_variadic_defmacro.rs`'s pattern. |
| 13 | clippy clean | No new warnings. |
| 14 | Audit-first discipline | If sonnet finds a substrate edge that can't be cleanly extended (e.g., `Function`'s use across the codebase has hardcoded assumptions about `params.len()` matching exactly), surface as honest delta. Don't improvise. |

## Independent prediction

- **Most likely (~60%) — Mode A clean ship.** Substrate work is
  well-defined; the defmacro precedent provides a 1:1 template.
  The check-side type inference is the riskiest piece; ~50-80 min
  wall-clock.
- **Mode B-check-side-coupling (~20%):** the call-site inference
  for variadic Function callees has a non-trivial coupling with
  existing TypeScheme machinery (parametric instantiation, rest_type
  vs fresh-vars, etc.). Surfaces as honest delta; orchestrator may
  ship a follow-up to refine.
- **Mode B-tail-call-rest-binding (~10%):** apply_function's tail-
  call loop has subtle interactions with rest-arg Vec creation
  (re-collecting on each iteration; shared closure state). Sonnet
  surfaces.
- **Mode C (~10%):** unforeseen substrate edge (lambda interaction;
  scope-deadlock interaction; reflection edge).

## Time-box

120 min wall-clock (≈1.5× the predicted upper-bound of 80 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

**Arc 148 slice 4 RESUMES.** The variadic arithmetic surface as a
wat-level function reducing over the binary Dispatch becomes
implementable per the locked DESIGN.

Beyond arc 148: every future variadic surface (format, log, pipe,
etc.) is now expressible.

The substrate becomes consistent — defmacros and defines both
support `& rest`.

## After sonnet completes

- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows (3, 4, 6) by spot-checking + running tests
- Write `SCORE-SLICE-1.md`
- Commit the SCORE before drafting slice 2's BRIEF (closure)
