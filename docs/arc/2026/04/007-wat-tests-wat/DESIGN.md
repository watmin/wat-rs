# wat tests wat — self-hosted testing + filesystem sandbox

**Status:** planned. Opened 2026-04-20 as the detour the builder wanted
to show; scoped 2026-04-21.
**Depends on:** arc 003 (TCO) and arc 004 (streams) — shipped.
**Blocks:** arc 006 slice 4 (with-state) — implementing with-state
wants wat-native tests; wat-native tests are THIS arc.
**Motivation:** if a user has to drop into Rust to test their wat
program, the language is incomplete. If wat can test wat, the
language is complete-for-its-own-verification. Every self-hosted
language reaches this moment — Smalltalk tests Smalltalk, Lisp
tests Lisp. The test harness is the first piece of self-reflection.

---

## What this arc ships

Five slices, ordered:

1. **Slice 1 — ScopedLoader + file-I/O audit.** The capability gate
   for sandboxed execution. Covered in detail below.
2. **Slice 2 — `:wat::kernel::run-sandboxed`.** The primitive that
   runs wat source inside a fresh frozen world with captured
   stdio and a scoped loader.
3. **Slice 3 — `:wat::test::*` stdlib.** `assert-eq`, `assert-
   contains`, `assert-stdout-is`, etc. Pure wat, built on
   run-sandboxed.
4. **Slice 4 — `wat-vm test` CLI subcommand.** Discovers `.wat`
   test files, runs each, reports pass/fail/time.
5. **Slice 5 — `wat::Harness` Rust API.** Embedder surface — for
   Rust programs that host wat as a sub-language. Reuses a frozen
   world across many invocations.

---

## Slice 1 — ScopedLoader + file-I/O audit

The first move. Everything else depends on it.

### Why this is slice 1

Chapter 16 of BOOK.md named the bypass explicitly:

> No sandboxing through a trait indirection (yet). The source form
> declares the capability: `:wat::eval::file-path "received.wat"`
> *says* this is a file read at eval time. Future work can gate it
> via a `SourceLoader`-style indirection. For now: the discipline
> is announced, not hidden.

Today wat has one announced capability (file I/O at eval time) and
zero gates on it. A sandboxed `run-sandboxed` that lets the inner
program `load!` `"../../../etc/passwd"` is a security hole by
design. The capability discipline wat has always claimed needs to
become operational before sandboxed execution ships.

### What "chroot" means in wat-rs terms

Not literal OS chroot — that requires kernel cooperation and
privileges we don't have. Loader-based path isolation at the
language layer. The Loader trait is the capability; the sandbox
gets a scoped Loader it can't escape.

Concretely:

1. **Audit every wat-rs callsite that touches `std::fs::*`.** The
   loader abstraction (`InMemoryLoader`, `FileLoader`) should mediate
   every file read. `freeze.rs::resolve_loads` already goes through
   the loader. `runtime.rs::eval_form_edn` (for `:wat::eval::file-
   path`) is the suspect bypass per Chapter 16. Any direct
   `std::fs::read_to_string` outside the loader becomes a bug to
   fix.

2. **Ship `ScopedLoader`.** New concrete Loader impl:
   ```rust
   pub struct ScopedLoader { root: PathBuf }
   impl Loader for ScopedLoader {
       fn load(&self, path: &str) -> Result<String, LoadError> {
           let candidate = self.root.join(path);
           let canonical = candidate.canonicalize()?;
           if !canonical.starts_with(&self.root) {
               return Err(LoadError::OutOfScope);
           }
           std::fs::read_to_string(canonical)
       }
   }
   ```
   Canonicalization handles symlink escape. The `starts_with` check
   handles `../` escape. Absolute paths get clamped or rejected.
   One type, ~40 lines of Rust.

3. **Make it the default for sandboxed runs.** Slice 2's
   `run-sandboxed` accepts either a caller-provided scope path
   (ScopedLoader) or no path (InMemoryLoader only — maximum
   isolation). No way for the caller to pass an unrestricted
   FileLoader into a sandboxed context. The outer CLI still uses
   FileLoader at the top; only the sandboxed inner world is
   constrained.

4. **Document the capability boundary in USER-GUIDE.** "Wat's
   file-I/O capability IS its loader. The host chooses the loader.
   A sandboxed program's loader is its ceiling." Matches Chapter 16's
   voice about announced capabilities.

### Why loader-on-SymbolTable — convergence with prior art

Before committing to this shape, a check against the inspiration
languages. Every serious language faces the same problem: how does
a runtime primitive access startup-bound capabilities (I/O handles,
loaders, etc.) at dispatch? Names differ; the shape is the same.

- **Common Lisp.** Special variables `*standard-input*`,
  `*standard-output*`, `*error-output*`, `*readtable*`, `*package*`
  — bound at image startup, accessed by `write-char` / `read` at
  runtime.
- **Scheme.** R7RS parameter objects `current-input-port`,
  `current-output-port`, `current-error-port` via `make-parameter`
  + `parameterize`. Plus first-class environments historically
  carrying eval-time capabilities (SICP's environment model).
- **Clojure.** Dynamic vars `*out*`, `*in*`, `*err*`,
  `*compile-path*`, `*print-length*`. `*out*` bound at JVM startup
  to `System/out`; `println` reads it at dispatch. The trading
  lab's Hickey lineage already put wat in this line.
- **Rust.** The compiler's `rustc_interface::Compiler` holds a
  `Session` carrying `SourceMap`, `diagnostic_emitter`, `file_loader`,
  `target_spec` — startup-bound, passed down through compiler
  passes, accessed by subsystems at dispatch. `SymbolTable.source_loader`
  mirrors this almost 1:1, down to the name. Closest precedent.
  Also: Tokio's `Handle::current()`, `std::thread::LocalKey`.
- **Ruby.** `$stdin`, `$stdout`, `$stderr`, `ENV`, `$LOAD_PATH` —
  startup-bound globals used by `require` / `load` / `puts`.
  `Kernel#require` reads `$LOAD_PATH`.
- **Haskell.** `ReaderT env IO a` — the dominant production
  pattern. `env` is a record built at `main` holding DB pools,
  loggers, filesystem access. Primitives access via `ask` / `asks`.
  Plus `System.IO`'s runtime-bound `stdin` / `stdout` / `stderr`.
- **Agda.** Evaluator calls into a backend-provided primitive
  table. `IO.Primitive.readFile` is program-facing; the compiler
  backend supplies the implementation. Most abstract version of
  the same shape.

**The shape is universal because it's what the substrate permits.**
A language that needs I/O primitives must carry SOMETHING
representing "how to do I/O" from program entry to where primitives
execute. Dynamic scope / monad threading / globals / startup
contexts / backend dispatch — all valid instantiations of the same
structural answer. Wat picks the startup-context-structure variant
because `FrozenWorld` already IS one, and `SymbolTable.encoding_ctx`
established the pattern one capability ago.

Second convergence this session (the first was `with-state`
matching Mealy 1955 / Elixir `Stream.transform/3` / Rust
scan-with-emit / Haskell `mapAccumL`). Same signal: when the right
shape for a problem is universal across languages with otherwise-
different philosophies, it's because the shape is what the
substrate permits — not a style choice.

### What sandbox isolation does NOT guarantee

Arc 007 gates **wat-level** capabilities. It gates filesystem via
the Loader (slice 1) and will gate signal-mutation via privileged
primitives (slice 2). It does NOT — cannot — isolate Rust-level
process state shared across sandboxes running in the same Rust
process.

Specifically **not isolated by this sandbox**:

- **Rust `static` / `lazy_static` / `OnceLock` state** in wat-rs
  itself or in any crate behind a `#[wat_dispatch]` shim. If a
  shim method touches `static COUNTER: AtomicU64`, sandboxed wat
  can increment it; the next sandbox sees the incremented value.
- **Thread pools, connection pools, caches** held in any crate
  the host registered via `#[wat_dispatch]`. The shim surface is
  a dispatcher; the backing state is process-global.
- **Environment variables** set via `std::env::set_var` through
  any shim that exposes it. Process-global.
- **File handles, sockets, memory-mapped files** held in shim-
  internal registries or global collections.
- **Thread-local state set by wat code.** Wat's own kernel runs
  in multiple threads; between `:wat::kernel::spawn` calls,
  thread-locals set by one sandbox's worker could be observed
  by the next.

**This is the same model `cargo test` has.** Tests run in one
process, share `static` state, and depend on the honor system
for hermeticity. Rust's ecosystem response is `#[serial]`,
per-test teardown, and subprocess test runners like `trybuild` —
language conventions, not language features. Wat inherits the
model.

**Full process-level isolation requires subprocess-per-test.**
That's a named future arc — see "Scaffolding for hermetic-mode"
below. Until it lands, wat-test hosts whose shims have mutable
process-global state must either (a) not use them in tests that
need isolation, (b) tear down the state explicitly between
tests, or (c) wait for the hermetic arc.

### Deferred to its own arc — `:rust::*` capability allowlist

Even more specific: a sandboxed test can today call any
`#[wat_dispatch]`-registered shim the host provides. A test
that `use!`s `:rust::std::net::TcpStream` makes a network
connection. A test that `use!`s `:rust::std::process::Command`
forks a process. Network/process capability isolation would
require a per-world allowlist of `:rust::*` types, enforced at
`use!` time. Not arc 007 scope. Deferred until a real caller
demands it; the pattern would be host-provides-allowlist + the
frozen world refuses `use!` of paths outside the list.

Even with that allowlist, process-state leakage (above) still
stands. The allowlist is about reducing *attack surface*;
hermetic is about eliminating *state leakage*. Different
concerns, different arcs.

### Scaffolding for hermetic-mode (future arc)

Arc 007 ships in-process testing. A future arc will add
subprocess-per-test isolation under `wat-vm test --hermetic`.
To make that arc a clean extension rather than a breaking
change, arc 007 bakes four scaffolding decisions into its
deliverables:

1. **Serializable `TestResult` shape.** Every test produces a
   structure that has a stable, serializable representation —
   either a derive-based serde shape or a documented JSON
   layout. The in-process runner builds these directly; the
   hermetic runner will deserialize them from a child
   subprocess's stdout. Fields pinned now: `{ name, passed: bool,
   elapsed_ms, failure: Option<{ message, location? }> }`.
   Slice 3 defines. Slice 4 uses.

2. **Single-test addressability.** Every test has a stable ID.
   In v1 (one-file-is-one-test): ID = file path. Future case-
   level granularity extends to `<file-path>::<case-name>`. The
   CLI contract supports `--run-one <id>` (not implemented in
   arc 007 but reserved).

3. **`wat-vm test` CLI contract** leaves room for future flags
   without breaking the in-process default:
   - `wat-vm test <path>` — run all tests under path, in-process,
     cargo-test-style report. (Arc 007 ships this.)
   - `wat-vm test --hermetic <path>` — subprocess per test.
     (Future arc.)
   - `wat-vm test --run-one <id>` — single-test subprocess entry
     point. Emits a single `TestResult` JSON to stdout.
     (Future arc; the hermetic runner uses this.)
   Choose flag names now so future code doesn't collide with v1.

4. **Exit-code parallelism.** Aggregate runner exits 0 iff all
   tests passed, non-zero if any failed. Single-test runner (when
   implemented) exits 0 iff the test passed, non-zero if failed.
   Parallel semantics between modes means a hermetic child's
   exit code IS the test's pass/fail signal.

These cost almost nothing to bake in during slice 3/4. Deferring
them risks breaking-change churn when hermetic lands.

**The hermetic arc itself is NOT this arc.** When it lands:
- Add `--hermetic` flag and `--run-one <id>` entry point
- Child emits `TestResult` as JSON on stdout; parent parses
- Startup cost: ~10-50ms/test; acceptable for CI where isolation
  matters more than per-test speed
- In-process mode stays default — fast, matches cargo-test
  semantics, the 90% case

Stdlib-as-blueprint on sandbox capabilities too — each capability
gate, and the subprocess-runner mode itself, ships when a
concrete caller demands.

---

## Slice 2 — `:wat::kernel::run-sandboxed`

The primitive that makes self-testing operational.

### Signature

```
(:wat::kernel::run-sandboxed
  (src :String)
  (stdin :Vec<String>)
  (scope :Option<String>)
  -> :RunResult)
```

Where:
- `src` — wat source text to evaluate.
- `stdin` — pre-seeded stdin lines; injected into the sandboxed
  `rx_stdin` channel before `:user::main` is invoked; tx drops
  at end of injection → EOF visible to the sandboxed program.
- `scope` — optional filesystem root path. `:Some "path"` creates
  a `ScopedLoader`; `:None` creates an `InMemoryLoader` with no
  disk access at all.
- `RunResult` — struct `{ returned :holon::HolonAST, stdout
  :Vec<String>, stderr :Vec<String> }`.

### Non-obvious implementation details

Each worth flagging explicitly because they're easy to miss:

1. **Panic isolation.** `std::panic::catch_unwind` around the
   inner `invoke_user_main`. A test's `:user::main` that panics
   (or an assertion that panics, per slice 3) must surface as a
   failed RunResult, not take down the outer wat program. Without
   catch_unwind, the host dies with the test.

2. **Shutdown wait.** If the sandboxed program `spawn`s sub-
   programs via `:wat::kernel::spawn` and those don't exit before
   main returns, they leak into the outer process. The CLI
   handles this via drain + join after main. `run-sandboxed`
   needs the same discipline — wait for all spawned work to
   drain before returning RunResult.

3. **Signal state bleeds.** `:wat::kernel::stopped?` and the
   SIGUSR1/2/HUP flags are process-global. A sandboxed test
   observes the outer process's signals. Usually fine (tests are
   fast), but timing-sensitive tests could flake if the outer
   wat-vm gets signaled mid-test. Documented in USER-GUIDE;
   not fixable cheaply.

4. **Main-signature: strict three-channel.** `run-sandboxed`
   enforces the `:user::main` three-channel contract. Tests
   exercise the real invocation shape. Users who want no-channel
   main have `eval-edn!` already — two paths, two concerns.

5. **Loader scope.** Covered in slice 1. The sandbox receives a
   loader it can't escape.

---

## Slice 3 — `:wat::test::*` stdlib

Pure wat. No new runtime support beyond one thing: a new
`RuntimeError::AssertionFailed { message, actual, expected,
location }` variant that flows through the standard error path
and gets caught by `run-sandboxed`'s panic isolation.

### Assertion mechanism — panic-and-catch

Assertions panic with a structured payload. `run-sandboxed`'s
`catch_unwind` surfaces the payload into RunResult. Clean
call-site syntax (no match ceremony per assertion). Matches what
Ruby, Python, and Rust test frameworks do at the user surface.

Alternative considered: assertions return `Result<(), AssertError>`;
users `try` or match. More composable but every assertion adds
ceremony. Rejected on the same "verbose is honest" grounds as
other language additions — except here the ceremony is the
*un*honest path because it puts weight on every test.

### Surface (first cut)

- `:wat::test::assert-eq a b` — compares via structural equality.
- `:wat::test::assert-contains haystack needle` — substring or
  element containment depending on type.
- `:wat::test::assert-stdout-is result expected-lines` — unwraps
  a RunResult and compares stdout lines.
- `:wat::test::assert-stderr-matches result pattern` — regex
  match on stderr.
- `:wat::test::run src stdin-lines` — thin wrapper over
  `run-sandboxed` with in-memory loader (no filesystem) as the
  common case.
- `:wat::test::run-in-scope src stdin-lines scope` — when the
  test needs filesystem fixtures.

Higher-level forms (`suite`, `test-case`) deferred until usage
patterns surface.

---

## Slice 4 — `wat-vm test` CLI subcommand

The test runner. Discovers `.wat` files, runs each, reports.

### Discovery convention

Any `.wat` file whose `:user::main` invokes `:wat::test::*`
forms IS a test. No special marker form needed. The import is
the discovery.

### Invocation

```
$ wat-vm test tests/
running 12 tests
test tests/hello.wat ... ok (3ms)
test tests/pipeline.wat ... ok (12ms)
test tests/broken.wat ... FAILED (5ms)

failures:

    tests/broken.wat — assert-eq failed at line 42
      expected: "hello"
      actual:   "hullo"

test result: FAILED. 11 passed; 1 failed; finished in 47ms
```

Report format modeled on Rust's `cargo test` output. Exit code
0 on all-pass, non-zero on any failure.

### Parallel execution

Deferred. Tests that use filesystem scopes or spawn threads may
collide if run in parallel without careful isolation. V1 runs
serial; parallelism is a follow-up once usage patterns expose
which tests can safely run concurrently.

---

## Slice 5 — `wat::Harness` Rust API

For Rust programs embedding wat. Freeze-once, invoke-many.

```rust
pub struct Harness { world: FrozenWorld }
pub struct RunResult {
    pub returned: Value,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}
impl Harness {
    pub fn from_source(src: &str) -> Result<Self, StartupError>;
    pub fn from_source_scoped(src: &str, scope: &Path) -> Result<Self, StartupError>;
    pub fn run(&self, stdin: &[&str]) -> Result<RunResult, RuntimeError>;
}
```

Re-exported as `wat::harness::{Harness, RunResult}` from crate
root.

Serves embedders (labs running user-scripted strategies, test
runners written in Rust that need low overhead, future
development tooling). Tests written in wat use slice 3 + slice 4;
this is the Rust-side mirror for Rust-side consumers.

---

## Out of scope for this arc

- **`:rust::*` capability isolation.** Sandbox that fully seals
  network + process access needs a per-world allowlist. Own arc
  when demanded.
- **Parallel test execution.** See slice 4.
- **Coverage / instrumentation.** Out of scope; language
  substrate first, tooling layer later.
- **Property-based testing.** Would live in `:wat::test::*` as
  a higher-level form once the base is stable.
- **Mocking / stubbing `:rust::*` types.** Probably handled by
  the capability-allowlist arc when it lands.

---

## The thesis

If wat can test wat, the language is complete-for-its-own-
verification. That's the proof. Chapter 20 of BOOK.md named
the convergence moment — finding the same shapes the greats
found. This arc closes an older loop: a language that has
always been able to run programs now can verify them in itself.
Programs are thoughts. Tests are thoughts about programs.
Both live in the same algebra.

The first inscription of the arc will be a `.wat` test file
whose `:user::main` runs a sandboxed wat program and asserts
against its RunResult. That file is the proof point.
