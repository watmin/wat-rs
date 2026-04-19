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

Phase 1 MVP shipped. The interpret path's tail — parse → lower → encode —
works end-to-end for algebra-core source. Everything above the algebra
(language forms, macros, types, config, load!, freeze, runtime) is
pending.

**Landed:**

- [x] `lexer` — s-expression tokenizer with paren-depth tracking per
      the corrected FOUNDATION tokenizer rule; keywords, string /
      numeric / bool literals, symbols, line comments, internal-colon
      rejection. 25 tests.
- [x] `ast::WatAST` — language-surface AST, 7 variants (IntLit,
      FloatLit, BoolLit, StringLit, Keyword, Symbol, List).
- [x] `parser` — recursive descent over tokens. `parse_one` / `parse_all`
      entry points. Structured error types. 22 tests.
- [x] `lower` — `WatAST` → `holon::HolonAST` for the 6-form algebra
      core: Atom (any Rust primitive + keyword), Bind, Bundle (with
      `:wat/core/list` form), Permute (i32 step), Thermometer, Blend
      (Option B). 16 tests.
- [x] `eval_algebra_source` — the "door works" public function. Source
      text → `holon::Vector` in one call. 10 integration tests.

**Pending (ordered per FOUNDATION's startup pipeline):**

- [ ] Entry-file discipline + config pass
      (`(:wat/config/set-dims!)`, `(:wat/config/set-capacity-mode!)`,
      `(:wat/config/set-global-seed!)`; all setters before any `load!`).
- [ ] Recursive `:wat/core/load!` resolution.
- [ ] `:wat/core/define` / `:wat/core/lambda` / `:wat/core/let` /
      `:wat/core/if` + a basic AST-walker runtime.
- [ ] `:wat/core/defmacro` + Racket sets-of-scopes hygiene.
- [ ] Type declarations (`struct`, `enum`, `newtype`, `typealias`) +
      type environment.
- [ ] Name resolution across the frozen symbol table.
- [ ] Rank-1 Hindley-Milner type checker.
- [ ] Canonical-EDN hashing + cryptographic verification (`md5`,
      `signed` load modes).
- [ ] Freeze (symbol table, type env, macro registry, config).
- [ ] Runtime + `:user/main` + constrained `eval`.
- [ ] `wat-vm` CLI binary.

The measurements tier (`:wat/algebra/cosine`, `:wat/algebra/dot`
returning `:f64`) lands with the runtime slice — measurements don't go
through `eval_algebra_source`, which only returns `Vector`. A unified
value-dispatch layer lives in the runtime.

Authoritative backlog: `holon-lab-trading/docs/058-backlog.md`.

## See also

- `holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this crate
  evaluates against.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md` —
  the language specification.
- `holon-lab-trading/docs/058-backlog.md` — the implementation arc.
