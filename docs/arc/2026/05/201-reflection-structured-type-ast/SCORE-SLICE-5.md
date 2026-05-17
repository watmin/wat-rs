# Arc 201 Slice 5 SCORE — `extract-arg-types` substrate primitive

**Slice:** 5 of arc 201. Commits: PENDING (orchestrator commits atomically).
**Date:** 2026-05-16.
**BRIEF:** `BRIEF-SLICE-5.md`.

## Scorecard

| Row | What | Verdict | Evidence |
|-----|------|---------|---------|
| A | `:wat::runtime::extract-arg-types` minted (eval + dispatch + type-scheme registration) | YES | `eval_extract_arg_types` at `src/runtime.rs` (after `eval_extract_arg_names`); dispatch arm at `src/runtime.rs:4053` (`:wat::runtime::extract-arg-types` → `eval_extract_arg_types`); `infer_list` special-case + `env.register` entry in `src/check.rs`; all 5 unit tests call the verb successfully |
| B | Monomorphic arg types extracted as atomic Symbols | YES | `extract_arg_types_returns_atoms_for_monomorphic_args` passes; `:wat::core::String` and `:wat::core::i64` appear as standalone Symbol payloads in rendered EDN; length verified as 2 |
| C | Parametric arg types extracted as Bundles per slice 1 emission rules | YES | `extract_arg_types_returns_bundles_for_parametric_args` passes; `:wat::core::Vector` head appears as standalone Symbol; flattened `:wat::core::Vector<wat::core::i64>` spelling absent |
| D | Composes with `Bundle/children` for D2 algorithm chain | YES | `extract_arg_types_composes_with_bundle_children_on_parametric` passes; `Option/expect` used to unwrap `get` result (see honest delta below); `Bundle/children` on the extracted Vector type-AST yields `[Symbol(:wat::core::Vector), Symbol(:wat::core::i64)]` |
| E | Workspace test failure count ≤ baseline (3 stable + 1 lifeline flake variance) | YES | Post-slice workspace run: 3 pre-existing failures (`deftest_wat_tests_tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`, `t6_spawn_process_factory_with_capture_round_trips`); lifeline flake passed this run (0/100 trials failed); 5 new tests all pass |

**5/5 PASS.**

## Honest deltas

### Walker factoring decision: parallel handler (near-duplicate)

Kept `eval_extract_arg_types` as a near-duplicate of `eval_extract_arg_names`. The handlers differ in exactly one line: `pair[0]` (name keyword) vs `pair[1]` (type AST) and `names: Vec<Value>` vs `types: Vec<Value>`. Per `feedback_simple_is_uniform_composition`: two near-identical handlers with one semantic difference are cleaner than a shared walker parameterised on slot index. The parameterised version would need a `fn(&HolonAST) -> Option<Value>` closure or an enum discriminant — adding indirection to save ~15 lines. Parallel handlers win YES YES YES YES; factored walker fails Simple.

### Return-type lifting pattern

`Value::Vec(Arc::new(types))` — same shape as `eval_extract_arg_names`. Each type AST is wrapped as `Value::holon__HolonAST(Arc::new(pair[1].clone()))` — `clone()` is necessary because `require_bundle` borrows `&Vec<HolonAST>` and we need to own the value. The clone cost is bounded: type ASTs are small trees (depth ≤ parametric nesting; typically 2-3 nodes).

### Arc 057/143 surface check (pre-implementation)

Searched `extract-arg`, `arg-types`, `signature-types`, `param-types` in `src/runtime.rs` and `src/check.rs` before implementation. Result: zero hits for `extract-arg-types` anywhere in the codebase. Confirmed genuinely additive — no pre-existing primitive serving this purpose.

### Variadic-rest handling

`eval_extract_arg_names`'s walker uses `HolonAST::Bundle(pair) if pair.len() == 2` — this guard excludes variadic-rest binders if they emit a different structure. Inspection of `function_to_signature_ast` (src/runtime.rs, the variadic emission path around line ~9128) showed rest params emit the same 2-child pair structure (`[name, type]`) — same shape as strict params. So the `pair.len() == 2` guard admits them uniformly. `eval_extract_arg_types` mirrors this identically — variadic type ASTs are extracted alongside strict param type ASTs, in binder order. This is correct: the D2 algorithm (which `extract-arg-types` enables) needs the complete positional sequence.

### D2 test — `Option/expect` to unwrap `get` result

Test 4 (`extract_arg_types_composes_with_bundle_children_on_parametric`) initially used `(:wat::core::get tys 0)` directly as the `Bundle/children` input. First run: `TypeMismatch { op: ":wat::holon::Bundle/children", expected: "HolonAST (Bundle)", got: "wat::core::Option" }`. Root cause: `:wat::core::get` returns `Option<T>` not `T`. Fixed by adding `Option/expect` unwrap between `get` and `Bundle/children`. One diagnostic cycle; not a substrate gap — the caller must unwrap `get`'s Option (same as any other Vector accessor call site).

### Naming — no `/gaze` needed

Working name `extract-arg-types` confirmed. It is the exact parallel to `extract-arg-names` (one word changes: `names` → `types`). Alternatives (`extract-param-types`, `arg-types`, `type-of-args`) were not considered further — the parallel naming wins YES YES YES YES immediately. Zero `/gaze` exchanges.

### check.rs registration ordering

`vec_holon_ty()` closure is defined in the Arc 201 slice 2 block, which is BELOW the Arc 143 + slice 5 registration block. To avoid a forward reference, the `extract-arg-types` registration inlines `TypeExpr::Parametric { head: "wat::core::Vector", args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())] }` directly (identical to what `vec_holon_ty()` returns). Honest, local, no dependency on lexical order of the `vec_holon_ty` closure.

## Files touched

- `src/runtime.rs` — added `eval_extract_arg_types` fn + dispatch arm (`:wat::runtime::extract-arg-types`)
- `src/check.rs` — added `infer_list` special-case + `env.register` entry for `:wat::runtime::extract-arg-types`
- `tests/wat_arc201_extract_arg_types.rs` — 5 new unit tests (new file)

## Workspace baseline delta

- Pre-slice: 2323 passed / 3 failed (stable pre-existing)
- Post-slice: 2328 passed / 3 failed (5 new tests added; no regressions)
- Lifeline flake: passed this run (0/100 trials failed); variance documented as expected

## STOP triggers fired

**0.** None of the 7 STOP triggers fired:
1. `eval_extract_arg_names` shape matched BRIEF description exactly (walker with head/arrow/ret skipping via `skip(1)` + break-on-`"->"` + `pair.len() == 2` guard).
2. Pair-Bundle structure confirmed as exactly 2 children `[name, type]`.
3. Return-type lifting worked directly: `Value::Vec(Arc::new(types))` with `Value::holon__HolonAST(Arc::new(pair[1].clone()))` per item.
4. N/A — extraction required only `clone()` (cheap; no ownership issue).
5. Workspace baseline not regressed.
6. No new substrate types/structs/special-forms minted.
7. No wat-side composition temptation; implemented substrate-side per Q2 resolution in DESIGN.md § Slice 5.
