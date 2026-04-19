# wat-rs

The Rust implementation of the wat language — parser, type checker, macro
expander, and runtime for the s-expression surface defined in the 058
algebra-surface proposal batch (`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`).

## What wat-rs is

`wat-rs` consumes wat source and produces vectors via holon-rs. Two paths:

- **Interpret path (this crate's first job).** Parse → resolve → type-check
  → freeze → runtime AST walker. The runtime dispatches algebra-core
  UpperCalls (`:wat::algebra::Atom`, `:wat::algebra::Bind`, …) to
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
      `:wat::core::vec` form), Permute (i32 step), Thermometer, Blend
      (Option B). 16 tests.
- [x] `eval_algebra_source` — the "door works" public function. Source
      text → `holon::Vector` in one call. 10 integration tests.

**Landed (continued):**

- [x] Entry-file discipline + config pass (`set-dims!`,
      `set-capacity-mode!`, `set-global-seed!`; all setters before any
      `load!`; setter in a loaded file halts parse).
- [x] Recursive `:wat::core::load!` resolution — depth-first,
      commit-once, cycle detection, in-memory and filesystem loaders.
- [x] `:wat::core::define` / `:wat::core::lambda` / `:wat::core::let` /
      `:wat::core::if` + basic AST-walker runtime with algebra-core
      dispatch.
- [x] `:wat::core::defmacro` + Racket sets-of-scopes hygiene (Flatt 2016).
- [x] Type declarations (`struct`, `enum`, `newtype`, `typealias`) +
      type environment; parametric names (`:my::Container<T>`).
- [x] Name resolution across the frozen symbol table — reserved prefix
      gate (`:wat::core::`, `:wat::kernel::`, `:wat::algebra::`, `:wat::std::`,
      `:wat::config::`).
- [x] Slice 7b rank-1 Hindley-Milner type check — parametric
      polymorphism (list: `∀T. T*→List<T>`; comparison: `∀T. T→T→bool`;
      Atom: `∀T. T→Holon`), substitution + occurs-check, user-define
      body-vs-signature checks with rigid type variables. `:Any`
      banned at parse time.
- [x] Canonical-EDN hashing + SHA-256 source-file integrity.
- [x] Ed25519 signed-load verification — per-file and full-program;
      signs SHA-256 of canonical-EDN.
- [x] Load-form grammar redesign — three sibling forms
      (`:wat::core::load!`, `:wat::core::digest-load!`,
      `:wat::core::signed-load!`) using `:wat::load::*` source-interface
      keywords and `:wat::verify::*` payload-interface + algorithm
      keywords. Sidecar signature files work via `:wat::verify::file-path`.

**Pending (ordered per FOUNDATION's startup pipeline):**

- [ ] Freeze (symbol table, type env, macro registry, config).
- [ ] Runtime + `:user/main` + constrained `eval`.
- [ ] `wat-vm` CLI binary (incl. full-program signature verification
      via `--signed <algo> --sig <b64> --pubkey <b64>` or sidecar).

The measurements tier (`:wat::algebra::cosine`, `:wat::algebra::dot`
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
