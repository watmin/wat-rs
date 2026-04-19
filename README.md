# wat-rs

The Rust implementation of the wat language — parser, type checker, macro
expander, and runtime for the s-expression surface defined in the 058
algebra-surface proposal batch (`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`).

## What wat-rs is

`wat-rs` consumes wat source and produces vectors via holon-rs. Two paths:

- **Interpret path (this crate's first job).** Parse → resolve → type-check
  → freeze → runtime AST walker. The runtime dispatches algebra-core
  UpperCalls (`:wat/algebra/Atom`, `:wat/algebra/Bind`, …) to
  `holon::HolonAST` and encodes via `holon::encode`.
- **Compile path (later).** Parse → resolve → type-check → emit Rust source
  that `rustc` compiles to a native binary. See `WAT-TO-RUST.md` in the 058
  batch for the seed design.

Both paths share the frontend. Only the tail differs.

## Dependency stack

```
holon-rs  (algebra substrate — 6 core forms, encode, registry)
    ↑
wat-rs    (this crate — wat frontend + interpret/compile runtime)
    ↑
holon-lab-trading / holon-lab-ddos / any wat-consuming application
```

Applications talk to wat-rs to load and run their wat programs; wat-rs
talks to holon-rs for the algebra primitives.

## Status

Initial commit — crate skeleton. Parser, lexer, and WatAST → HolonAST
lowering in progress per Phase 1 of the 058 implementation backlog.

## See also

- `holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this crate
  evaluates against.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md` —
  the language specification.
- `holon-lab-trading/docs/058-backlog.md` — the implementation arc.
