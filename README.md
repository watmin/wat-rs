# wat

The Rust implementation of the wat language — parser, type checker, macro
expander, and runtime for the s-expression surface specified in the 058
algebra-surface proposal batch (`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`).

Ships:

- `wat` — the library (this crate).
- `wat-vm` — the CLI runner. `wat-vm <entry.wat>` reads the file, runs the
  full startup pipeline, invokes `:user::main` with real stdio channels,
  and exits. See [the contract](#user-main-contract) below.

## What wat is

`wat` consumes wat source and produces vectors (via `holon`) and runtime
effects (via the kernel primitives the wat-vm binary wires up).

Two intended paths:

- **Interpret path (shipped).** Parse → load-resolve → macro-expand →
  register types → register defines → resolve → type-check → freeze →
  invoke `:user::main`. The runtime dispatches algebra-core UpperCalls
  (`:wat::algebra::Atom`, `:wat::algebra::Bind`, …) to `holon::HolonAST`
  and encodes via `holon::encode`.
- **Compile path (later).** Parse → resolve → type-check → emit Rust
  source that `rustc` compiles to a native binary. Shares the frontend
  with the interpret path; only the tail differs. Seed design in
  `WAT-TO-RUST.md` in the 058 batch.

## Dependency stack

```
holon   (algebra substrate — 6 core forms, encode, registry)
    ↑
wat     (this crate — wat frontend + interpret/compile runtime)
    ↑
holon-lab-trading / holon-lab-ddos / any wat-consuming application
```

Applications talk to `wat` to load and run their wat programs; `wat` talks
to `holon` for the algebra primitives.

## Status

**Phase 1 complete.** Every startup-pipeline step from FOUNDATION.md is
implemented and tested end-to-end:

1. Parse
2. Entry-file shape check + config pass (`set-dims!`, `set-capacity-mode!`,
   `set-global-seed!`)
3. Recursive `load!` / `digest-load!` / `signed-load!` resolution
4. `defmacro` registration + quasiquote expansion (Racket sets-of-scopes
   hygiene, Flatt 2016)
5. Type declarations (`struct` / `enum` / `newtype` / `typealias`, with
   parametric names)
6. `define` registration — functions + lambdas with typed signatures
7. Name resolution across the symbol table and type environment
8. Rank-1 Hindley-Milner type check (parametric polymorphism, substitution,
   occurs-check; `:Any` banned)
9. Canonical-EDN hashing + SHA-256 source-file integrity + Ed25519
   signature verification
10. Freeze — `FrozenWorld` bundles config + types + macros + symbols;
    Rust borrow checker is the immutability gate
11. Invoke `:user::main` with three crossbeam channels
12. Constrained eval — four forms (`eval-ast!`, `eval-edn!`,
    `eval-digest!`, `eval-signed!`) with the same verification discipline
    as load

**353 tests passing; zero warnings.** Full test surface: library units,
integration tests, and end-to-end CLI tests that spawn the built `wat-vm`
binary and exercise it with real OS stdin/stdout.

## Module tour

- [`lexer`] — s-expression tokenizer. `:` is the symbol-literal reader
  macro; internal `::` is Rust's path separator, allowed freely.
  Paren-depth tracking handles `(…)` inside keyword bodies (`:fn(T,U)->R`,
  `:(i64,String)`).
- [`ast::WatAST`] — language-surface AST: `IntLit`, `FloatLit`, `BoolLit`,
  `StringLit`, `Keyword`, `Symbol(Identifier)`, `List`. Symbols carry
  `BTreeSet<ScopeId>` scope sets for hygiene.
- [`parser`] — recursive descent over tokens. `parse_one` / `parse_all`
  entry points. Reader macros (`` ` `` / `,` / `,@`) rewrite to
  `:wat::core::quasiquote` / `unquote` / `unquote-splicing`.
- [`config`] — entry-file discipline + `set-*!` setter commit. Required
  fields (`dims`, `capacity-mode`) + optional `global-seed` (default 42).
- [`load`] — recursive load-form resolution with `:wat::load::*` source
  interfaces and `:wat::verify::*` payload + algorithm keywords. Three
  load forms (`load!` / `digest-load!` / `signed-load!`). Cycle detection,
  commit-once, setter-in-loaded-file refusal.
- [`macros`] — `defmacro` + quasiquote + Racket sets-of-scopes hygiene.
- [`types`] — type declarations, `TypeEnv`, `TypeExpr` (Path / Parametric /
  Fn / Tuple / Var). `:Any` refused at parse.
- [`resolve`] — call-site reference validation; reserved-prefix gate
  (`:wat::core::`, `:wat::kernel::`, `:wat::algebra::`, `:wat::std::`,
  `:wat::config::`, `:wat::load::`, `:wat::verify::`, `:wat::eval::`).
- [`check`] — rank-1 HM. Built-in schemes: `∀T. T* -> Vec<T>` for
  `:wat::core::vec`; `∀T. T -> T -> :bool` for comparison operators;
  `∀T. Sender<T> -> T -> :()` for `:wat::kernel::send`; and so on.
- [`hash`] — canonical-EDN serialization + SHA-256 + Ed25519 verification.
- [`lower`] — `WatAST` algebra-core subtree → `holon::HolonAST`.
- [`runtime`] — AST walker, `:wat::core::*` / `:wat::algebra::*` dispatch,
  `:wat::kernel::stopped` / `send` / `recv`, four eval forms, `Value`
  enum with namespace-honest variant names (`Value::crossbeam_channel__Sender`,
  `Value::holon__HolonAST`, `Value::wat__core__lambda`, …).
- [`freeze`] — `FrozenWorld`, `startup_from_source`, `invoke_user_main`,
  `eval_*_in_frozen`.

## `wat-vm` binary

```
$ wat-vm <entry.wat>
```

Reads the entry file, runs the full startup pipeline, installs OS signal
handlers, wires stdio over `crossbeam_channel`, invokes `:user::main`,
waits for threads to drain, exits.

### `:user::main` contract

```scheme
(:wat::core::define (:user::main
                     (stdin  :crossbeam_channel::Receiver<String>)
                     (stdout :crossbeam_channel::Sender<String>)
                     (stderr :crossbeam_channel::Sender<String>)
                     -> :())
  ...body...)
```

Exact signature enforced at startup. Any deviation (different arity,
different parameter types, different return type) halts with exit code 3.

Signals: `SIGINT` and `SIGTERM` both route through one handler that sets
a kernel stop flag. User programs poll the flag via `(:wat::kernel::stopped)`
in their loops:

```scheme
(:wat::core::let (((stop? :bool) (:wat::kernel::stopped)))
  (if stop?
      ()
      ...do-work...))
```

Stdin: one line read from OS stdin, sent to the stdin channel, sender
dropped. A program that calls `(:wat::kernel::recv stdin)` once gets that
line. Multi-line stdin needs `:Option<T>` at the runtime layer — future
slice.

### Exit codes

| Code | Meaning |
|---|---|
| 0  | `:user::main` returned cleanly |
| 1  | Startup error (parse/config/load/macro/type/resolve/check) |
| 2  | Runtime error (channel disconnect, type mismatch, etc.) |
| 3  | `:user::main` signature mismatch |
| 64 | Usage error — wrong argv |
| 66 | Entry file read failed |

### Hello world (the test that proves it)

```scheme
;; echo.wat
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :crossbeam_channel::Receiver<String>)
                     (stdout :crossbeam_channel::Sender<String>)
                     (stderr :crossbeam_channel::Sender<String>)
                     -> :())
  (:wat::kernel::send stdout (:wat::kernel::recv stdin)))
```

```
$ echo watmin | wat-vm echo.wat
watmin
```

## Namespace discipline

Every type identifier in wat source shows its full namespace. Every `Value`
variant in the Rust enum encodes its path via `__` separator. Errors read
exactly like user-written declarations: `expected
crossbeam_channel::Sender, got i64`. No short names hiding what a value is
or where a type comes from.

Full rename table in `FOUNDATION-CHANGELOG.md` (entry dated 2026-04-19,
"Namespace honesty").

## What's next

Phase 1 is complete. Further work is additive:

- **Multi-line stdin.** `:Option<T>` runtime + `match` form → graceful EOF
  for `recv`.
- **More kernel primitives.** `:wat::kernel::spawn` / `select` / `drop` /
  `try-recv` for richer concurrency; matches FOUNDATION's eight-primitive
  kernel surface.
- **Full-program signature verification on the CLI.** `wat-vm --signed
  <algo> --sig <b64> --pubkey <b64>` verifies the post-expand AST before
  `:user::main` runs.
- **Compile path.** Emit Rust source from the frozen world; `rustc`
  produces a native binary with `wat`'s frontend as its builder.

## See also

- `../holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this
  crate evaluates against.
- `../holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
  — the language specification.
- `../holon-lab-trading/docs/058-backlog.md` — the implementation arc.
- `../holon-lab-trading/BOOK.md` — the story.
