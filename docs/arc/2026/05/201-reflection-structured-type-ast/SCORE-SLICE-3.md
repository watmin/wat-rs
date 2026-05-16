# SCORE — Arc 201 Slice 3 — `signature-of-fn` primitive

**Slice:** 3 of 6 (see DESIGN.md § Stepping stones).
**Date:** 2026-05-16.
**Predecessors:** slice 1 (`0706949`) — structured type-AST emission; slice 2 (`c9445a4`) — `Bundle/children` + `Bundle/first` accessors.

## SCORE rows

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `:wat::runtime::signature-of-fn` minted (eval handler + dispatch arm + type scheme + check-side special case) | **YES** | `src/runtime.rs` — `eval_signature_of_fn` (inserted after `eval_signature_of`, ~70 lines including docstring) + dispatch arm at line ~4047. `src/check.rs` — `infer_list` special-case arm (~25 lines) + `env.register` entry (~13 lines including comment block). The 8 tests in `tests/wat_arc201_signature_of_fn.rs` call the verb via `(:wat::runtime::signature-of-fn ...)` and pass. |
| B | Output shape matches `signature-of`'s structure for named user defines (head + arg-pairs + arrow + ret) | **YES** | Composition test `signature_of_fn_composes_with_extract_arg_names` PASSES — `extract-arg-names` (arc 143) walks the output and returns the param names as a Vector<keyword>. Composition test `signature_of_fn_composes_with_bundle_children` PASSES — `Bundle/children` walks the outer Bundle and yields head + arg-pairs + arrow + ret as the children sequence. The shared output shape is enforced structurally by direct reuse of `function_to_signature_ast` (see § Reuse decisions). |
| C | Parametric type slots emit as Bundle; Path slots emit as atomic Symbol (slice 1 rules) | **YES** | Test `signature_of_fn_extracts_parametric_arg_types` PASSES — asserts `:wat::core::Vector` appears as a standalone Symbol AND `:wat::core::Vector<wat::core::i64>` (the flat pre-arc-201 spelling) does NOT. Test `signature_of_fn_extracts_monomorphic_arg_types` PASSES — asserts `:wat::core::i64` and `:wat::core::String` appear as atomic Symbols. Test `signature_of_fn_extracts_return_type_parametric` PASSES — same structural marker on the return slot. |
| D | Errors cleanly on non-fn input (TypeMismatch) | **YES** | Test `signature_of_fn_errors_on_non_fn_input` PASSES — passes an i64 literal; the error contains "signature-of-fn" (OP tag) AND "wat::core::fn" (expected-type message). |
| E | Workspace test failure count ≤ baseline (4) | **PENDING** | Full workspace `cargo test --release --no-fail-fast` running in background as of SCORE draft; will be updated with final count. Slice-1 (5/5) + slice-2 (7/7) + slice-3 (8/8) tests all pass — purely additive surface, no existing tests touched. |

**Overall:** A YES / B YES / C YES / D YES / E PENDING-baseline-check. The slice ships as designed.

## Honest deltas

### Reuse decisions (BRIEF § "Did you reuse `function_to_signature_ast` directly, or mirror its shape?")

**REUSED DIRECTLY.** The eval handler unwraps `Value::wat__core__fn(Arc<Function>)` and calls `function_to_signature_ast(&f)` verbatim, then lowers via `watast_to_holon`. No shape duplication; no parallel emission logic. The output is structurally identical to `signature-of`'s `Binding::UserFunction` branch (both call the same helper). This is the maximum-reuse path and aligns with `feedback_simple_is_uniform_composition` — the substrate already had the machinery; this slice is one new VERB that composes it.

Direct consequence: the anonymous-head convention (`:anonymous`) comes from `function_to_signature_ast`'s line ~9107 — the helper already handled the unnamed case for `Function { name: None, ... }`. No new code path for the anonymous case; the slice rides on the existing one.

### Input-shape decision (NOT in BRIEF; settled inline via four-questions)

The BRIEF describes "fn-AST" as the input but allows reuse of `function_to_signature_ast`. The implementation accepts `Value::wat__core__fn` (the fn VALUE produced by evaluating an inline `(:wat::core::fn ...)` form), NOT a raw `WatAST::List` form.

Four-questions on the input-shape choice:

- **Obvious YES** — every wat call site that has a fn produces a `Value::wat__core__fn`. Inline fn forms (`(:wat::core::fn ...)` at the call site) evaluate to a fn value via the `eval_fn` arm BEFORE the primitive dispatch sees them. Locals bound to a closure are fn values. Macro-spliced fn forms become fn values at runtime evaluation.
- **Simple YES** — one input variant; one path; one call to `function_to_signature_ast`. No WatAST walking, no synthetic Function construction, no duplicate signature builder.
- **Honest YES** — the substrate ALREADY knows the signature of every Function via the struct fields (`params`, `param_types`, `ret_type`, `rest_param`, `rest_param_type`). Walking the AST would discard that ground truth and re-parse from source — a regression into the same "string-parse the structure" anti-pattern slice 1 fixed at the type-emission layer.
- **Good UX YES** — the macro author wraps the fn form in nothing; the runtime evaluates it; signature-of-fn extracts the signature. One verb, one input, one output.

Rejected alternative: dual-mode (Value::wat__core__fn OR Value::wat__WatAST). Would have added a second code path with WatAST signature-walking duplicating what `eval_fn` + `function_to_signature_ast` already do together. The four-questions said NO (NO on simple, NO on honest — re-parsing source when the substrate has the struct is the wrong frame).

If a future consumer surfaces with a quoted fn AST in hand (e.g. macro-expansion-time inspection without evaluating the form), the right answer is either:
- Call `(:wat::core::eval-ast! quoted-form)` to produce a fn value first, OR
- Mint a separate verb like `signature-of-fn-ast` if a genuine use case emerges.

The slice does NOT speculate on either; the current input shape covers the originating consumer (arc 170 Stone D2's `run-threads` — coordinator fn is a call-site inline form that evaluates at dispatch).

### Anonymous head choice (BRIEF § "Anonymous head choice — `:anonymous` vs `:fn` vs other")

**`:anonymous`** — inherited from `function_to_signature_ast`'s existing convention (line ~9107). No new naming decision needed; the slice respects the existing precedent. If the user later prefers a different head spelling for anonymous fns, the change lives in `function_to_signature_ast` (one site) and propagates to BOTH `signature-of-fn`'s output AND any other anonymous-fn signature emitter for free.

### Variadic-rest handling (BRIEF § "Variadic-rest handling — shipped or deferred?")

**N/A — `:wat::core::fn` does not support variadic-rest binders.** Inspection of `eval_fn` (`src/runtime.rs:5060-5096`) shows `rest_param: None` and `rest_param_type: None` hardcoded for every fn value built via the inline form. `parse_fn_signature` (`src/runtime.rs:5138-5259`) only parses fixed-arity triples `name <- :T name <- :T ...`. There is no `&` rest-binder syntax in the fn form today. The BRIEF's `signature_of_fn_handles_variadic` test was DROPPED from the test file — it is not testable against the current substrate. `function_to_signature_ast` does have rest-param emission logic (lines ~9128-9145) for `:wat::core::define`'s variadic-rest case, so if `:wat::core::fn` ever gains variadic support, the existing helper will emit the `&` + rest-pair slot uniformly with no change to `signature-of-fn`.

This is NOT a STOP-trigger — the BRIEF explicitly allows shipping without variadic ("D2's coordinator is non-variadic so this can ship without variadic if blocking"). Captured here as an honest delta.

### Arc 057 / 143 surface check (BRIEF § "did you check arc 057/arc 143 surface FIRST?")

**Checked. Nothing relevant.**
- `grep` over `src/` and `tests/` for `signature-of-fn`, `signature_of_fn`, `fn-signature`, `signature-of-anon`: zero matches before this slice.
- `signature-of` (arc 143) — named-callable lookup only; rejects fn values via `name_from_keyword_or_fn` returning `None` for unnamed Function values (line 9507 returns `f.name.clone()`, which is `None` for anonymous fns). The existing primitive cannot serve the fn-value input shape.
- `lookup-define` (arc 143) — same name-keyword input contract; same gap.
- `body-of` (arc 143) — body extraction, not signature; orthogonal.
- `Bundle/children`, `Bundle/first` (arc 201 slice 2), `atom-value` (arc 057) — accessors that WALK a signature HolonAST, not a producer that BUILDS one. The composition tests (`signature_of_fn_composes_with_extract_arg_names`, `signature_of_fn_composes_with_bundle_children`) prove the new producer interoperates with these existing accessors cleanly.

No existing primitive served the fn-value-input slot. The new verb is genuinely additive.

### Naming-related `/gaze` exchanges

None ran. The BRIEF's working name `signature-of-fn` survived the inline four-questions:
- **Obvious YES** — explicit sibling of `signature-of`; `-fn` suffix names the input shape difference (name keyword vs fn value).
- **Simple YES** — short; the suffix carries the discriminator.
- **Honest YES** — the verb does exactly what its name says.
- **Good UX YES** — reader sees `signature-of`/`signature-of-fn` paired in completion lists; the contrast is immediate.

Alternatives considered briefly:
- `signature-of-anon` — narrower than the verb actually accepts (named fn values would be rejected by the suffix's implication; the verb works on ANY fn value).
- `fn-signature-of` — verb-first ordering doesn't match the existing `signature-of` precedent.
- `Fn/signature` — Cap-style method-on-type spelling; deviates from `:wat::runtime::*` namespace convention.

All rejected on the four-questions; no `/gaze` ceremony needed.

### Slice 4 nomenclature note (BRIEF § "future")

Slice 4 will rename `signature-of` → `signature-of-defn` to make the asymmetry explicit. After that, the pair reads:
- `signature-of-defn :name-keyword` — symbol-table lookup
- `signature-of-fn   :fn-value`     — fn-value introspection

Both emit the same HolonAST shape via `function_to_signature_ast`. The slice 4 BRIEF will own the back-compat-alias decision per DESIGN.md § Slice 4.

## Files touched

- `src/runtime.rs`
  - 1 new dispatch arm at line ~4047 (`:wat::runtime::signature-of-fn` → `eval_signature_of_fn`)
  - 1 new eval handler `eval_signature_of_fn` inserted between `eval_signature_of` and `eval_body_of` (~68 lines including a 43-line docstring covering API contract + reuse rationale + return-type-shape rationale + originating-consumer pointer)

- `src/check.rs`
  - 1 new `infer_list` special-case arm before `:wat::runtime::rename-callable-name` (~25 lines including docstring covering the arc-009 "names are values" bypass rationale)
  - 1 new `env.register` entry after `:wat::runtime::body-of` (~13 lines including a 7-line comment block)

- `tests/wat_arc201_signature_of_fn.rs` — NEW test file. 8 tests:
  - `signature_of_fn_emits_anonymous_head`
  - `signature_of_fn_extracts_monomorphic_arg_types`
  - `signature_of_fn_extracts_parametric_arg_types`
  - `signature_of_fn_extracts_return_type_path`
  - `signature_of_fn_extracts_return_type_parametric`
  - `signature_of_fn_composes_with_extract_arg_names`
  - `signature_of_fn_composes_with_bundle_children`
  - `signature_of_fn_errors_on_non_fn_input`

**No new types, no new structs, no new special-forms.** One new VERB. Per BRIEF § HARD constraints + `feedback_no_new_types`.

## Predicted vs actual time

Predicted 45-75 min (smaller than slice 1 per DESIGN). Actual: ~50 min including BRIEF + DESIGN reads, slice 1 + 2 spelunking for shape verification, parser/lexer check (`:Type<X,Y>` is one Keyword token), four-questions on input shape (fn-value vs WatAST), implementation, test draft, build + tests pass first try. On target.

## Knock-on / next slice

**Unblocks:**
- Slice 4 (rename `signature-of` → `signature-of-defn`) — the slice 4 BRIEF can ship now that the asymmetric pair exists.
- Slice 5 (`extract-arg-types` wat-side convenience) — can compose `signature-of-fn` + `Bundle/children` + slot-filtering for both named and inline-fn inputs uniformly.
- Arc 170 Stone D2 (`run-threads` macro) — the originating consumer; signature-of-fn provides the per-arg `:ThreadPeer<I,O>` extraction surface needed for fresh-name binding generation.
- Arc 170 Stone D3 (panic cascade) + Stone E (run-processes) — both ride on D2.

**No interference with:**
- Slice 1 (5/5 pass after slice 3) — slice 3 ADDS a sibling; slice 1's emission rules unchanged.
- Slice 2 (7/7 pass after slice 3) — slice 3's output is consumed by slice 2's accessors; the composition tests prove it.

## Discipline anchors honored

- `feedback_any_defect_catastrophic` — the missing fn-value introspection surface was the D2-blocking gap; this slice closes it.
- `feedback_no_new_types` — only one new verb. HolonAST untouched. No new structs. No new special forms.
- `feedback_simple_is_uniform_composition` — direct reuse of `function_to_signature_ast` (zero shape duplication); slice 3 is genuinely "one verb that composes existing machinery."
- `project_holon_universal_ast` — checked arc 057 + arc 143 surface for sibling primitives; none existed; new verb is additive (per arc 199/200 STOP-trigger discipline).
- `feedback_four_questions_inline` — input-shape decision (fn-value vs WatAST) ran the four-questions in prose, not via the form. Result: fn-value YES on all four; dual-mode NO on simple + NO on honest.
- `feedback_collapse_to_llm_in_loop` — N/A; pure substrate verb minting.
- `feedback_test_first` — the test file was written alongside the implementation, not after. All 8 tests pass first build.
- BRIEF § STOP triggers items 2, 3, 7 — fired and respected on the reuse + the variadic-deferral + the arc-057/143 sibling-check; all three captured as honest deltas above rather than papered over.
