# wat

A Lisp-family language for holon algebra, hosted on Rust. Same pattern as
Clojure on the JVM: wat is a full language with its own parser, type
checker, macro expander, and runtime, and it borrows Rust's type system,
safety, and ecosystem underneath. Rust crates surface into wat source under
the `:rust::` namespace; wat programs call them like native forms.

This crate implements wat as specified by the 058 algebra-surface proposal
batch in the holon-lab-trading repo
(`docs/proposals/2026/04/058-ast-algebra-surface/`).

Ships:

- `wat` — the library (this crate).
- `wat-macros` — the sibling proc-macro crate. `#[wat_dispatch]` generates
  the shim code that surfaces a Rust `impl` block under `:rust::...`.
- `wat-vm` — the CLI runner. `wat-vm <entry.wat>` reads the file, runs the
  full startup pipeline, invokes `:user::main` with real stdio handles,
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
wat     (this crate — wat frontend + interpret runtime + :rust:: shims)
    ↑
holon-lab-trading / holon-lab-ddos / any wat-consuming application
```

Applications talk to `wat` to load and run their wat programs; `wat` talks
to `holon` for the algebra primitives. Applications that need to surface
their own Rust crates to wat (e.g. rusqlite, parquet, aya) use
`#[wat_dispatch]` + `RustDepsBuilder` — see [Rust interop](#rust-interop).

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
10. Freeze — `FrozenWorld` bundles config + types + macros + symbols; Rust
    borrow checker is the immutability gate; `EncodingCtx` (VM +
    ScalarEncoder + AtomTypeRegistry with `WatAST` canonicalizer) is
    constructed from Config and attached to `SymbolTable` at freeze
11. Invoke `:user::main` with the real `io::Stdin` / `io::Stdout` /
    `io::Stderr` handles
12. Constrained eval — four forms (`eval-ast!`, `eval-edn!`,
    `eval-digest!`, `eval-signed!`) with the same verification discipline
    as load

**Programs-as-holons operational.** `:wat::core::quote` + parametric
`:wat::algebra::Atom` + `:wat::core::atom-value` carry wat programs as
first-class data in the algebra. `:wat::algebra::presence?` (FOUNDATION 1718)
is the retrieval primitive — cosine between encoded holons, returning
scalar `:f64` the caller binarizes against the 5σ noise floor committed at
config pass. The vector-level proof runs end-to-end:

```
$ echo watmin | wat-vm presence-proof.wat
None       ; presence(program-atom, Bind(k, program-atom)) below floor
Some       ; presence(program-atom, Bind(Bind(k,p), k)) above floor
watmin     ; (eval-ast! (atom-value program-atom)) fires the echo
```

**Rust interop operational.** The `:rust::` namespace carries any
consumer-registered Rust type: `:rust::lru::LruCache<K,V>` ships as a
default, `:rust::std::io::Stdin` / `Stdout` / `Stderr` are kernel-wired for
`:user::main`, and application crates layer their own (`:rust::rusqlite::`,
`:rust::parquet::`, …) through `#[wat_dispatch]`. Three scope modes cover
the full Rust ownership surface: `shared` (plain `Arc<T>`), `thread_owned`
(`ThreadOwnedCell<T>` with a thread-id guard — zero Mutex), and
`owned_move` (`OwnedMoveCell<T>` — consumed on first use).

**Capacity-guard arc operational.** `:wat::algebra::Bundle` enforces
Kanerva's per-frame capacity at dispatch time and returns
`:Result<holon::HolonAST, :wat::algebra::CapacityExceeded>`; authors
choose `:silent` / `:warn` / `:error` / `:abort` at startup via
`:wat::config::set-capacity-mode!`. Paired with two supporting forms
that shipped in the same arc: `:wat::core::try` for error-propagation
without try/catch, and first-class struct runtime (auto-generated
`<struct-path>/new` constructors and `<struct-path>/<field>` accessors
from any `:wat::core::struct` declaration). See
[Capacity guard](#capacity-guard--bundles-result-return) below for the
canonical pattern.

**490 library-unit tests + 70+ integration tests pass; zero clippy
warnings.** Full test surface: library units, macro-feature integration
(`wat_dispatch_193a`/`193b`/`e1_vec`/`e2_tuple`/`e3_result`/`e4_shared`/
`e5_owned_move`), `wat_core_try` (13 cases), `wat_structs` (9 cases),
`wat_bundle_capacity` (9 cases across the four modes), `wat_vm_cache`
(the nested-driver shutdown proof), and `wat_vm_cli` (end-to-end spawns
of the built binary against real OS stdio).

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
  and `noise-floor` (default `5.0 / sqrt(dims)` — the 5σ substrate noise
  floor per FOUNDATION 1718). Each optional field overridable exactly once.
- [`load`] — recursive load-form resolution with `:wat::load::*` source
  interfaces and `:wat::verify::*` payload + algorithm keywords. Three
  load forms (`load!` / `digest-load!` / `signed-load!`). Cycle detection,
  commit-once, setter-in-loaded-file refusal.
- [`macros`] — `defmacro` + quasiquote + Racket sets-of-scopes hygiene.
- [`types`] — type declarations, `TypeEnv`, `TypeExpr` (Path / Parametric /
  Fn / Tuple / Var). `:Any` refused at parse.
- [`resolve`] — call-site reference validation; reserved-prefix gate
  (`:wat::core::`, `:wat::kernel::`, `:wat::algebra::`, `:wat::std::`,
  `:wat::config::`, `:wat::load::`, `:wat::verify::`, `:wat::eval::`,
  `:wat::io::`, `:rust::`).
- [`check`] — rank-1 HM. Built-in schemes for the wat core; the
  `:rust::*` surface registers schemes dynamically through
  [`rust_deps::RustDepsRegistry`].
- [`hash`] — canonical-EDN serialization + SHA-256 + Ed25519 verification.
- [`lower`] — `WatAST` algebra-core subtree → `holon::HolonAST`.
- [`runtime`] — AST walker, `:wat::core::*` / `:wat::algebra::*` dispatch,
  `:wat::kernel::*` (stopped?, send, recv, try-recv, drop, spawn, join,
  select, HandlePool, make-bounded-queue, make-unbounded-queue, user-signal
  query + reset), `:wat::io::write` / `:wat::io::read-line`, four eval
  forms. Programs-as-holons surface: `:wat::core::quote` captures
  unevaluated AST as `:wat::WatAST`; `:wat::algebra::Atom` accepts
  `Value::wat__WatAST` payloads; `:wat::core::atom-value` structurally
  reads an Atom's payload field. `Value` enum with namespace-honest
  variant names (`Value::io__Stdin`, `Value::holon__HolonAST`,
  `Value::wat__WatAST`, `Value::crossbeam_channel__Sender`,
  `Value::RustOpaque` for the generic `:rust::*` opaques, …).
- [`freeze`] — `FrozenWorld`, `startup_from_source`, `invoke_user_main`,
  `eval_*_in_frozen`. Constructs `EncodingCtx` from `Config` at freeze
  and attaches it to the `SymbolTable`.
- [`rust_deps`] — the `:rust::*` namespace registry. `RustDepsBuilder`
  composes consumer shims over the wat-rs defaults; `FromWat` / `ToWat`
  traits marshal wat `Value` ↔ Rust types; `ThreadOwnedCell<T>` and
  `OwnedMoveCell<T>` implement the `thread_owned` and `owned_move` scope
  disciplines. See [Rust interop](#rust-interop).
- [`stdlib`] — baked-in wat source files, registered before user code
  parses: `Subtract`, `Amplify`, `Log`, `Circular`, `Reject`, `Project`,
  `Sequential`, `Ngram` / `Bigram` / `Trigram`, `LocalCache`, and the
  two programs `program::Console` and `program::Cache<K,V>`.

## Rust interop

wat programs reach into Rust through the `:rust::` namespace. A path like
`:rust::lru::LruCache<String,i64>::new` names a method on a concrete Rust
type; the wat runtime calls into the shim the consumer registered.

Namespaces are **fully qualified and honest**: a wat program names a Rust
type by its full Rust path, not a short alias. `:rust::std::io::Stdin`
(not `:rust::Stdin`). `:rust::crossbeam_channel::Sender<T>` (not
`:rust::Sender<T>`). `:wat::` and `:rust::` are sibling namespaces, both
rooted at the colon, and a wat program declares which `:rust::*` paths it
intends to use:

```scheme
(:wat::core::use! :rust::lru::LruCache)
(:wat::core::use! :rust::rusqlite::Connection)
```

The resolver gates every `:rust::X` reference against the `use!` set — a
program cannot reach into a crate it hasn't declared.

### Shipping a shim

The `#[wat_dispatch]` proc-macro (in `wat-macros`) generates the dispatch
function, type-scheme function, and registry hook from an annotated
`impl` block. Here is the complete shim for `:rust::lru::LruCache<K,V>`
that ships in this crate:

```rust
use wat_macros::wat_dispatch;
use wat::rust_deps::RustDepsBuilder;

pub struct WatLruCache {
    inner: lru::LruCache<String, wat::runtime::Value>,
}

#[wat_dispatch(
    path = ":rust::lru::LruCache",
    scope = "thread_owned",
    type_params = "K,V"
)]
impl WatLruCache {
    pub fn new(capacity: i64) -> Self { /* ... */ }
    pub fn put(&mut self, k: Value, v: Value) { /* ... */ }
    pub fn get(&mut self, k: Value) -> Option<Value> { /* ... */ }
}

pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatLruCache::register(builder);
}
```

The macro reads the `impl` block, generates one `dispatch_<m>` +
`scheme_<m>` per method, wraps the register calls in a module named
`__wat_dispatch_<TypeIdent>`, and leaves the original `impl` untouched.

### Scope modes

Rust ownership semantics that cross the wat boundary fall into three
modes. The `scope` attribute picks one:

- **`shared`** — plain `Arc<T>`. For immutable / shareable Rust values
  (query results, frozen snapshots). `&self` methods only.
- **`thread_owned`** — `Arc<ThreadOwnedCell<T>>`. Every op asserts the
  current thread is the owner before touching the interior; cross-thread
  access errors cleanly. Zero Mutex — the guard is structural, not
  contended. For mutable state with single-thread affinity
  (`lru::LruCache`, `rusqlite::Connection` in some configs).
- **`owned_move`** — `Arc<OwnedMoveCell<T>>`. Ownership transfers out of
  the cell on first use via an atomic take; subsequent access errors.
  For consumed-after-use handles (prepared-statement bindings, one-shot
  tokens).

### Consumer composition

An application that bundles its own shims composes them on top of the
wat-rs defaults:

```rust
use wat::rust_deps::{install, RustDepsBuilder};

fn main() {
    let mut deps = RustDepsBuilder::with_wat_rs_defaults();
    rusqlite_shim::register(&mut deps);   // consumer's crate
    parquet_shim::register(&mut deps);    // consumer's crate
    install(deps.build()).expect("install rust_deps once");
    // ...now run a wat program that can (:wat::core::use!) any of these...
}
```

The registry is installed once before wat code runs; the wat-vm binary
installs it lazily with defaults when no consumer has done so, which keeps
unit tests running without setup.

See `docs/arc/2026/04/002-rust-interop-macro/MACRO-DESIGN.md` for the full
design and `NAMESPACE-PRINCIPLE.md` for the naming rule.

## `wat-vm` binary

```
$ wat-vm <entry.wat>
```

Reads the entry file, runs the full startup pipeline, installs OS signal
handlers, passes real `io::Stdin` / `io::Stdout` / `io::Stderr` to
`:user::main`, waits for the program to return, exits.

### `:user::main` contract

```scheme
(:wat::core::define (:user::main
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  ...body...)
```

Exact signature enforced at startup. Any deviation (different arity,
different parameter types, different return type) halts with exit code 3.

The program reads lines with `(:wat::io::read-line stdin)` and writes with
`(:wat::io::write stdout msg)` / `(:wat::io::write stderr msg)`. Both
primitives go straight to the OS stream (std's internal locking handles
concurrent writers). No bridge threads, no tagged-tuple hops in the hot
path — honest stdio.

Signals: the kernel measures; userland owns transitions.

- **Terminal (SIGINT, SIGTERM)** → set the `stopped` flag irreversibly.
  Userland polls `(:wat::kernel::stopped?)` and cascades shutdown by
  dropping its root producers.
- **Non-terminal (SIGUSR1, SIGUSR2, SIGHUP)** → each flips its own
  kernel-maintained boolean. Userland polls via
  `(:wat::kernel::sigusr1?)` / `(sigusr2?)` / `(sighup?)` and clears via
  the matching `(:wat::kernel::reset-sigusr1!)` / `(reset-sigusr2!)` /
  `(reset-sighup!)`. Coalesced — five SIGHUPs in a burst read as one
  "yes"; counter semantics is userland's problem if it needs them.

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

(:wat::core::use! :rust::std::io::Stdin)
(:wat::core::use! :rust::std::io::Stdout)
(:wat::core::use! :rust::std::io::Stderr)

(:wat::core::define (:user::main
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::core::match (:wat::io::read-line stdin) -> :()
    ((Some line) (:wat::io::write stdout line))
    (:None ())))
```

```
$ echo watmin | wat-vm echo.wat
watmin
```

`read-line` returns `:Option<String>` — `(Some line)` on a payload,
`:None` on EOF. `:wat::core::match` decomposes it; exhaustiveness on
`:Option<T>` is a type-check-time requirement.

## Stdlib

Every file under `wat/std/` is baked into the binary at compile time via
`include_str!` and registered during startup before user code parses.
Consumers reference them by path — no explicit `load!` needed.

Algebra conveniences:
- `:wat::std::Amplify`, `:wat::std::Subtract`, `:wat::std::Log`,
  `:wat::std::Circular`, `:wat::std::Reject`, `:wat::std::Project`,
  `:wat::std::Sequential`, `:wat::std::Ngram` / `:wat::std::Bigram` /
  `:wat::std::Trigram`.

Caches (FOUNDATION § caching stack):
- `:wat::std::LocalCache<K,V>` — L1. Three thin wrappers over
  `:rust::lru::LruCache`. Single-thread-owned, no pipe, no queue. Fastest
  memoization possible.
- `:wat::std::program::Cache<K,V>` — L2. Driver thread owns its own
  `LocalCache`; clients send tagged requests with a reply-to sender
  embedded. Nested tuple protocol, allocates the cache **inside** the
  driver thread (thread-owned values must not cross threads).

Stream programs:
- `:wat::std::program::Console` — the single gateway to stdout+stderr.
  Owns the real IO handles, hands out pooled `Sender<(i64,String)>` via
  `:wat::kernel::HandlePool`; tag 0 = stdout, tag 1 = stderr. A well-formed
  program routes all output through Console handles and leaves the raw
  stdout/stderr bindings alone.

## Capacity guard — Bundle's Result return

`:wat::algebra::Bundle` enforces Kanerva's per-frame capacity bound at
dispatch time. Every program picks a `:capacity-mode` at startup; the
Bundle dispatcher consults it when a frame's constituent count exceeds
`floor(sqrt(dims))` and behaves per the committed policy.

**Budget.** `floor(sqrt(dims))` — at d=10k → 100, at d=4k → 64, at
d=1k → 32. The wat algebra is AST-primary (no codebook to distinguish
against), so the classical `d/(2·ln K)` bound has no `K` term;
`sqrt(d)` is what keeps a single bundled element's presence
comfortably above the 5σ noise floor.

**Return type (every mode).**
```
:wat::algebra::Bundle : :Vec<holon::HolonAST>
                     -> :Result<holon::HolonAST, :wat::algebra::CapacityExceeded>
```

**Four modes** (`:wat::config::set-capacity-mode!`):

| Mode | Under budget | Over budget |
|---|---|---|
| `:silent` | `Ok(h)` | `Ok(h)` — degraded vector, no check, no diagnostic |
| `:warn`   | `Ok(h)` | `Ok(h)` — degraded vector plus `eprintln!` cost/budget/dims |
| `:error`  | `Ok(h)` | `Err(CapacityExceeded { cost, budget })` |
| `:abort`  | `Ok(h)` | `panic!` with diagnostic — fail-closed, no cleanup |

**`:wat::algebra::CapacityExceeded`** is a built-in struct:

```scheme
(:wat::core::struct :wat::algebra::CapacityExceeded
  (cost   :i64)   ;; the constituent count the Bundle was asked to hold
  (budget :i64))  ;; floor(sqrt(dims)) at the dispatcher
```

Auto-generated accessors `:wat::algebra::CapacityExceeded/cost` and
`/budget` read each field. No user declaration required — wat-rs seeds
this via `TypeEnv::with_builtins()`.

**The canonical program shape** uses `:wat::core::try` to propagate
Err through a Result-returning helper and `match` to handle at the
caller:

```scheme
(:wat::config::set-dims! 10000)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:app::build
                    (items :Vec<holon::HolonAST>)
                    -> :Result<holon::HolonAST, wat::algebra::CapacityExceeded>)
  (Ok (:wat::core::try (:wat::algebra::Bundle items))))

(:wat::core::define (:user::main
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::core::match (:app::build huge-list) -> :()
    ((Ok _) ())
    ((Err e)
      (:wat::io::write stderr
        (format-overflow
          (:wat::algebra::CapacityExceeded/cost e)
          (:wat::algebra::CapacityExceeded/budget e))))))
```

Every stdlib macro whose expansion ends in `:wat::algebra::Bundle`
inherits the Result wrap — `:wat::std::Ngram`, `:wat::std::Bigram`,
`:wat::std::Trigram`. Callers match or `try` at the call site.

**Two supporting forms** that shipped in the same arc:

- **`:wat::core::try`** — unwrap `Ok v` or short-circuit the innermost
  enclosing Result-returning function/lambda with `Err e`. Not
  try/catch; no handler block. Matches Rust's `?`-operator scoping.
- **Struct runtime** — `(:wat::core::struct :my::ns::T (f1 :T1) ...)`
  declarations auto-generate `:my::ns::T/new` constructors and
  `:my::ns::T/<field>` accessors at registration time. Users invoke
  them by full keyword path. No named-argument construction, no
  field-by-bare-keyword dispatch — positional construction plus
  accessor keyword-paths, symmetric on both sides via `let` bindings.

Full design: the INSCRIPTION 2026-04-19 entries across
`058-003-bundle-list-signature`, `058-030-types`, and the new
`058-033-try` proposal, rolled up in FOUNDATION-CHANGELOG.

## Namespace discipline

Every type identifier in wat source shows its full namespace. Every
`Value` variant in the Rust enum encodes its path via `__` separator.
Errors read exactly like user-written declarations: *expected
`:rust::crossbeam_channel::Sender<String>`, got `:i64`*. No short names
hiding what a value is or where a type comes from.

Two sibling namespaces, both rooted at the colon:
- `:wat::*` — forms and types defined by the wat language itself
  (`:wat::core::*`, `:wat::algebra::*`, `:wat::kernel::*`, `:wat::std::*`,
  `:wat::config::*`, `:wat::load::*`, `:wat::verify::*`, `:wat::eval::*`,
  `:wat::io::*`).
- `:rust::*` — forms and types surfaced from Rust crates
  (`:rust::std::io::*`, `:rust::crossbeam_channel::*`, `:rust::lru::*`,
  and whatever the consumer registered).

Full honesty rule in `docs/arc/2026/04/002-rust-interop-macro/NAMESPACE-PRINCIPLE.md`
and the FOUNDATION-CHANGELOG entry dated 2026-04-19 ("Namespace honesty").

## Workspace layout

```
wat-rs/
├── Cargo.toml              # workspace + wat package
├── src/
│   ├── lib.rs              # extern crate self as wat;
│   ├── bin/wat-vm.rs       # CLI runner
│   ├── {lexer,parser,config,load,identifier,macros,
│   │    types,resolve,check,hash,lower,runtime,
│   │    freeze,stdlib}.rs  # pipeline stages
│   └── rust_deps/
│       ├── mod.rs          # RustDepsBuilder, Registry, SchemeCtx,
│       │                   # UseDeclarations, install(), get()
│       ├── marshal.rs      # FromWat/ToWat, ThreadOwnedCell,
│       │                   # OwnedMoveCell, RustOpaqueInner
│       └── lru.rs          # :rust::lru::LruCache shim (macro-generated)
├── wat-macros/             # sibling proc-macro crate
│   └── src/{lib.rs,codegen.rs}
├── wat/std/                # baked-in wat source files
│   ├── Amplify.wat Subtract.wat Log.wat Circular.wat
│   ├── Reject.wat Project.wat Sequential.wat
│   ├── Ngram.wat Bigram.wat Trigram.wat LocalCache.wat
│   └── program/
│       ├── Console.wat
│       └── Cache.wat
├── tests/                  # integration suites
│   ├── mvp_end_to_end.rs
│   ├── wat_dispatch_{193a,193b,e1_vec,e2_tuple,
│   │                 e3_result,e4_shared,e5_owned_move}.rs
│   ├── wat_vm_cache.rs
│   └── wat_vm_cli.rs
└── docs/
    ├── README.md           # orientation
    ├── USER-GUIDE.md       # building on wat
    ├── CONVENTIONS.md      # naming rules for new primitives
    ├── ZERO-MUTEX.md       # the concurrency architecture
    └── arc/2026/04/
        ├── 001-caching-stack/              # DESIGN + DEADLOCK-POSTMORTEM
        ├── 002-rust-interop-macro/         # MACRO-DESIGN + NAMESPACE-PRINCIPLE + PROGRESS
        ├── 003-tail-call-optimization/     # DESIGN + INSCRIPTION
        ├── 004-lazy-sequences-and-pipelines/ # DESIGN + INSCRIPTION + BACKLOG
        └── 005-stdlib-naming-audit/        # DESIGN + INVENTORY
```

## What's next

Phase 1 is complete. Further work is additive:

- **More consumer shims.** holon-lab-trading needs `:rust::rusqlite::` for
  the candle DB and `:rust::parquet::` for archive reads. Each follows
  the `#[wat_dispatch]` pattern demonstrated by `wat-rs/src/rust_deps/lru.rs`.
- **Compile path.** Emit Rust source from the frozen world; `rustc`
  produces a native binary with `wat`'s frontend as its builder.
- **Macro return-type marshaling of `Result<T,E>`.** Today's macro
  surfaces invalid Rust inputs as panics because `RuntimeError` round-trip
  isn't yet plumbed through the return path; shim authors work around this
  by validating arguments in wat source. Lands when the next macro slice
  closes the gap.

Signature verification is **per-form, not per-invocation.** It lives at
`:wat::core::signed-load!` (startup) and `:wat::core::eval-signed!`
(runtime). A program may invoke any number of either, each with its own
key and signature. There is no `wat-vm --signed` / `--sig` / `--pubkey`
CLI flag; a program's verification surface is its collection of `signed-*`
forms. See FOUNDATION's cryptographic-provenance section.

## See also

- `../holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this
  crate evaluates against.
- `../holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
  — the language specification.
- `../holon-lab-trading/docs/058-backlog.md` — the implementation arc.
- `../holon-lab-trading/BOOK.md` — the story.
- `docs/USER-GUIDE.md` — building applications on wat.
- `docs/CONVENTIONS.md` — naming rules for new primitives.
- `docs/arc/2026/04/001-caching-stack/DESIGN.md` — the L1/L2 cache design.
- `docs/arc/2026/04/002-rust-interop-macro/MACRO-DESIGN.md` — the
  `#[wat_dispatch]` design.
