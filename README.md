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
   `set-global-seed!`, `set-noise-floor!`)
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
    Rust borrow checker is the immutability gate; `EncodingCtx` (VM +
    ScalarEncoder + AtomTypeRegistry with `WatAST` canonicalizer) is
    constructed from Config and attached to `SymbolTable` at freeze
11. Invoke `:user::main` with three crossbeam channels
12. Constrained eval — four forms (`eval-ast!`, `eval-edn!`,
    `eval-digest!`, `eval-signed!`) with the same verification discipline
    as load

**Programs-as-holons operational.** `:wat::core::quote` + parametric
`:wat::algebra::Atom` + `:wat::core::atom-value` carry wat programs as
first-class data in the algebra. `:wat::core::presence` (FOUNDATION
1718) is the retrieval primitive — cosine between encoded holons,
returning scalar `:f64` the caller binarizes against the 5σ noise
floor committed at config pass. The vector-level proof runs end-to-end:

```
$ echo watmin | wat-vm presence-proof.wat
None       ; presence(program-atom, Bind(k, program-atom)) below floor
Some       ; presence(program-atom, Bind(Bind(k,p), k)) above floor
watmin     ; (eval-ast! (atom-value program-atom)) fires the echo
```

**372 tests passing; zero warnings.** Full test surface: library units,
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
  fields (`dims`, `capacity-mode`); optional `global-seed` (default 42)
  and `noise-floor` (default `5.0 / sqrt(dims)` — the 5σ substrate
  noise floor per FOUNDATION 1718). Each optional field overridable
  exactly once.
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
  `:wat::kernel::stopped` / `send` / `recv`, four eval forms. Programs-as-holons
  surface: `:wat::core::quote` captures unevaluated AST as `:wat::WatAST`;
  `:wat::algebra::Atom` accepts `Value::wat__WatAST` payloads (canonicalizer
  registered in `AtomTypeRegistry`); `:wat::core::atom-value` structurally
  reads an Atom's payload field; `:wat::core::let*` for sequential
  binding. Retrieval: `:wat::core::presence target reference -> :f64`
  (cosine between encoded holons; FOUNDATION 1718). Config accessors:
  `:wat::config::dims`, `:wat::config::global-seed`,
  `:wat::config::noise-floor`. `EncodingCtx` (VM + ScalarEncoder +
  registry + Config) attached to `SymbolTable` at freeze so primitives
  needing projection reach it via dispatch. `Value` enum with
  namespace-honest variant names (`Value::crossbeam_channel__Sender`,
  `Value::holon__HolonAST`, `Value::wat__WatAST`,
  `Value::wat__core__lambda`, …).
- [`freeze`] — `FrozenWorld`, `startup_from_source`, `invoke_user_main`,
  `eval_*_in_frozen`. Constructs `EncodingCtx` from `Config` at freeze
  and attaches it to the `SymbolTable`.

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

Signals: the kernel measures; userland owns transitions.

- **Terminal (SIGINT, SIGTERM)** → set the `stopped` flag irreversibly.
  Userland polls `(:wat::kernel::stopped)` and cascades shutdown by
  dropping its root producers.
- **Non-terminal (SIGUSR1, SIGUSR2, SIGHUP)** → each flips its own
  kernel-maintained boolean. Userland polls via
  `(:wat::kernel::sigusr1?)` / `(sigusr2?)` / `(sighup?)` and clears
  via the matching `(:wat::kernel::reset-sigusr1!)` / `(reset-sigusr2!)`
  / `(reset-sighup!)`. Coalesced — five SIGHUPs in a burst read as one
  "yes"; counter semantics is userland's problem if it needs them.

```scheme
(:wat::core::let (((stop? :bool) (:wat::kernel::stopped)))
  (if stop?
      ()
      ...do-work...))

;; Reload-on-SIGHUP pattern:
(:wat::core::if (:wat::kernel::sighup?)
    (:wat::core::let (((_ :()) (:my::app::reload-config)))
      (:wat::kernel::reset-sighup!))
    ())
```

Stdin: one line read from OS stdin, sent to the stdin channel, sender
dropped. A program that calls `(:wat::kernel::recv stdin)` gets back
`(Some line)` for the line and `:None` once the sender drops.

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
  (:wat::core::match (:wat::kernel::recv stdin)
    ((Some line) (:wat::kernel::send stdout line))
    (:None ())))
```

```
$ echo watmin | wat-vm echo.wat
watmin
```

`recv` returns `:Option<String>` — `(Some line)` on a payload,
`:None` when every sender has dropped. `:wat::core::match` decomposes
it; exhaustiveness on `:Option<T>` is a type-check-time requirement
(every match must cover both `:None` and `(Some _)`, or include a
wildcard).

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

- **More kernel primitives.** `:wat::kernel::spawn` / `select` / `drop` /
  `try-recv` / `join` / `make-bounded-queue` / `make-unbounded-queue` /
  `HandlePool` for richer concurrency; matches FOUNDATION's eight-primitive
  kernel surface. `Signal` enum + signals queue as `:user::main`'s
  fourth parameter.
- **Compile path.** Emit Rust source from the frozen world; `rustc`
  produces a native binary with `wat`'s frontend as its builder.

Signature verification is **per-form, not per-invocation.** It lives at
`:wat::core::signed-load!` (startup) and `:wat::core::eval-signed!`
(runtime). A program may invoke any number of either, each with its own
key and signature. There is no `wat-vm --signed` / `--sig` / `--pubkey`
CLI flag; a program's verification surface is its collection of
signed-* forms. See FOUNDATION's cryptographic-provenance section.

## See also

- `../holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this
  crate evaluates against.
- `../holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
  — the language specification.
- `../holon-lab-trading/docs/058-backlog.md` — the implementation arc.
- `../holon-lab-trading/BOOK.md` — the story.
