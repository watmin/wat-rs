# Arc 212 stone δ-refuse-mutation — SCORE: migrate refuse_mutation_forms to children()

## Summary

`refuse_mutation_forms` in `src/freeze.rs` was migrated from List-only recursion to `ast.children()` generic recursion. The List-head mutation-form check (`if let WatAST::List(items, list_span)` → keyword head → `is_mutation_form`) is preserved verbatim. The `for child in items` recursion that was nested inside the `if let` block has been moved out and replaced with `for child in ast.children()`, extending coverage uniformly to Vector and StructPattern bracketed shapes. Pre-arc-212, mutation forms buried inside Vector RHSes or StructPattern positions would silently slip past freeze-time refusal; they are now caught. Neither named test triggered the Mode B extended-coverage scenario — no previously-silent mutation forms were found lurking in the test fixtures.

## Build

```
cargo build --release 2>&1 | tail -5
   Compiling with-lru-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
   Compiling interrogate-example v0.1.0 (...)
   Compiling with-loader-example v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 16.22s
```

Build: CLEAN.

## Verification

`cargo test --release --test probe_declaration_form_lift 2>&1 | tail -10`:

```
running 6 tests
test probe_is_declaration_form_covers_all_8_keywords ... ok
test probe_typealias_in_fn_body_do_prefix_lifts_to_prologue ... ok
test probe_newtype_in_fn_body_do_prefix_lifts_to_prologue ... ok
test probe_define_dispatch_in_fn_body_do_prefix_lifts_to_prologue ... ok
test probe_mixed_declaration_prelude_all_lift ... ok
test probe_defmacro_in_fn_body_do_prefix_lifts_to_prologue ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

PASS — 6/6 green.

`cargo test --release --test wat_eval_result 2>&1 | tail -10`:

```
test eval_edn_bang_wrong_arity_surfaces_as_err ... ok
test eval_ast_bang_mutation_form_surfaces_as_err ... ok
test try_propagates_eval_err_through_helper ... ok
test eval_edn_bang_parse_failure_surfaces_as_err ... ok
test eval_ast_bang_happy_path_returns_ok_holon ... ok
test eval_digest_string_bang_hash_mismatch_surfaces_as_err ... ok
test eval_err_exposes_both_kind_and_message ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

PASS — 7/7 green.

## Honest-delta note

None. Neither test broke. No previously-silent mutation forms in Vector positions surfaced in the test fixtures. Mode B did not fire.

## Mode classification

**Mode A** — migration applied; both named tests green; cargo build clean; SCORE written. Zero STOP triggers fired.
