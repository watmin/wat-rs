# Arc 212 stone δ-def-restricted — SCORE

## Summary

Migrated `walk_for_def_restricted_call` in `src/check.rs` from explicit `List + Vector` match arms to `node.children()` generic recursion. List-head restriction check preserved verbatim in a leading `if let WatAST::List` guard. StructPattern coverage extended: call sites buried inside StructPattern nodes are now visited by `children()` where the old explicit-arm walker silently skipped them.

## Verification

```
cargo test --release --test wat_arc198_def_restricted
5 passed; 0 failed — test result: ok
```

## Build

```
cargo build --release
Finished `release` profile [optimized] target(s) in 16.25s — clean
```

## Scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `walk_for_def_restricted_call` uses `node.children()` for recursion | YES |
| 2 | List-head restriction check preserved verbatim | YES |
| 3 | `cargo test --release --test wat_arc198_def_restricted` green | YES |
| 4 | `cargo build --release` clean | YES |
| 5 | SCORE file written | YES |
| 6 | Zero other code edits | YES |

## Honest-delta note

No latent restriction violations surfaced via the extended StructPattern coverage. Test gate passed without revert. Expected — restriction violations live in call-head positions which only appear inside List.

## Mode classification

**Mode A** — migration applied; named test green (5/5); cargo build clean; SCORE written.
