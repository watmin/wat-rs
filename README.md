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
- `wat` — the CLI binary. `wat <entry.wat>` runs a program; see
  [the `:user::main` contract](#usermain-contract) below. (A legacy
  `wat test <path>` subcommand still ships — it predates the Cargo
  integration — but `cargo test` through `wat::test! {}` is the
  canonical test workflow. See [self-hosted testing](#self-hosted-testing).)

## What wat is

`wat` consumes wat source and produces vectors (via `holon`) and runtime
effects (via the kernel primitives the binary wires up).

**The INTERPRET path.** Parse → load-resolve → macro-expand → register
types → register defines → resolve → type-check → freeze → invoke
`:user::main`. The runtime dispatches algebra-core UpperCalls
(`:wat::holon::Atom`, `:wat::holon::Bind`, …) to `holon::HolonAST`
and encodes via `holon::encode`.

A source-to-source COMPILE path was sketched in the 058 batch's
`WAT-TO-RUST.md` but retired 2026-04-21. Rust-interop turned out to be
covered by `#[wat_dispatch]` + `:rust::` namespace (arc 002, BOOK
Chapter 18); native binary emission has no current caller. The sketch
stays as historical record.

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

**Every startup-pipeline step from FOUNDATION.md is implemented.** Every
058 proposal that shipped has an INSCRIPTION record. Ten arcs landed
between 2026-03 and 2026-04; each with a dated DESIGN / BACKLOG /
INSCRIPTION triple under `docs/arc/2026/04/`.

The language is self-hosted at the testing layer: `cargo test`
(through `tests/test.rs` + `wat::test! {}`) runs the wat test tree
alongside the Rust suite, and the assertion primitives assert about
the assertion primitives. Arc 007 — *wat tests wat* — was
the proof point. DESIGN's closing line, *"if wat can test wat, the
language is complete for its own verification,"* held.

**Programs-as-holons operational.** `:wat::core::quote` + parametric
`:wat::algebra::Atom` + `:wat::core::atom-value` carry wat programs as
first-class data in the algebra. Arc 010's `:wat::core::forms` ships
the variadic-quote substrate so AST-consuming callers compose without
per-form quote ceremony. `:wat::algebra::presence?` (FOUNDATION 1718) is
the retrieval primitive — cosine between encoded holons binarized
against the 5σ noise floor committed at config pass; returns `:bool`.
For the raw scalar, use `:wat::algebra::cosine` (returns `:f64`) and
compare against `:wat::config::noise-floor` yourself.

**Rust interop operational.** The `:rust::` namespace carries any
consumer-registered Rust type. `:rust::lru::LruCache<K,V>` ships as a
default; `:wat::io::IOReader` / `IOWriter` are abstract types the CLI
wraps real OS stdio in and the sandbox wraps StringIo in (arc 008);
application crates layer their own (`:rust::rusqlite::`, `:rust::parquet::`,
…) through `#[wat_dispatch]`. Three scope modes cover the full Rust
ownership surface: `shared` (plain `Arc<T>`), `thread_owned`
(`ThreadOwnedCell<T>` with a thread-id guard — zero Mutex), and
`owned_move` (`OwnedMoveCell<T>` — consumed on first use).

**Self-hosted testing operational.** Arc 007 shipped `:wat::kernel::
run-sandboxed` (in-process sandbox with `catch_unwind` panic
isolation) and `:wat::kernel::run-sandboxed-ast` (AST-entry path for
macro-generated programs). Arc 011 added the AST-entry hermetic
(`run-sandboxed-hermetic-ast`) for services that spawn threads.
Arc 012 moved hermetic to wat stdlib on top of `:wat::kernel::
fork-with-forms` + `wait-child` — no binary-path coupling, no
tempfile. The stdlib wraps them as `:wat::test::run`,
`:wat::test::run-ast`, `:wat::test::run-hermetic-ast`,
`:wat::test::deftest` (Clojure-style ergonomic shell), and six
assertion primitives with panic-and-catch semantics. `cargo test`
(via `tests/test.rs` + `wat::test! {}`) auto-discovers deftests by
`test-` prefix + zero-arg `:wat::kernel::RunResult` return.

**Capacity-guard arc operational.** `:wat::algebra::Bundle` enforces
Kanerva's per-frame capacity at dispatch time and returns
`:Result<holon::HolonAST, :wat::algebra::CapacityExceeded>`; authors
choose `:silent` / `:warn` / `:error` / `:abort` at startup via
`:wat::config::set-capacity-mode!`. Paired with two supporting forms
that shipped in the same arc: `:wat::core::try` for error-propagation
without try/catch, and first-class struct runtime (auto-generated
`<struct-path>/new` constructors and `<struct-path>/<field>` accessors
from any `:wat::core::struct` declaration). See
[Capacity guard](#capacity-guard--bundles-result-return) below.

**Test surface: 731 Rust + 31 wat, zero regressions.** Library units,
macro-feature integration (`wat_dispatch_193a`/`193b`/`e1_vec`/`e2_tuple`
/`e3_result`/`e4_shared`/`e5_owned_move`), `wat_core_try` (13),
`wat_structs` (9), `wat_bundle_capacity` (9), `wat_stream` (22),
`wat_core_forms` (6), `wat_names_are_values` (5), `wat_harness` (7),
`wat_run_sandboxed{,_ast}` + `wat_hermetic_round_trip`, `wat_test_cli` +
`wat_cli` + `wat_cache`, zero clippy warnings. On the wat side: the
`wat-tests/` tree (run via `cargo test`) covers every stdlib-file test
written in wat (Subtract, Circular, Reject/Project, Sequential,
Trigram, test-harness self-tests, Console + Cache via hermetic
sandbox, stream with-state).

## Module tour

- [`lexer`] — s-expression tokenizer. `:` is the symbol-literal reader
  macro; internal `::` is Rust's path separator, allowed freely.
  Paren-depth tracking handles `(…)` inside keyword bodies
  (`:fn(T,U)->R`, `:(i64,String)`). UTF-8-correct via `char_indices`
  (arc 008 caught a pre-existing bug and fixed it).
- [`ast::WatAST`] — language-surface AST: `IntLit`, `FloatLit`,
  `BoolLit`, `StringLit`, `Keyword`, `Symbol(Identifier)`, `List`.
  Symbols carry `BTreeSet<ScopeId>` scope sets for hygiene.
- [`parser`] — recursive descent over tokens. `parse_one` / `parse_all`
  entry points. Reader macros (`` ` `` / `,` / `,@`) rewrite to
  `:wat::core::quasiquote` / `unquote` / `unquote-splicing`.
- [`config`] — entry-file discipline + `set-*!` setter commit. Required
  fields (`dims`, `capacity-mode`); optional `global-seed` (default 42)
  and `noise-floor` (default `5.0 / sqrt(dims)` — the 5σ substrate noise
  floor per FOUNDATION 1718). Each optional field overridable exactly
  once.
- [`load`] — recursive load-form resolution. Six load forms
  (`load-file!` / `load-string!` / `digest-load!` / `digest-load-string!` /
  `signed-load!` / `signed-load-string!`) — each takes its source
  directly (path or inline string) as the first argument. Verification
  payloads use `:wat::verify::*` keywords (`file-path` / `string` /
  `digest-sha256` / `signed-ed25519`). Cycle detection, canonical-path
  dedup (arc 027 slice 1). Capability-gated via `SourceLoader` (arc 007
  slice 1) — `ScopedLoader`, `FsLoader`, `InMemoryLoader` all impl.
- [`macros`] — `defmacro` + quasiquote + Racket sets-of-scopes hygiene,
  plus `&`-suffix variadic rest-params.
- [`types`] — type declarations, `TypeEnv`, `TypeExpr` (Path / Parametric
  / Fn / Tuple / Var). `:Any` refused at parse. `TypeEnv::with_builtins()`
  seeds wat-rs's own `:wat::*` types (Failure, Location, Frame,
  RunResult, CapacityExceeded, EvalError).
- [`resolve`] — call-site reference validation; reserved-prefix gate
  (`:wat::*` catch-all covering every sub-namespace + root-level load/eval
  forms, plus `:rust::*` — arc 028 consolidation).
- [`check`] — rank-1 HM. Built-in schemes for the wat core; the
  `:rust::*` surface registers schemes dynamically through
  [`rust_deps::RustDepsRegistry`]. Structural equality across composite
  values. Arc 009: keyword-as-value lift — a registered define's
  keyword-path in expression position infers to `:fn(...)->Ret`.
- [`hash`] — canonical-EDN serialization + SHA-256 + Ed25519 verification.
- [`lower`] — `WatAST` algebra-core subtree → `holon::HolonAST`.
- [`runtime`] — AST walker; dispatch for `:wat::core::*` / `:wat::algebra::*`
  / `:wat::kernel::*` / `:wat::io::*` / `:wat::test::*` primitives; four
  eval forms; `:wat::core::forms` (variadic quote, arc 010). Programs-
  as-holons surface: `:wat::core::quote` captures unevaluated AST as
  `:wat::WatAST`; `:wat::algebra::Atom` accepts `Value::wat__WatAST`
  payloads; `:wat::core::atom-value` structurally reads the payload.
  `Value` enum with namespace-honest variant names
  (`Value::io__IOReader`, `Value::holon__HolonAST`, `Value::wat__WatAST`,
  `Value::crossbeam_channel__Sender`, `Value::RustOpaque` for generic
  `:rust::*` opaques, …).
- [`io`] — the abstract IO trait objects (arc 008). `WatReader` /
  `WatWriter` traits; `RealStdin` / `RealStdout` / `RealStderr` wrap
  `std::io` handles; `StringIoReader` / `StringIoWriter` ThreadOwnedCell-
  backed for in-memory testing. 15 primitives under the `<Type>/<method>`
  convention.
- [`assertion`] — arc 007 slice 3. `AssertionPayload` struct for
  panic-and-catch. `:wat::kernel::assertion-failed!` primitive raises
  via `panic_any`; the sandbox's `catch_unwind` downcasts and populates
  `:wat::kernel::Failure.actual` / `.expected`.
- [`sandbox`] — the in-process sandbox primitives: `run-sandboxed`,
  `run-sandboxed-ast`. Shared failure downcast chain
  (`AssertionPayload` → structured; string panic → message; runtime
  error → message). Arc 012 retired the hermetic Rust primitives
  (string- and AST-entry); hermetic is now wat stdlib in
  `wat/std/hermetic.wat` on top of `:wat::kernel::fork-with-forms`.
- [`fork`] — the fork substrate (arc 012): `:wat::kernel::pipe`,
  `fork-with-forms`, `wait-child`. `PipeReader` / `PipeWriter`
  live in `io.rs` (same trait surface as `RealStdin` etc.).
- [`harness`] — `wat::Harness` thin embedding wrapper for Rust programs
  that host wat as a sub-language. Sugar over `startup_from_source` +
  `StringIo` + `invoke_user_main` + `snapshot_bytes` (arc 007 slice 5).
- [`freeze`] — `FrozenWorld`, `startup_from_source`, `startup_from_forms`
  (arc 007 slice 3b — split at the parse boundary for AST-entry
  callers), `invoke_user_main`, `eval_*_in_frozen`. Constructs
  `EncodingCtx` from `Config` at freeze and attaches to the
  `SymbolTable`.
- [`rust_deps`] — the `:rust::*` namespace registry. `RustDepsBuilder`
  composes consumer shims over the wat-rs defaults; `FromWat` / `ToWat`
  traits marshal wat `Value` ↔ Rust types; `ThreadOwnedCell<T>` and
  `OwnedMoveCell<T>` implement the `thread_owned` and `owned_move`
  scope disciplines. See [Rust interop](#rust-interop).
- [`stdlib`] — baked-in wat source files, registered before user code
  parses. See [Stdlib](#stdlib).
- [`string_ops`] — `:wat::core::string::*` + `:wat::core::regex::*`
  primitives (arc 007 slice 3 precursor). Seven char-oriented string
  ops plus regex match.

## Rust interop

wat programs reach into Rust through the `:rust::` namespace. A path like
`:rust::lru::LruCache<String,i64>::new` names a method on a concrete Rust
type; the wat runtime calls into the shim the consumer registered.

Namespaces are **fully qualified and honest**: a wat program names a Rust
type by its full Rust path, not a short alias.
`:rust::crossbeam_channel::Sender<T>`, not `:rust::Sender<T>`.
`:wat::` and `:rust::` are sibling namespaces, both rooted at the colon,
and a wat program declares which `:rust::*` paths it intends to use:

```scheme
(:wat::core::use! :rust::lru::LruCache)
(:wat::core::use! :rust::rusqlite::Connection)
```

The resolver gates every `:rust::X` reference against the `use!` set — a
program cannot reach into a crate it hasn't declared.

### Shipping a shim

The `#[wat_dispatch]` proc-macro (in `wat-macros`) generates the dispatch
function, type-scheme function, and registry hook from an annotated
`impl` block. Three scope modes cover the full Rust ownership surface:

- **`shared`** — plain `Arc<T>`. For immutable / shareable Rust values.
  `&self` methods only.
- **`thread_owned`** — `Arc<ThreadOwnedCell<T>>`. Every op asserts the
  current thread is the owner; zero Mutex; for mutable state with
  single-thread affinity (`lru::LruCache`, `rusqlite::Connection`).
- **`owned_move`** — `Arc<OwnedMoveCell<T>>`. Ownership transfers out on
  first use via an atomic take. For consumed-after-use handles.

An application that bundles its own shims composes them via the
`wat::main!` macro's `deps:` list — each dep is a crate (or
module) exposing `pub fn wat_sources()` and `pub fn register()`
per the arc 013 external-wat-crate contract:

```rust
// src/main.rs — one line wires the substrate + all deps + user source
wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru, rusqlite_shim, parquet_shim],
}
```

The macro expands to `fn main() -> Result<(), wat::HarnessError>`
that installs both halves of each dep's contract
(`wat::source::install_dep_sources` for wat source,
`wat::rust_deps::install` for Rust shims), freezes the user source
against the composed world, and invokes `:user::main` with real
OS stdio. See `docs/USER-GUIDE.md` § 1 for the full consumer
shape + test suite companion (`wat::test_suite!`).

Reference crate: `crates/wat-lru/` — the first external wat crate
(arc 013). Shows the publisher-side contract; `examples/with-lru/`
shows the consumer-side shape. See
`docs/arc/2026/04/002-rust-interop-macro/MACRO-DESIGN.md` for the
full `#[wat_dispatch]` design and
`docs/arc/2026/04/013-external-wat-crates/INSCRIPTION.md` for the
external-crate architecture.

## `wat` binary

```
wat <entry.wat>      # run a program — INTERPRET path
```

Program mode reads the entry file, runs the full startup pipeline,
installs OS signal handlers, passes real stdio (wrapped in the
`:wat::io::IOReader` / `IOWriter` trait objects) to `:user::main`, and
exits.

A `wat test <path>` subcommand still ships as the pre-cargo-integration
workflow, but the canonical test path is `cargo test` through
`tests/test.rs` + `wat::test! {}` — Cargo is the authority. The CLI's
test mode discovers the same `test-`-prefixed zero-arg defines
returning `:wat::kernel::RunResult`.

### `:user::main` contract

```scheme
(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  ...body...)
```

Exact signature enforced at startup. Any deviation (different arity,
different parameter types, different return type) halts with exit code 3.

The program reads with `(:wat::io::IOReader/read-line stdin)` and writes
with `(:wat::io::IOWriter/print stdout msg)` /
`(:wat::io::IOWriter/println stdout msg)`. The IOReader / IOWriter trait
objects hide the backing — under the CLI it's real OS stdio (with std's
internal locking for thread safety); under `run-sandboxed` it's
`StringIo` stand-ins for in-memory testing.

Signals: the kernel measures; userland owns transitions.

- **Terminal (SIGINT, SIGTERM)** → set `stopped` irreversibly. Userland
  polls `(:wat::kernel::stopped?)` and cascades shutdown by dropping its
  root producers.
- **Non-terminal (SIGUSR1, SIGUSR2, SIGHUP)** → each flips its own
  kernel-maintained boolean. Userland polls via
  `(:wat::kernel::sigusr1?)` / `(sigusr2?)` / `(sighup?)` and clears via
  the matching `(:wat::kernel::reset-sigusr1!)` etc. Coalesced — five
  SIGHUPs in a burst read as one "yes".

### Exit codes (program mode)

| Code | Meaning |
|---|---|
| 0  | `:user::main` returned cleanly |
| 1  | Startup error (parse/config/load/macro/type/resolve/check) |
| 2  | Runtime error (channel disconnect, type mismatch, etc.) |
| 3  | `:user::main` signature mismatch |
| 64 | Usage error — wrong argv |
| 66 | Entry file read failed |

### Hello world

```scheme
;; echo.wat
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    ((Some line) (:wat::io::IOWriter/print stdout line))
    (:None ())))
```

```
$ echo watmin | wat echo.wat
watmin
```

## Self-hosted testing

Every test of the wat stdlib is written in wat, in the `wat-tests/`
directory, and runs through the test harness the stdlib itself defines.
The assertion primitives assert about the assertion primitives.

### Writing a test — `:wat::test::deftest`

```scheme
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::deftest :my::app::test-two-plus-two
  ()
  (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
```

The test file's top-level preamble commits capacity-mode + dims;
every `deftest` below inherits those values through the sandbox's
config-inheritance path (arc 031). `deftest` itself takes name +
prelude (loads / type declarations) + body. The `:user::main`
wrapper comes from the macro. Callers invoke the registered
function directly; the `wat::test!` runner auto-discovers by name
prefix + signature.

### Running tests — `cargo test`

```
$ cargo test -- --nocapture
running 1 test
running 31 tests
test stream.wat :: wat-tests::std::stream::test-chunks-exact-multiple ... ok (2ms)
test test.wat :: wat-tests::std::test::test-assert-eq-on-i64 ......... ok (1ms)
test service/Console.wat :: wat-tests::std::service::Console::test-hello-world ... ok (6ms)
...
test result: ok. 31 passed; 0 failed; finished in 107ms
test wat_suite ... ok
```

Recursive directory traversal. Random-ordered per file (nanos-seeded
xorshift64 inline — no `rand` dependency). Cargo-style output. The
outer `wat_suite` line is libtest's wrapper; per-wat-test lines stream
with `--nocapture` or print after the suite with `--show-output`.

### Fork/sandbox tests — `:wat::test::program` + `:wat::test::run-ast`

Tests that need to sandbox an inner program (to capture its stdout,
stderr, or failure) compose two stdlib forms:

```scheme
(:wat::test::deftest :my::test-captures-inner-output
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::core::define (:user::main ... -> :())
            (:wat::io::IOWriter/println stdout "hello-from-inside")))
        (:wat::core::vec :String)))
     ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r)))
    (:wat::test::assert-eq (:wat::core::first lines) "hello-from-inside")))
```

`:wat::test::program` is a variadic defmacro that expands to
`:wat::core::forms` (arc 010 — the variadic-quote substrate). Each
top-level form passes through as AST data. No escape backslashes.
Inner programs nest arbitrarily deep as pure s-expressions.

### Services that spawn threads — `:wat::test::run-hermetic-ast`

In-process `:wat::test::run` and `:wat::test::run-ast` back onto
`StringIo` stdio (ThreadOwnedCell-backed, single-thread). Services
like Console and Cache spawn driver threads that write to stdio;
writing from a spawned thread would panic the driver on the
thread-owner check. Those tests use `:wat::test::run-hermetic-ast`
— the wat stdlib wrapper over `:wat::kernel::run-sandboxed-hermetic-ast`,
which forks a child running fresh wat evaluation with fd-backed
stdio (`PipeReader` / `PipeWriter`, thread-safe by kernel
semantics).

Decision rule: **spawns-and-writes → hermetic; stays-on-main-thread → in-process**.

### Rust embedding — `wat::Harness`

For Rust programs that host wat as a sub-language:

```rust
use wat::Harness;

let h = Harness::from_source(src)?;
let out = h.run(&["stdin line 1", "stdin line 2"])?;
assert_eq!(out.stdout, vec!["captured".to_string()]);
```

Thin wrapper over `startup_from_source` + `invoke_user_main` + stdio
snapshot (arc 007 slice 5). Not a sandbox — no panic isolation; callers
that want that use `:wat::kernel::run-sandboxed` from inside their wat
code.

## Stdlib

Every file under `wat/std/` is baked into the binary at compile time via
`include_str!` and registered during startup before user code parses.
Consumers reference them by path — no explicit `load!` needed.

Algebra conveniences:
- `:wat::std::Amplify`, `:wat::std::Subtract`, `:wat::std::Log`,
  `:wat::std::Circular`, `:wat::std::Reject`, `:wat::std::Project`,
  `:wat::std::Sequential`, `:wat::std::Ngram` / `:wat::std::Bigram` /
  `:wat::std::Trigram`.

Streams (arcs 003 + 004 + 006):
- `:wat::std::stream::Stream<T>` typealias, `spawn-producer`,
  `from-receiver`, `map`, `filter`, `inspect`, `chunks` (rewritten on
  `with-state`), `flat-map`, `take`, `for-each`, `collect`, `fold`,
  `with-state` (the Mealy-machine substrate — every stateful stage
  reduces to an `(init, step, flush)` triple).

Test harness (arcs 007 + 010):
- `:wat::test::assert-eq`, `assert-contains`, `assert-stdout-is`,
  `assert-stderr-matches`, `run`, `run-in-scope`, `run-ast`, `deftest`,
  `program`.

Caches (external — `crates/wat-lru/`; arc 013 externalization):
- `:wat::lru::LocalCache<K,V>` — L1. Three thin wrappers
  over `:rust::lru::LruCache`. Single-thread-owned. Fastest memoization.
  Ships in the `wat-lru` sibling crate; consumers add `wat-lru =
  "..."` to `Cargo.toml` + `deps: [wat_lru]` to their `wat::main!`.
- `:wat::lru::CacheService<K,V>` — L2 shared cache.
  Driver thread owns its `LocalCache`; clients send tagged requests
  with an embedded reply channel. Also in `wat-lru`.

Services (long-running driver programs with client handles, baked):
- `:wat::std::service::Console` — the single gateway to stdout+stderr.
  Hands out pooled `Sender<(i64,String)>` via `:wat::kernel::HandlePool`;
  tag 0 = stdout, tag 1 = stderr.

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
| `:silent` | `Ok(h)` | `Ok(h)` — degraded vector, no diagnostic |
| `:warn`   | `Ok(h)` | `Ok(h)` — degraded vector + `eprintln!` cost/budget/dims |
| `:error`  | `Ok(h)` | `Err(CapacityExceeded { cost, budget })` |
| `:abort`  | `Ok(h)` | `panic!` with diagnostic — fail-closed |

**`:wat::algebra::CapacityExceeded`** is a built-in struct with
auto-generated `/cost` and `/budget` accessors.

**Canonical program shape** uses `:wat::core::try`:

```scheme
(:wat::core::define (:app::build
                    (items :Vec<holon::HolonAST>)
                    -> :Result<holon::HolonAST, wat::algebra::CapacityExceeded>)
  (Ok (:wat::core::try (:wat::algebra::Bundle items))))
```

Every stdlib macro whose expansion ends in Bundle inherits the Result
wrap — `:wat::std::Ngram`, `:wat::std::Bigram`, `:wat::std::Trigram`.
Callers match or `try` at the call site.

## Namespace discipline

Every type identifier in wat source shows its full namespace. Every
`Value` variant in the Rust enum encodes its path via `__` separator.
Errors read exactly like user-written declarations: *expected
`:rust::crossbeam_channel::Sender<String>`, got `:i64`*. No short names
hiding what a value is or where a type comes from.

Two sibling namespaces, both rooted at the colon:
- `:wat::*` — forms and types defined by the wat language itself.
  Sub-namespaces: `:wat::core::*` (evaluator primitives), `:wat::holon::*`
  (algebra + measurements), `:wat::kernel::*` (concurrency), `:wat::std::*`
  (stdlib), `:wat::config::*` (committed config), `:wat::verify::*`
  (verification vocabulary), `:wat::io::*` (stdio), `:wat::test::*` (test
  harness). Plus root-level substrate forms — `:wat::load-file!`,
  `:wat::eval-ast!`, etc. (arc 028) — and the `:wat::WatAST` type.
- `:rust::*` — forms and types surfaced from Rust crates
  (`:rust::std::io::*`, `:rust::crossbeam_channel::*`, `:rust::lru::*`,
  and whatever the consumer registered).

Full honesty rule in
`docs/arc/2026/04/002-rust-interop-macro/NAMESPACE-PRINCIPLE.md`.

## Workspace layout

```
wat-rs/
├── Cargo.toml              # workspace + wat package
├── src/
│   ├── lib.rs              # extern crate self as wat;
│   ├── bin/wat.rs          # CLI binary (program mode + test subcommand)
│   ├── {lexer,parser,config,load,identifier,macros,
│   │    types,resolve,check,hash,lower,runtime,
│   │    freeze,stdlib,source,io,string_ops,assertion,
│   │    sandbox,harness,compose,fork,test_runner}.rs
│   └── rust_deps/
│       ├── mod.rs          # RustDepsBuilder, Registry, SchemeCtx,
│       │                   # UseDeclarations, install(), get()
│       └── marshal.rs      # FromWat/ToWat, ThreadOwnedCell,
│                           # OwnedMoveCell, RustOpaqueInner
├── wat-macros/             # sibling proc-macro crate
│   └── src/{lib.rs,codegen.rs}  # #[wat_dispatch] + wat::main! + wat::test_suite!
├── wat/std/                # baked-in wat source files
│   ├── Amplify.wat Subtract.wat Log.wat Circular.wat
│   ├── Reject.wat Project.wat Sequential.wat
│   ├── Ngram.wat Bigram.wat Trigram.wat
│   ├── stream.wat hermetic.wat test.wat
│   └── service/
│       └── Console.wat     # Cache moved to wat-lru (arc 013)
├── crates/wat-lru/         # external wat crate — LRU surface (arc 013)
│   ├── Cargo.toml          # depends on wat + wat-macros + lru
│   ├── src/{lib.rs,shim.rs}  # wat_sources(), register(), #[wat_dispatch] impl
│   ├── wat/{LocalCache,CacheService}.wat
│   ├── wat-tests/{LocalCache,CacheService}.wat  # deftests
│   └── tests/wat_suite.rs  # one-line wat::test_suite!
├── examples/with-lru/      # reference consumer binary (arc 013 slice 5)
│   ├── Cargo.toml
│   ├── src/{main.rs,program.wat}  # main.rs is one wat::main!
│   └── tests/smoke.rs      # spawns the binary, asserts "hit"
├── wat-tests/              # wat-rs's own baked-stdlib tests
│   ├── README.md
│   └── std/
│       ├── {Subtract,Circular,Reject,Sequential,Trigram,test,stream}.wat
│       └── service/Console.wat
├── tests/                  # Rust integration suites
│   ├── mvp_end_to_end.rs
│   ├── wat_dispatch_{193a,193b,e1_vec,e2_tuple,
│   │                 e3_result,e4_shared,e5_owned_move}.rs
│   ├── wat_core_try.rs wat_structs.rs wat_bundle_capacity.rs
│   ├── wat_stream.rs wat_core_forms.rs wat_names_are_values.rs
│   ├── wat_harness.rs wat_harness_deps.rs
│   ├── wat_run_sandboxed{,_ast}.rs wat_hermetic_round_trip.rs
│   ├── wat_test_cli.rs wat_cli.rs wat_io.rs wat_u8.rs wat_fork.rs
│   └── ...
└── docs/
    ├── README.md           # orientation
    ├── USER-GUIDE.md       # building on wat (wat::main! + wat::test_suite!)
    ├── CONVENTIONS.md      # naming + folder layouts + three varieties
    ├── ZERO-MUTEX.md       # the concurrency architecture
    └── arc/2026/04/
        ├── 001-caching-stack/              # DESIGN + DEADLOCK-POSTMORTEM
        ├── 002-rust-interop-macro/         # MACRO-DESIGN + NAMESPACE-PRINCIPLE
        ├── 003-tail-call-optimization/     # DESIGN + INSCRIPTION
        ├── 004-lazy-sequences-and-pipelines/ # DESIGN + INSCRIPTION + BACKLOG
        ├── 005-stdlib-naming-audit/        # DESIGN + INVENTORY + INSCRIPTION
        ├── 006-stream-stdlib-completions/  # BACKLOG + INSCRIPTION
        ├── 007-wat-tests-wat/              # DESIGN + BACKLOG + INSCRIPTION
        ├── 008-wat-io-substrate/           # DESIGN + BACKLOG + INSCRIPTION
        ├── 009-names-are-values/           # BACKLOG + INSCRIPTION
        ├── 010-variadic-quote/             # BACKLOG + INSCRIPTION
        ├── 011-hermetic-ast/               # DESIGN + BACKLOG + INSCRIPTION
        ├── 012-fork-and-pipes/             # DESIGN + BACKLOG + INSCRIPTION
        ├── 013-external-wat-crates/        # DESIGN + BACKLOG + INSCRIPTION
        ├── 014-core-scalar-conversions/    # DESIGN + BACKLOG + INSCRIPTION
        ├── 015-wat-test-for-consumers/     # DESIGN + BACKLOG + INSCRIPTION
        ├── 016-failure-location-and-frames/ # DESIGN + BACKLOG + INSCRIPTION
        ├── 017-loader-option-for-consumer-macros/ # DESIGN + BACKLOG + INSCRIPTION
        └── 018-opinionated-defaults-and-test-rename/ # DESIGN + BACKLOG + INSCRIPTION
```

## What's next

The substrate is complete for its stated bar. Further work is
caller-demanded per `stdlib-as-blueprint` discipline:

- **More consumer shims.** holon-lab-trading needs `:rust::rusqlite::`
  for the candle DB and `:rust::parquet::` for archive reads. Each
  follows the `#[wat_dispatch]` pattern.
- **Stream library follow-ups.** Arc 006 still holds open `chunks-by`,
  `window`, `dedupe`, `sessionize`, `time-window`, and `from-iterator`
  / Level 2 iterator surfacing. Each ships as library code on
  `with-state` when a concrete caller cites use.
- **Arc 007 follow-ups.** `Failure.location` + `.frames` population
  shipped via arc 016 (2026-04-21). Different implementation than
  the original sketch: wat-level call stack + wat-source spans on
  every AST node, not `std::backtrace::Backtrace::capture()`. Panic
  hook renders `cargo test`-shaped output gated on `RUST_BACKTRACE`.
  Still open from arc 007: parallel test execution, `:rust::*`
  capability allowlist, richer assertion payloads via generic
  `show<T>`. Each waits for demand.
- **UX pass on the trading lab.** When holon-lab-trading rewrites its
  wat programs against this crate, every ceremonial shape it hits is
  a candidate for a substrate follow-up — the same way arc 010's
  variadic-quote fell out of writing fork/sandbox tests.

Signature verification is **per-form, not per-invocation.** It lives at
`:wat::signed-load!` (startup) and `:wat::eval-signed!`
(runtime). A program may invoke any number of either, each with its own
key and signature. There is no `wat --signed` / `--sig` / `--pubkey`
CLI flag; a program's verification surface is its collection of
`signed-*` forms. See FOUNDATION's cryptographic-provenance section.

## See also

- `../holon-rs/src/kernel/holon_ast.rs` — the algebra-core AST this
  crate evaluates against.
- `../holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
  — the language specification.
- `../holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/INDEX.md`
  — reading guide for the 058 batch.
- `../holon-lab-trading/BOOK.md` — the story.
- `docs/README.md` — arc index + USER-GUIDE, CONVENTIONS, ZERO-MUTEX
  pointers.
- `docs/USER-GUIDE.md` — building applications on wat.
- `docs/CONVENTIONS.md` — naming rules for new primitives.
