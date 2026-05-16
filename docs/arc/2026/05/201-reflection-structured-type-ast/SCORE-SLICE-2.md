# SCORE — Arc 201 Slice 2 — General-purpose HolonAST accessors

**Slice:** 2 of 6 (see DESIGN.md § Stepping stones).
**Date:** 2026-05-16.
**Predecessor:** slice 1 (commit `0706949`) — structured type-AST emission.

## SCORE rows

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `:wat::holon::Bundle/children` minted with type scheme + eval handler + dispatch arm | **YES** | `src/runtime.rs` dispatch arm + `eval_bundle_children` definition; `src/check.rs` `infer_list` special-case + `env.register` call. Tests `bundle_children_returns_vec_of_holonast_from_signature`, `bundle_children_walks_parametric_type_slot` PASS. |
| B | `:wat::holon::Bundle/first` minted similarly | **YES** | `src/runtime.rs` dispatch arm + `eval_bundle_first`; `src/check.rs` `infer_list` special-case + `env.register`. Tests `bundle_first_returns_head_keyword_of_signature`, `bundle_first_composes_with_atom_value` PASS. |
| C | `:wat::holon::Atom/value` minted similarly | **NO (DELIBERATE — STOP trigger 3 fired)** | Accessor-shaped sibling `:wat::core::atom-value` already exists (arc 057, `src/runtime.rs:11756` — `eval_atom_value`). It unwraps `HolonAST::Atom` AND extracts the wat-`Value` for primitive leaves (Symbol/String/I64/F64/Bool). Minting `Atom/value` would duplicate this. Per BRIEF § STOP triggers item 3 ("don't duplicate; reuse if appropriate"), the slice surfaces the existing accessor and composes with it. Test `bundle_first_composes_with_atom_value` proves Bundle/first + atom-value interoperate cleanly. |
| D | All minted accessors error cleanly on wrong input shape (TypeMismatch) | **YES** | Tests `bundle_children_errors_on_atom_input`, `bundle_first_errors_on_leaf_input`, `bundle_first_errors_on_empty_bundle` PASS — each asserts the OP-tagged TypeMismatch message and discriminating substring ("non-Bundle", "empty Bundle"). |
| E | Workspace test failure count ≤ baseline (4) | **YES** | `cargo test --release --workspace --no-fail-fast` produces exactly 4 failed individual tests across 4 binaries — matches commit `0706949` baseline. Pass count post-slice: 2098 tests pass + 7 new accessor tests in `wat_arc201_holon_ast_accessors` (folded into the 2098 total). |

**Overall:** A YES / B YES / C NO-deliberate / D YES / E YES. The "NO" on row C is a positive finding — the substrate already had the verb the BRIEF anticipated, and surfacing it is the right move (per `feedback_no_known_defect_left_unfixed` inverted: don't add a known DUPLICATE).

## Honest deltas

### Final naming chosen

- `:wat::holon::Bundle/children` (proposed `Bundle/children` — kept; the HolonAST source comment on `Bundle(Arc<Vec<HolonAST>>)` describes them as "children" so the verb echoes the docstring vocabulary)
- `:wat::holon::Bundle/first` (proposed `Bundle/head` — **changed to `Bundle/first`** to mirror `:wat::core::first` precedent in the wat conventions; user-precedent search found 20+ existing call sites of `:wat::core::first` in `src/check.rs` and tests, zero existing `:wat::core::head`. The four-questions YES YES YES YES landed on `first` over `head` cleanly inline — no `/gaze` ceremony needed since the wat convention is unambiguous.)
- **No `Atom/value` minted.** Per § Sibling check below.

### HolonAST::Atom payload shape

The `HolonAST::Atom` variant wraps `Arc<HolonAST>` — it is the OPAQUE-IDENTITY wrap (per `holon-rs/src/kernel/holon_ast.rs:84` and arc 057 doctrine). It is NOT the "atomic value" variant a fresh reader might assume from the name.

The atomic LEAF variants are `Symbol(Arc<str>)`, `String(Arc<str>)`, `I64(i64)`, `F64(f64)`, `Bool(bool)`. Slice 1's structured type-AST emission produces these as the leaves of each Bundle (e.g., `HolonAST::Symbol(":wat::core::i64")` for a parametric type's argument slot, lowered from `WatAST::Keyword`).

The existing `:wat::core::atom-value` handles BOTH the opaque-wrap unwrap AND the leaf-Value extraction in one verb (see `runtime.rs:11772-11812`):
- `HolonAST::Atom(inner)` → `Value::holon__HolonAST(inner)` (unwraps one layer)
- `HolonAST::Symbol(s)` → `Value::keyword(s)`
- `HolonAST::String(s)` → `Value::String(s)`
- `HolonAST::I64(n)` → `Value::i64(n)`
- `HolonAST::F64(x)` → `Value::f64(x)`
- `HolonAST::Bool(b)` → `Value::bool(b)`
- Composite variants → TypeMismatch (structural decomposition goes through `Bundle/children` etc., not `atom-value`)

This makes `:wat::core::atom-value` the canonical leaf-side accessor; `Bundle/children` and `Bundle/first` are the canonical sequence-side accessors. Together they cover the full HolonAST decomposition surface for the reflection use case.

### Sibling check — did an accessor already exist?

**YES, for the leaf case.** `:wat::core::atom-value` (arc 057, `runtime.rs:11756-11812`) already handles every shape a hypothetical `Atom/value` would have handled. Per BRIEF § STOP triggers item 3, I surfaced the duplicate and DID NOT mint `Atom/value`. The test `bundle_first_composes_with_atom_value` documents the composition pattern: `Bundle/first` returns a HolonAST, `atom-value` extracts its wat-`Value`.

**NO, for the Bundle case.** `grep` over `src/runtime.rs` and `src/check.rs` for `Bundle/children`, `Bundle/head`, `Bundle/first`, `Bundle/items`, `bundle-children`, `bundle-head`, `bundle-first` returns empty. The only sibling-shaped primitive (`require_bundle` in `runtime.rs:9923`) is a Rust-internal helper used by `eval_extract_arg_names` and `eval_rename_callable_name`, not a wat-surface accessor.

### `/gaze` exchanges

None ran formally — the inline four-questions resolved naming cleanly:

- `Bundle/first` vs `Bundle/head`: `first` mirrors `:wat::core::first` (precedent: 20+ existing call sites). YES YES YES YES on `first`; `head` would deviate from the convention without a reason. Disqualified.
- `Bundle/children` vs `Bundle/items` vs `Bundle/parts`: `children` matches the HolonAST docstring vocabulary verbatim. YES YES YES YES on `children`. Disqualified `items` (vocabulary drift) and `parts` (less specific).

### Workspace baseline preserved?

**YES exactly.**

Baseline (commit `0706949`): 4 stable failures + lifeline flake variance:
- `probe_lifeline_pipe_proof::lifeline_pipe_zero_orphans_across_100_trials` (flake)
- `test::deftest_wat_tests_tmp_totally_bogus` (stable)
- `wat_arc170_program_contracts::t6_spawn_process_factory_with_capture_round_trips` (stable)
- `wat-cli/tests/wat_cli::startup_error_bubbles_up_as_exit_3` (stable)

Post-slice-2: SAME 4 failures, no new ones, no regressions in slice-1's `wat_arc201_structured_signature_types` (5/5 pass).

Total individual test passes across the workspace: 2098. Net add: +7 (new accessor tests; all pass).

## Files touched

- `src/runtime.rs` — 2 dispatch arms (line ~4051), 2 eval handlers (`eval_bundle_children`, `eval_bundle_first` — ~100 lines including docstrings, inserted after `eval_extract_arg_names`).
- `src/check.rs` — 2 `infer_list` special-case arms (line ~4798), 2 `env.register` entries (line ~14172).
- `tests/wat_arc201_holon_ast_accessors.rs` — NEW test file, 7 tests.

No new types, no new structs, no new special-forms. Only new VERBS on existing `HolonAST` (per BRIEF discipline anchor `feedback_no_new_types`).

## Predicted vs actual time

Predicted 30-60 min. Actual ~45 min including investigation, four-questions, draft, build, three rounds of test-file syntax fixups (match arrow, None pattern, Bundle returns Result). On target.

## Knock-on / next slice

Slice 2 unblocks:
- Slice 3 (`signature-of-fn` for inline fn-AST inputs) — its tests can use these accessors to walk the structured signature it returns.
- Slice 5 (`extract-arg-types` wat-side convenience) — it will compose `Bundle/children` + filtering, the exact pattern Q2 (β) anticipated.

Arc 170 Stone D2's run-threads macro can now walk the structured type-AST end-to-end:
`signature-of` → outer Bundle → `Bundle/children` → arg-pair Bundles → `Bundle/first` for arg names (or `extract-arg-names`) → recurse `Bundle/children` for parametric type slots → `atom-value` for leaf keyword extraction.

## Discipline anchors honored

- `feedback_any_defect_catastrophic` — the reflection-walker gap was the originating crack; this slice closes it.
- `feedback_no_new_types` — only new verbs; HolonAST untouched.
- `feedback_simple_is_uniform_composition` — `Bundle/children` + `Bundle/first` + `atom-value` are uniformly composable for any Bundle shape, not signature-specific.
- `project_holon_universal_ast` — accessors operate on HolonAST as the universal structured-semantic AST; they serve any consumer, not just reflection.
- `feedback_collapse_to_llm_in_loop` (anti-pattern check) — N/A here; the slice is pure-substrate verb minting.
- BRIEF § STOP triggers item 3 — fired and respected on `Atom/value`; surfaced rather than duplicated.
