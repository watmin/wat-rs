# Arc 212 stone δ-bare-primitives — SCORE: migrate walk_for_bare_primitives to children()

## Summary

The recursion shape of `walk_for_bare_primitives` in `src/check.rs` was migrated from an explicit `match` with duplicated `List` and `Vector` arms to an `if let WatAST::Keyword` guard followed by a single `node.children()` generic recursion loop. The Keyword arm body — all four legacy-keyword checks (`let*`, `lambda`, `unit`, `:fn(`) plus the `parse_type_expr_audit` call — was preserved verbatim including every early `return`. A final `return;` was added at the end of the `if let` block so Keyword nodes do not fall through to the children() loop. The `List`, `Vector`, and `_ => {}` arms were removed entirely and replaced by the four-line `node.children()` loop with an arc-212 annotation.

## Verification

```
cargo test --release --test wat_arc154_kill_let_star 2>&1 | tail -5
  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

cargo test --release --test wat_arc153_nil_rename 2>&1 | tail -5
  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --test wat_arc155_fn_rename 2>&1 | tail -5
  test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Build

```
cargo build --release 2>&1 | tail -5
  Finished `release` profile [optimized] target(s) in 16.13s
```

Compile clean.

## Mode classification

**Mode A** — migration applied; three named tests green; cargo build clean; SCORE written.
