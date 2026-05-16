# Arc 201 Slice 1 — SCORE

**Branch baseline:** `90bb496` (workspace: 4 stable failures + lifeline flake).

## Rows

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `type_expr_to_ast` (chosen name) replaces `type_expr_to_kw`; emits Bundle for Parametric/Tuple/Fn, Atom (Keyword) for Path/Var | YES | `src/runtime.rs:9049-9088` — recursive `type_expr_to_ast` with all five `TypeExpr` arms. Old `type_expr_to_kw` symbol fully removed (only mentioned in docstring as historical reference). |
| B | All 5 signature-AST builders use the new emission | YES | `grep "type_expr_to_kw"` → 1 hit (a docstring reference, not a call). Call-site updates: `function_to_signature_ast` (params + rest + ret, 3 sites), `type_scheme_to_signature_ast` (params + ret, 2 sites), `dispatch_to_signature_ast` (ret slot via direct `type_expr_to_ast` call replacing the `type_expr_to_keyword` wrapper). `typedef_to_signature_ast` and `macrodef_to_signature_ast` carry no per-arg `TypeExpr` (typedef heads are name+type-params strings; defmacro params have no per-param type slot — they always emit the `:AST<wat::WatAST>` sentinel keyword), so they need no recursion entry-point — but they were INSPECTED to confirm slice 1's scope. |
| C | Existing consumers (extract-arg-names, rename-callable-name, define-alias) still work | YES | `wat_arc143_lookup` 11/11 pass, `wat_arc143_define_alias` 3/3 pass, `wat_arc144_uniform_reflection` 9/9 pass, `wat_arc144_special_forms` 9/9 pass. New test `define_alias_round_trips_on_parametric_signature` exercises the splice round-trip on `:wat::core::foldl` (Parametric + Fn types in the spliced head). |
| D | New unit test asserts structured emission for parametric + Tuple + Fn shapes | YES | `tests/wat_arc201_structured_signature_types.rs` — 5/5 pass: `signature_of_emits_structured_parametric_user_fn`, `signature_of_emits_atomic_for_monomorphic_path_types`, `signature_of_foldl_emits_structured_parametric_and_fn` (covers Parametric + Fn + Var + Path together), `signature_of_emits_structured_tuple_return_type`, `define_alias_round_trips_on_parametric_signature`. |
| E | Workspace test failure count ≤ baseline | YES | Post-slice workspace failures (intermittent): `lifeline_pipe_zero_orphans_across_100_trials` (the baseline flake), `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`, `deftest_wat_rs_test_test_ambient_stdio_println_string` (intermittent). All five are pre-existing — none reference `type_expr_to_*`, `signature-of`, or the reflection layer. Net new failures from this slice: **0**. Net new passes: **5** (the arc 201 test file) + the previously-broken-by-emission-change `signature_of_variadic_define_returns_rest_shape` is now updated to assert the structured spelling and passes. |

**All five rows: YES.**

## Honest deltas

### Naming

- **`type_expr_to_ast`** — chosen for symmetry with the sibling builders (all named `*_to_*_ast`) and to mark the shape distinction from the retired `type_expr_to_kw` (Keyword) → new produces general AST (Keyword or List). No `/gaze` round needed; the four-questions converged on first pass.
- **`parse_type_slot`** — new consumer-sweep helper at `src/runtime.rs:3303`. Pairs with `parse_type_keyword` (legacy keyword-only path) and accepts EITHER `WatAST::Keyword` OR `WatAST::List` (the structured form). The name was picked over `parse_structured_type` because the caller's question is "what type goes in this slot?" not "is this structured or flat?" — the helper internally dispatches. Verbose-is-honest: the name describes the call-site role.

### `TypeExpr` variants

Confirmed five variants exist in `src/types.rs:45-78` (Path, Parametric, Fn, Tuple, Var). All five enumerated in DESIGN.md. **No surprise variants** — no `Concrete`, no hidden refinement type. `type_expr_to_ast` matches exhaustively.

### Consumer-sweep finding — the round-trip gap

The DESIGN said `extract-arg-names` and `rename-callable-name` only read pair[0] / the head Symbol, so consumers were "BACKWARD-COMPATIBLE for consumers that only read names." This was true for those two functions in isolation but **MISSED ONE LAYER DOWN**: `:wat::runtime::define-alias` SPLICES the entire signature head back into a fresh `(:wat::core::define ...)`. After arc 201's structured emission, the spliced head carries Bundle type-slots — which the `:wat::core::define` parser refused (`parameter type must be a type keyword; got list`). This broke the substrate boot path because `wat/list.wat:16-17` uses `define-alias` on `:wat::core::foldl` at boot.

**Fix:** extended `parse_param_pair` and the ret-type slot in `parse_define_signature` to route through the new `parse_type_slot` helper, which accepts both the legacy Keyword form AND the structured List form. The substrate now ROUND-TRIPS structured signatures through the define parser. This is a slice-1 consumer fix (per BRIEF "If a consumer DOES break, fix it inline"), NOT new substrate — it widens the INPUT domain of an existing parser to accept the new emission shape it must now consume.

The fix is symmetric in spirit: the substrate emits structured at the signature layer, accepts structured back at the parse layer. No back-compat shim was needed; both shapes flow to the same `TypeExpr`.

### `format_type` boundary

`format_type` is UNCHANGED — it still emits flat keyword spellings for diagnostics and error messages (its docstring's stated purpose). Only SIGNATURE emission paths got the structured form. Verified by grep: the four remaining `format_type` callers are:

- `src/check.rs` — many diagnostic / error-message uses
- `src/runtime.rs:9335` — error string in `is_vector_type` rejection (diagnostic)
- `src/runtime.rs:9304` — `type_expr_to_keyword` thin wrapper (still used by `dispatch_to_define_ast`'s per-arm pattern emission, which is the ARM declaration round-trip surface, not the signature surface — kept flat per DESIGN's explicit decision)
- Various other diagnostic surfaces unchanged

The asymmetry is intentional: `dispatch_to_define_ast` mirrors the user's source spelling for the arms (a `define-dispatch` form), where flat keyword spellings ARE the source form. `dispatch_to_signature_ast` (which builds the CALLABLE-shaped signature head for alias / reflection consumers) now uses structured emission for its ret-type slot.

### Updated arc-150 test

`tests/wat_arc150_variadic_define.rs:241-249` asserted the legacy flat spellings `Vec<i64>` / `Vector<i64>` / `Vector<wat::core::i64>` for the variadic rest-binder's type slot. Post-arc-201, those flat spellings no longer appear — the type is emitted as a structured Bundle. Test updated to assert the structured-emission witnesses (`:wat::core::Vector` head + `:wat::core::i64` arg appear as separate Symbol tokens). Test still passes the same intent (round-trip the variadic shape) against the new shape.

This was the SINGLE consumer-side test update needed; no other existing test asserted flat-spelling structure inside signature output.

## Surprises observed

- **The boot path is a consumer.** I expected the `define-alias` test to be the only round-trip witness; in fact `wat/list.wat` calls `define-alias :wat::list::reduce :wat::core::foldl` at SUBSTRATE BOOT, so every single test was failing at startup. The structured emission failure mode was load-bearing: ANY test running through `startup_from_source` was breaking. This made the fix mandatory rather than nice-to-have — substrate boot is the strictest consumer.

- **EDN double-quoting in `println`.** `:wat::kernel::println` EDN-renders its argument, even when the argument is already a String. So an EDN-rendered HolonAST String containing `Symbol ":T"` gets re-encoded as a quoted EDN string `"...Symbol \":T\"..."` with literal backslash-escaped inner quotes. Test assertions needed to anchor on `\":T\"` (literal backslash) not `":T"`. Worth noting for future reflection tests; pre-existing arc-143 tests sidestep this by checking unquoted substrings only.

- **`type_expr_to_keyword` is alive.** Different from `type_expr_to_kw`. `type_expr_to_keyword` (returns String) is a thin wrapper over `format_type`; it's still used by `dispatch_to_define_ast` for arms-as-data emission and by the dispatch's `:T` fallback. I left it untouched per BRIEF (only SIGNATURE emission, not arm declaration).

- **`typedef_to_signature_ast` and `macrodef_to_signature_ast` have no TypeExpr recursion entry.** Both build their head purely from name + type-param strings (typedef) or emit a fixed `:AST<wat::WatAST>` sentinel keyword (macrodef). Neither needed the new helper. Inspected and confirmed in-scope — DESIGN mentioned them but they have no `TypeExpr` to recurse into.

## Files touched (slice 1 only)

- `src/runtime.rs` — `type_expr_to_kw` → `type_expr_to_ast` rewrite (recursive); new `parse_type_slot`; `parse_param_pair` + `parse_define_signature` ret-type slot wired to `parse_type_slot`; `dispatch_to_signature_ast` ret slot upgraded.
- `tests/wat_arc201_structured_signature_types.rs` — NEW, 5 tests.
- `tests/wat_arc150_variadic_define.rs:241-265` — updated `signature_of_variadic_define_returns_rest_shape` to assert structured emission witnesses (the single consumer-side test update).

## Time

~75 min of clock time including consumer-sweep diagnosis. Within 60-90 min predicted, well under 120 min hard stop.
