# Arc 170 slice 3 Gap A — SCORE (keyword reflection primitives + keyword/of + Layer 2 migration)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** complete

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::core::keyword/to-string` registered + dispatched | `grep -n "keyword/to-string" src/runtime.rs src/check.rs` | PASS — dispatch arm at runtime.rs:3338; scheme at check.rs:11219 |
| B | `:wat::core::keyword/from-string` registered + dispatched | grep | PASS — dispatch arm at runtime.rs:3339; scheme at check.rs:11228 |
| C | `keyword/to-string` returns text WITHOUT leading colon | unit test `keyword_to_string_strips_leading_colon` | PASS — `(:keyword/to-string :foo)` → `"foo"` (not `":foo"`) |
| D | Round-trip `(from-string (to-string k)) = k` works for ≥ 3 sample keywords | unit test `keyword_reflection_round_trip` | PASS — 3 cases: `:foo`, `:wat::core::i64`, `:wat::kernel::Receiver<wat::core::i64>` |
| E | `:wat::core::keyword/of` macro special-form recognized in `expand_form` | `grep -n "keyword/of" src/macros.rs` | PASS — check at macros.rs:537-539; construct_keyword_of at macros.rs:578 |
| F | `keyword/of` constructs correct multi-arg parametric text (commas, args sans-colon) | unit test `keyword_of_multi_arg_comma_separated` | PASS — `(:keyword/of :wat::core::Result :wat::core::i64 :wat::core::String)` → `:wat::core::Result<wat::core::i64,wat::core::String>` |
| G | Layer 2 macro in `wat/test.wat` uses `keyword/of` for channel types | `grep -n "keyword/of" wat/test.wat` | PASS — lines 767-768 in `run-hermetic-with-io` body |
| H | T18 + T18b updated to pass inner element types; both still pass | `cargo test --release --test wat_arc170_program_contracts t18` → 2 passed 0 failed | PASS |
| I | Workspace stays at 0 failed (2184 → 2193) | `cargo test --release --workspace --no-fail-fast` | PASS — 2193 passed / 0 failed (+9 new tests) |
| J | `cargo check --release` green | clean | PASS |

**All 10 rows pass.**

## Implementation locations

### Phase 1 — Runtime primitives

**`src/runtime.rs`**
- Dispatch arms at line 3338-3339 (inside `dispatch_keyword_head`, after `:wat::core::string::to-bool`)
- `eval_keyword_to_string` function at line 5530 (after `eval_string_to_bool`)
- `eval_keyword_from_string` function at line 5560 (after `eval_keyword_to_string`)
- Unit tests at lines 24789-24860: `keyword_to_string_strips_leading_colon`, `keyword_from_string_prepends_colon`, `keyword_reflection_round_trip`, `keyword_from_string_rejects_colon_prefix`

**`src/check.rs`**
- Two `env.register` calls at lines 11219-11235, after `:wat::core::string::to-bool` scheme registration
- Local `keyword_ty` closure defined inline at line 11217 (mirrors the one at line 11580 in a later scope)

### Phase 2 — Macro special-form

**`src/macros.rs`**
- `keyword/of` dispatch in `expand_form` at lines 520-541 (after child recursion, before macro check)
- `construct_keyword_of` function at lines 577-671 (between `expand_form` and `expand_macro_call`)
- Unit tests at lines 1966-2047: `keyword_of_single_arg`, `keyword_of_multi_arg_comma_separated`, `keyword_of_inside_macro_template_with_unquote`, `keyword_of_arity_error_no_args`, `keyword_of_non_keyword_child_error`

### Phase 3 — Layer 2 migration

**`wat/test.wat`**
- `run-hermetic-with-io` defmacro (lines 756-771) — parameters renamed from `(rx-type, tx-type)` to `(input-type, output-type)`; body uses `(:wat::core::keyword/of :wat::kernel::Receiver ~input-type)` and `(:wat::core::keyword/of :wat::kernel::Sender ~output-type)` to construct channel types

**`tests/wat_arc170_program_contracts.rs`**
- T18 and T18b updated — call sites pass `:wat::core::i64` (inner element type) instead of `:wat::kernel::Receiver<wat::core::i64>` / `:wat::kernel::Sender<wat::core::i64>`

## Honest deltas

### Delta 1: Module location — runtime.rs owns the primitives directly

The BRIEF anticipated needing to check "wherever keyword primitives currently live." Grep confirmed there are no existing keyword/string primitives — this is the first pair. They landed in `src/runtime.rs` inline with the other scalar conversion primitives (`i64::to-string`, `f64::to-string`, `bool::to-string`) following the established Type/verb naming convention. The dispatch table slot in `dispatch_keyword_head` is the natural home: every existing primitive lives there. No new file or module was created.

### Delta 2: Expansion order — quasiquote fires BEFORE expand_form sees keyword/of

The BRIEF raised the question of whether `keyword/of` could see substituted unquotes. The order is:

1. `expand_form` is called on the `run-hermetic-with-io` macro call.
2. Children are recursed — at this point the macro call `(:wat::test::run-hermetic-with-io :wat::core::i64 :wat::core::i64 inputs body)` has fully expanded children (all are already concrete AST nodes).
3. The head `:wat::test::run-hermetic-with-io` matches a registered macro.
4. `expand_macro_call` → `expand_template` → `walk_template` fires, substituting `~input-type` with `:wat::core::i64` and `~output-type` with `:wat::core::i64`.
5. The result is a WatAST list containing `(:wat::core::keyword/of :wat::kernel::Receiver :wat::core::i64)` as a concrete node.
6. `expand_form` is called again on this result (fixpoint loop at macros.rs:531).
7. In this re-expansion pass, children are recursed (they are all already concrete keywords — no further substitution needed).
8. The `keyword/of` dispatch arm fires on the fully-substituted children.

**The architectural order is correct and composes cleanly.** No ordering blocker was encountered. The constraint is: `keyword/of` must live in `expand_form` (not `walk_template`), because `walk_template` runs INSIDE the quasiquote's substitution pass, where children have not yet been recursed by `expand_form`. The current placement — in `expand_form` after child recursion — is exactly right.

### Delta 3: `keyword_ty` closure defined twice in check.rs

The `keyword_ty` closure used for the new scheme registrations (line 11217) is a local definition at the insertion point. A second `keyword_ty` closure already exists at line 11580 in a later sub-scope of the same `register_builtins` function. Rust permits two closures with the same name in non-overlapping scopes; both compile cleanly. A single top-level definition would be cleaner, but the existing pattern (each sub-section defines the types it needs locally) makes a one-off local the natural choice per established codebase discipline.

### Delta 4: `construct_keyword_of` uses an inline `ast_kind` helper

The BRIEF suggested `other.type_tag()` for error messages. `WatAST` has no `type_tag` method — the closest equivalent is `variant_name` in `src/load.rs:785` (private to that module). Rather than expose a public method or import from load.rs, `construct_keyword_of` defines a local inline helper `ast_kind`. The logic is identical to `load::variant_name` (9-arm match, same labels). This is the correct scope: a private helper for one error-formatting site, not a candidate for cross-module exposure at this stage.

### Delta 5: 9 new tests, not N (N was unspecified)

The BRIEF left N open. The final count is 9:
- 4 runtime unit tests (to-string, from-string, round-trip, error-case)
- 5 macro unit tests (single-arg, multi-arg, unquote-composition, arity-error, non-keyword-error)

T18 and T18b remain at 2 passing tests (they migrated to the simplified surface, not new tests).

## Files modified

| File | Change |
|------|--------|
| `src/runtime.rs` | Added `eval_keyword_to_string` + `eval_keyword_from_string` functions; dispatch arms in `dispatch_keyword_head`; 4 unit tests. No existing code modified. |
| `src/check.rs` | Added 2 `env.register` calls for `keyword/to-string` and `keyword/from-string` schemes in `register_builtins`. No existing registration modified. |
| `src/macros.rs` | Added `keyword/of` dispatch arm in `expand_form`; added `construct_keyword_of` function; added 5 unit tests. No existing expansion logic modified. |
| `wat/test.wat` | Updated `run-hermetic-with-io` defmacro: parameters renamed `(rx-type, tx-type)` → `(input-type, output-type)`; body uses `keyword/of` to construct channel types. No other form in the file modified. |
| `tests/wat_arc170_program_contracts.rs` | Updated T18 and T18b to pass inner element types (`:wat::core::i64`); updated comments. |

## What's next — Gap B

Gap B: `Sender/close` for explicit EOF signaling on the channel write side. This was documented in Phase D Delta 3 and D3 honest delta: the driver cannot close the parent's tx to signal EOF to the child's rx. For child patterns that loop until `Ok(None)`, the parent's tx must be dropped (closed) before the child detects EOF. Without a `Sender/close` verb, the only safe pattern is a bounded-I/O child (T18 pattern: read exactly N, send exactly M, exit). Gap B adds this verb to unlock unbounded streaming patterns.
