# Arc 167 slice 4b — sweep src/ lib unit-test legacy fn-sig fixtures

## Goal

Migrate 16 legacy fn-sig sites in `src/runtime.rs` and `src/check.rs`
lib unit-test fixtures (embedded wat strings inside `#[test]`
functions) from nested-sig to flat-shape. These were slice-3
leftovers: slice 2 delta A scoped the walker to user-source forms,
so substrate-internal `mod tests` fixtures never appeared in slice
3's diagnostic stream. Slice 4's parser deletion now surfaces them
as 16 failing tests.

After this slice ships green, arc 167's workspace is fully green
and ready for slice 5 closure.

## Branch + commit policy

- **Active branch**: `arc-167-slice-2-fn-sig-consumer` (slices 2 +
  3 + 4 + 4b share this branch per atomic-merge discipline)
- Multiple WIP commits + pushes welcome
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure

## Scope: edit ONLY embedded wat strings inside `#[test]` blocks

**THIS IS NOT SUBSTRATE WORK.** The 16 sites are wat-source-code
fixtures embedded inside Rust `#[test]` functions in `src/runtime.rs`
and `src/check.rs`. The legitimate work touches ONLY the contents
of `r#" ... "#` raw-string literals inside `#[test] fn ...()` blocks.

**DO NOT modify:**
- Substrate Rust code (eval, infer, check, parsers, walkers)
- Production functions (anything outside `#[test]` blocks)
- The new `walk_for_bare_primitives` Vector arm at
  `src/check.rs:2200+`
- `parse_fn_signature` / `parse_fn_signature_for_check`
- `wat/core.wat` defn macro
- Any non-test files

If a site needs changes outside the embedded wat string, STOP and
report. The discipline is: **mechanical translation of wat code
inside test fixtures, nothing else.**

## The migration recipe (same as slice 3)

For every legacy fn-sig:

```scheme
(:wat::core::fn ((x :T) (y :T) -> :R) BODY)
```

Translate to:

```scheme
(:wat::core::fn [x <- :T y <- :T] -> :R BODY)
```

Same for defn (less common in lib unit tests; check). Zero-arg case:

```scheme
(:wat::core::fn (-> :R) BODY)
;; →
(:wat::core::fn [] -> :R BODY)
```

## Sites (per opus's slice 4 honest delta A report)

`src/check.rs` — 1 site:
- Line 13782 inside `typed_let_binding_with_fn_value`

`src/runtime.rs` — 15 sites at:
- 18776, 18789, 18804
- 20760, 20783, 20874, 20885, 20895, 20959
- 21192, 21432
- 21702, 21718, 21732, 21755

These line numbers are pre-edit; running cargo test will surface
the same failures with current line numbers (which may have
drifted slightly after deletion). Use the failing test list below
to navigate.

## Failing tests (driven by `./scripts/cargo-test-failures.sh`)

```
target: wat (lib unit tests)
  runtime::tests::arc159_new_shape_closure_capture
  runtime::tests::closure_captures_enclosing_variable
  runtime::tests::closure_captures_let_binding
  runtime::tests::concat_nested_for_more_than_two
  runtime::tests::filter_keeps_true_predicates
  runtime::tests::filter_refuses_non_bool_predicate
  runtime::tests::find_last_index_returns_none_for_empty
  runtime::tests::find_last_index_returns_rightmost_match
  runtime::tests::find_last_index_returns_none_for_no_match
  runtime::tests::foldl_vs_foldr_differ_on_nonassoc_op
  runtime::tests::foldr_is_right_associative
  runtime::tests::foldl_sums_with_init
  runtime::tests::fn_as_value
  runtime::tests::map_with_index_attaches_positions
  runtime::tests::map_doubles_every_element
  runtime::tests::values_sum_matches_map_values
```

(`check::tests::typed_let_binding_with_fn_value` may also appear
when running the lib test suite.)

## Sweep procedure

1. Run `./scripts/cargo-test-summary.sh` — note the current
   `failed: N` count (should be 16)
2. Run `./scripts/cargo-test-failures.sh` — get the failing test
   names
3. For each failing test: open the file at the test's location,
   find the embedded wat string with the legacy fn-sig, apply the
   mechanical recipe, save
4. Re-run `./scripts/cargo-test-summary.sh` — count drops as you
   go
5. Repeat until `passed: N failed: 0`

The error message from the parser will guide each fix:
> expected `(:wat::core::fn [name <- :T ...] -> :Ret body); got 2 args`

That's the parser's clear signal that this site needs the
mechanical translation.

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify substrate Rust code (only embedded wat strings
  inside `#[test]` blocks)
- DO NOT modify the canonical fn-sig parsers (`parse_fn_signature`,
  `parse_fn_signature_for_check`)
- DO NOT modify `wat/core.wat`'s defn macro
- DO NOT bridge by re-adding the legacy parser (that's the
  retired-cruft we just deleted)
- USE `./scripts/cargo-test-summary.sh` for progress measurement
- USE `./scripts/cargo-test-failures.sh` to navigate failing
  tests
- DO NOT pipe `cargo test` through `awk` — use the scripts (this
  triggers a known hallucination pattern)
- If a site doesn't fit the mechanical recipe (e.g., the test is
  testing legacy syntax intentionally; the wat string has nested
  quoting; some other quirk), STOP and report; don't bridge

## FM 5 GUARDRAIL — explicit

- If a test fails AFTER your migration of its fixture (i.e., your
  edit didn't make the test green), STOP and report
- DO NOT rewrite the test's assertions to match a different
  outcome
- DO NOT modify substrate code to "make the test work"
- The right answer is always: STOP, report what you observed, let
  orchestrator decide

## Report shape

When complete, report:
1. Final cargo test summary via `./scripts/cargo-test-summary.sh`
   (should be `passed: N failed: 0`)
2. Site count by file (calibration: `src/check.rs: 1` and
   `src/runtime.rs: N`)
3. Honest deltas — sites that didn't fit the recipe; substrate
   quirks discovered (note: per discipline, substrate quirks
   should trigger STOP-and-report, not workarounds)
4. Branch state confirmation
5. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-4B.md. If you exceed the upper bound still
iterating, STOP and report current state.
