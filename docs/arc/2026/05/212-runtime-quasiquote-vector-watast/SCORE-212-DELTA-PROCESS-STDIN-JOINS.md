# Arc 212 stone δ-process-stdin-joins — SCORE

## Summary

`collect_process_stdin_and_joins` in `src/check.rs` migrated from explicit
`List + Vector` match arms to `node.children()` generic recursion.

- Outer `match node { WatAST::List(...) => { ... } WatAST::Vector(...) => { ... } _ => {} }`
  collapsed to `if let WatAST::List(items, span) = node { ... }` for List-head
  classification only.
- The `:wat::core::fn | :wat::core::lambda => return` early-return preserved verbatim —
  load-bearing scope boundary that stops descent into nested fn bodies.
- Recursion moved out of the `if let` block and routed through `node.children()`,
  which covers List, Vector, and StructPattern uniformly.
- Arc 212 comment block added as specified.
- LOC delta: ~15 lines collapsed.

## Verification

```
cargo test --release --test wat_arc202_process_join_holds_stdin
test process_join_without_stdin_extraction_fails_check ... ok
test process_join_with_stdin_extraction_passes_check ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Build

```
cargo build --release
Finished `release` profile [optimized] target(s) in 16.00s
```

Clean. Zero warnings attributed to this change.

## Scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `collect_process_stdin_and_joins` uses `node.children()` for recursion | YES |
| 2 | List-head classification + fn/lambda early-return preserved verbatim | YES |
| 3 | `cargo test --release --test wat_arc202_process_join_holds_stdin` green | YES |
| 4 | `cargo build --release` clean | YES |
| 5 | SCORE file written at named path | YES |
| 6 | Zero other code edits anywhere | YES |

## Mode classification

**Mode A** — migration applied; named test green (3/3); cargo build clean; SCORE written.
