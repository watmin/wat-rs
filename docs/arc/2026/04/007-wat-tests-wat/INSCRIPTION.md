# Arc 007 — wat tests wat — INSCRIPTION

**Status:** shipped 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — pre-ship intent + decision
record.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — narrative of the slices,
commit refs, decision log.
**This file:** completion marker.

Same inscription discipline as arcs 003 / 004 / 005 / 006 / 008:
DESIGN was the intent; BACKLOG is the narrative; this INSCRIPTION is
the shipped contract. If DESIGN and INSCRIPTION disagree, INSCRIPTION
wins.

---

## What shipped

Eight slices across two sessions. Arc 007 paused mid-flight for arc
008 (the `:wat::io::IOReader` / `IOWriter` substrate), then resumed
and carried through to the test-runner CLI + Rust Harness + a full
stdlib-tests migration into wat.

### Slice 1 — ScopedLoader + file-I/O audit (five sub-commits)

The capability gate for sandboxed execution. Chapter 16 of BOOK.md
had announced wat's filesystem capability without gating it; slice 1
operationalized the discipline.

- **1a.** `SourceLoader` trait + `source_loader` field on `SymbolTable`.
- **1b.** `:wat::eval::file-path` routes through the loader instead of
  `std::fs::read_to_string` directly.
- **1c.** `:wat::verify::file-path` same treatment.
- **1d.** `ScopedLoader` impl + `LoadFetchError::OutOfScope`. Canonicalize
  the candidate path; refuse any path that doesn't start with the
  scope root. Handles `../` escape, symlink escape, absolute-path
  attempts. ~40 lines of Rust.
- **1e.** USER-GUIDE section on the capability boundary — "wat's
  file-I/O capability IS its loader."

Convergence discovery recorded in DESIGN.md: loader-on-SymbolTable
mirrors Common Lisp's `*special*` variables, Scheme parameter
objects, Clojure dynamic vars, Rust's `rustc_interface::Session`,
Ruby globals, Haskell's `ReaderT`, and Agda's backend table. Same
shape in every language with startup-bound I/O capabilities. Wat
joined the line.

### Slice 2a — `:wat::kernel::run-sandboxed` happy path (`647e2b7`)

First sandbox primitive. Accepts `(src, stdin, scope)`, returns
`:wat::kernel::RunResult { stdout, stderr }`. `:None` scope gives
`InMemoryLoader` (zero disk); `:Some path` gives `ScopedLoader`. The
sandboxed `:user::main` runs with `StringIoReader`/`StringIoWriter`
stand-ins wrapped in the arc-008 trait objects; same wat source runs
in production and test. The Ruby StringIO model made operational.

### Slice 2b — isolation + structured Failure (`a6302c0`)

`std::panic::catch_unwind` around `invoke_user_main`. Panics in the
sandboxed program no longer take down the outer wat; they land in
`RunResult.failure` as a `:wat::kernel::Failure` struct:

```
struct Failure {
  message  :String
  location :Option<:wat::kernel::Location>
  frames   :Vec<:wat::kernel::Frame>
  actual   :Option<:String>    ; populated by slice 3's assertions
  expected :Option<:String>    ; populated by slice 3's assertions
}
```

Three downcast paths: `AssertionPayload` (slice 3's contract),
`&'static str` (bare `panic!()`), `String` (`panic!("{}", ...)`).
Location/frames stubbed — future slice installs a panic hook +
`Backtrace::capture`.

### Slice 2c — hermetic subprocess sandbox (`5bd73c8`)

**Brought forward from a future arc.** Originally `--hermetic` was
reserved as a slice-4 CLI flag behind a separate future arc for
"process-level isolation from Rust-runtime state." The SIGUSR
subprocess-isolation pattern already in the signal tests made the
implementation cheap — the result shipped as a sibling primitive
rather than a mode flag:

```
(:wat::kernel::run-sandboxed-hermetic src stdin scope)
  → :wat::kernel::RunResult
```

Spawns `wat` as a subprocess, writes source to a tempfile, pipes
stdin, captures stdout/stderr, translates exit code into failure.
`WAT_HERMETIC_BINARY` env var picks the binary; falls back to
`std::env::current_exe()`.

Round-trip integration test proves:

```
(eval-edn! (first (stdout (run-sandboxed-hermetic "(print (+ 40 2))"))))
  → 42
```

**wat generates wat, wat runs wat, wat evaluates wat's output.**

Also fixed a double-colon bug in `types.rs` builtin struct
registrations — same class as the arc-008 slice-2 finding. `head:
":Vec"` produced `"::Vec<...>"` in error messages; bare `"Vec"` is
correct.

### Slice 3 — `:wat::test::*` stdlib + assertion mechanism (`f03d821`)

One new kernel primitive + one stdlib file + one `RuntimeError`
variant.

- **`:wat::kernel::assertion-failed!`** — takes
  `(message :String, actual :Option<String>, expected :Option<String>)`.
  `panic_any(AssertionPayload { ... })`; the sandbox's downcast chain
  populates `Failure.actual` / `Failure.expected` directly.
- **`RuntimeError::AssertionFailed` variant** — surfaces via the
  non-panic path when `assertion-failed!` is invoked without a
  sandbox to catch it (user runs an assertion-containing wat program
  directly).
- **`wat/std/test.wat`** — six forms:
  - `:wat::test::assert-eq<T>`
  - `:wat::test::assert-contains`
  - `:wat::test::assert-stdout-is`
  - `:wat::test::assert-stderr-matches`
  - `:wat::test::run src stdin` — wraps `run-sandboxed`, `:None` scope
  - `:wat::test::run-in-scope src stdin path` — wraps with ScopedLoader

**Substrate precursor.** Slice 3's needs forced two new substrate
additions that went in first (commit `97a1ec5`):
- `:wat::core::string::*` — seven char-oriented primitives (`contains?`,
  `starts-with?`, `ends-with?`, `length`, `trim`, `split`, `join`).
- `:wat::core::regex::matches?` — pulled in the `regex` crate as a
  dep.

**And one surfaced gap.** Slice 3's `assert-stdout-is` compares two
`:Vec<String>` values; the old primitive-only `:wat::core::=` had no
case for Vec. Extended to structural equality across Vec, Tuple,
Option, Result, Struct. `values_equal` returns `Option<bool>`: `Some`
when comparable, `None` when meaningfully incomparable (e.g.,
`Function` vs. anything). `<`, `>`, `<=`, `>=` stay primitive-only —
no canonical ordering for composite values worth inventing.

### Slice 3b — AST-entry sandbox + `deftest` (`5c74eef`)

**Opened mid-arc.** Slice 3 shipped `:wat::test::run` as a string-
accepting wrapper; writing tests required the full `:user::main` +
config-setter scaffolding in every invocation. The obvious next move
was a Clojure-style `deftest` macro, but the sandbox's
string-accepting entry would have forced the macro to serialize its
quoted body back to source — the honest-but-wasteful round-trip
`/temper` would flag.

Fix: split `startup_from_source` at the parse boundary.

- **`startup_from_forms(forms: Vec<WatAST>, ...)` public** — runs
  steps 2–9 of the pipeline. `startup_from_source` becomes
  `parse_all + startup_from_forms`.
- **`:wat::kernel::run-sandboxed-ast`** — same semantics as
  `run-sandboxed` but first arg is `:Vec<wat::WatAST>`. Skips the
  parse + re-serialize round trip when the caller already has AST.
- **`:wat::test::deftest`** — defmacro in `wat/std/test.wat`:

  ```
  (:wat::test::deftest :my::app::test-two-plus-two 1024 :error
    (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
  ```

  Expands to a named zero-arg function returning `RunResult`. Config
  setters + `:user::main` wrapper come from the macro; callers write
  only the name, dims, mode, and body.

Doors opened: dynamically-generated tests (property-based, fuzzers,
template expansion) hand a `Vec<AST<()>>` to `run-sandboxed-ast`
directly. Any future compiler pass with AST in hand composes with
the sandbox without a serialize layer.

### Slice 4 — `wat test <path>` CLI subcommand (`92042be`)

Two invocation shapes on one binary:

```
wat <entry.wat>      # program mode — original
wat test <path>      # test mode — new; file or directory
```

**Discovery.** Any top-level `:wat::core::define` where (1) the
path's final `::`-segment starts with `test-` AND (2) the signature
is `() -> :wat::kernel::RunResult`. Dual filter — name prefix +
signature — so a helper function named `test-data` returning a
`String` isn't accidentally invoked.

**Random order.** Per-file Fisher-Yates shuffle with a nanos-seeded
xorshift64. No `rand` crate dep — ~15 lines inline. Random order
surfaces accidental inter-test dependencies.

**Output.** Cargo-test-style:

```
running 24 tests
test test.wat :: my::test-foo ............ ok (2ms)
test test.wat :: my::test-bar ............ FAILED (3ms)
  failure: assert-eq failed

test result: FAILED. 23 passed; 1 failed; finished in 79ms
```

**Dropped from the original DESIGN:** `--hermetic` and `--run-one`
flags. Userland `:wat::kernel::run-sandboxed-hermetic` covers the
isolation use case as a wat-visible primitive, not a CLI mode. The
scaffolding-for-hermetic-mode section of DESIGN is retired.

### Slice 5 — `wat::Harness` Rust embedding wrapper (`f0b1d1c`)

**Reframed.** Originally slated as a `Harness` architecture with
`Outcome { returned, stdout, stderr }` and multiple construction
paths. Ended up as ~130 lines of Rust that captures the
`startup_from_source + StringIo + invoke_user_main + snapshot`
pattern the integration tests had hand-rolled in every file.

```rust
pub struct Harness { world: FrozenWorld }
pub struct Outcome { pub stdout: Vec<String>, pub stderr: Vec<String> }
pub enum HarnessError { Startup | MainSignature | Runtime | StdioSnapshot }

Harness::from_source(src)                      // InMemoryLoader
Harness::from_source_with_loader(src, loader)  // caller-chosen
h.run(&["stdin lines"]) -> Result<Outcome>
h.world() -> &FrozenWorld
```

Re-exported at crate root. Not a sandbox (no panic isolation), not a
test runner (`wat test` is that), not a freeze-many (FrozenWorld is
already `&self`-shareable). Just the boilerplate captured.

---

## Migrations

### `wat-vm` → `wat` (commit `2bbf0ae`)

Binary renamed. 29 files touched (source, tests, docs, arc records,
stdlib comments). Mechanical sed pass. File renames:

```
src/bin/wat-vm.rs       → src/bin/wat.rs
tests/wat_vm_cli.rs     → tests/wat_cli.rs
tests/wat_vm_cache.rs   → tests/wat_cache.rs
```

The lab repo (`holon-lab-trading/wat-vm.sh`) still references the
old name — that's its concern; this repo is cleanly renamed.

### Stdlib tests: Rust → wat-tests/ (commits `cb8e7fa`, `dc25693`, `d258581`)

Dogfooding move: every test of `wat/std/*` now lives in wat, tested
by the very harness the stdlib defines. Layout mirrors one-to-one:

```
wat/std/Subtract.wat         ↔ wat-tests/std/Subtract.wat
wat/std/Circular.wat         ↔ wat-tests/std/Circular.wat
wat/std/Reject.wat           ↔ wat-tests/std/Reject.wat       (tests Project too)
wat/std/Sequential.wat       ↔ wat-tests/std/Sequential.wat
wat/std/Trigram.wat          ↔ wat-tests/std/Trigram.wat
wat/std/test.wat             ↔ wat-tests/std/test.wat
wat/std/service/Console.wat  ↔ wat-tests/std/service/Console.wat
wat/std/service/Cache.wat    ↔ wat-tests/std/service/Cache.wat
```

`wat test wat-tests/` recurses the tree; 24 tests in 107ms. Dropped
`tests/wat_deftest.rs`, `tests/wat_test_stdlib.rs`,
`tests/wat_cache.rs`, + five algebra stdlib tests and two Console
tests inside `wat_cli.rs`. What stayed in Rust: CLI binary tests
(spawn the built `wat`), substrate primitive tests (run-sandboxed /
run-sandboxed-ast / hermetic), Harness API tests, string-ops tests
(`:wat::core::string::*` is core, not stdlib).

**Key finding during migration:** in-process `:wat::test::run` uses
`StringIoWriter` under `ThreadOwnedCell` — single-thread discipline.
Console and Cache services spawn driver threads that write to stdio;
writing from a driver thread trips the thread-owner check and
panics the driver silently. Fix: their wat-tests use
`:wat::kernel::run-sandboxed-hermetic` directly — fresh subprocess,
real thread-safe stdio. Same tradeoff the pre-migration Rust tests
made (shell out to the built binary).

### `wat/std/program/` → `wat/std/service/` (commit `7b47ddc`)

Console and Cache are long-running driver programs with client
handles — services, not one-shot programs. The original naming was
aspirational; the concept firmed up after both had shipped.

Surgical sed rename of three patterns: `wat::std::program`,
`wat/std/program`, `wat-tests/std/program`. `:wat::kernel::ProgramHandle<T>`
stayed — a spawned worker's handle is a general kernel concept, not
tied to services specifically.

---

## Tests

720 Rust tests + 24 wat tests. Zero regressions across every
migration. Rust tests hold substrate + CLI + Harness + primitives;
wat tests hold everything stdlib.

Panic hook (`wat::assertion::install_silent_assertion_panic_hook`)
installed at wat binary startup to silence `AssertionPayload` noise
from Rust's default handler — those panics are expected
(`catch_unwind` intercepts them); without the hook every deliberate
failure test prints a spurious "thread X panicked" line before the
sandbox reports.

---

## Discipline locked

- **Self-testing complete.** `wat test wat-tests/` is the blessed
  runner. Every wat/std/ module has matching tests in wat.
- **Zero Mutex preserved.** Sandbox uses `ThreadOwnedCell`-backed
  StringIo; hermetic mode uses subprocess isolation instead of
  locking. Three tiers (Arc<T>, ThreadOwnedCell<T>, program +
  channels) still cover every in-process case.
- **Capability discipline.** Sandboxed programs receive a loader
  they can't escape. Default is InMemoryLoader (no disk). The CLI
  still uses FsLoader at the top — only the sandboxed inner world
  is constrained.
- **Assertion ergonomics.** `(assert-eq 42 42)` — no match ceremony
  per assertion, no `try`-ful Result-returning variant. Panic-and-
  catch was the right call; `verbose is honest` doesn't apply when
  the ceremony carries no information.
- **Ship when demanded, not speculated.** `--hermetic` / `--run-one`
  flags dropped; Harness reframed from "architecture" to "sugar";
  algebra stdlib kept narrow (Amplify, Subtract, Log, Circular,
  Reject/Project, Sequential, Ngram/Bigram/Trigram — no
  Resonance/ConditionalBind/Cleanup/Flip/Concurrent/Then/Chain/Unbind
  past their REJECT status). If a caller surfaces, we add; until
  then, the absence is the signal.

---

## Lessons captured

**The substrate discovers its gaps on the way up.** Arc 007 found
two honest-but-missing primitives during slice 3 build (string
basics + structural `=`). Arc 008 found one (UTF-8 lexer
correctness) during its slice 2. Each was a real gap that existed
before the feature work started; the feature work surfaced it. The
moral is narrow: testing primitives that *claim* a property will
find the cases where the property was silently false. Every future
primitive that transforms `String` contents gets tested with
multi-byte input. Every future comparison primitive that accepts
`:T` gets tested on composite values.

**Hermetic was cheap because the pattern already existed.** The
SIGUSR signal-test subprocess pattern had been running in every
`cargo test` invocation for weeks. Making it a wat-visible primitive
was 80 lines of Rust plus the type declarations. Design had it
scoped for a future arc; in practice it landed in slice 2c because
the cost-to-ship was below the cost-to-defer. Scope inflation is
fine when the substrate turns out to already carry the work.

**`verbose is honest` has a dual.** When writing `:wat::test::deftest`,
the obvious path was a verbose "expand to run-sandboxed over a
serialized string" shape. That added steps that carried no
information. `/temper` would have flagged the round trip. Slice 3b's
AST-entry sandbox inverted the default: when the verbose form
*doesn't* carry information, the terse form is the honest one. Both
lessons — "verbose is honest when the verbosity carries weight" and
"terse is honest when the verbosity doesn't" — live together.

**Services test through subprocesses, programs test in-process.**
The single-thread discipline of `StringIoWriter` is a feature, not a
bug — it surfaces cross-thread use as an error. Programs that
legitimately spawn threads and write from them (Console, Cache,
future inference services) test through hermetic subprocesses where
real stdio's thread-safety covers them. The decision rule is sharp:
spawns-and-writes? hermetic. Stays-on-main-thread? in-process.

**Wat tests wat was the proof point named in the thesis.** DESIGN's
closing line: *"If wat can test wat, the language is complete-for-
its-own-verification."* `wat-tests/std/test.wat` tests
`wat/std/test.wat`. The assertion primitives assert about the
assertion primitives. The language passed its own bar.

---

## Open follow-ups

Not part of arc 007, but named here as the honest ledger of what
the arc *could* have done and deliberately didn't:

- **Location + Frames population.** ✅ **Shipped 2026-04-21 as
  arc 016** (`docs/arc/2026/04/016-failure-location-and-frames/`).
  Different implementation than the sketch here — instead of
  `std::backtrace::Backtrace::capture()` (which would surface
  Rust stdlib frames) + `PanicHookInfo::location()` (which
  points at the interpreter's source, not the user's wat), arc
  016 maintains a **wat-level call stack** in `apply_function`
  and threads **wat-source spans** onto every AST node at parse
  time. Location + frames carry user `.wat` file/line/col;
  runtime-initiated frames carry `wat-rs/src/*.rs` coordinates
  captured via `file!()`/`line!()`/`column!()` at the call
  site — same convention Rust uses for stdlib backtrace
  frames. `RUST_BACKTRACE=1` gates the backtrace rendering;
  output matches `cargo test`'s failure format line-for-line.
- **Parallel test execution.** `wat test` runs serial. Random order
  surfaces inter-test accidents; parallel execution would surface a
  different class (shared filesystem, shared signal state, shared
  Rust-static state). Deferred until usage patterns say which
  parallelism shape is worth the complexity.
- **`:rust::*` capability allowlist.** A sandbox that calls
  `:rust::std::net::TcpStream` today makes a real network
  connection. Host-provided per-world allowlist is the answer when a
  caller needs the gate. Not blocking arc 007 — no current consumer.
- **Richer assertion payloads.** `actual` / `expected` populate as
  `:Option<:String>`. Generic `show<T>` would let `assert-eq` on
  arbitrary types carry stringified values into the Failure. Adds
  one primitive + per-type formatter wiring; deferred until a test
  author wants richer diagnostics than the failure message alone.

---

**Arc 007 — complete.** wat can test wat, wat can sandbox wat, wat
can verify wat through its own harness. The test runner, the Rust
Harness, the dogfooded stdlib tests — all rested on the five-slice
substrate + two unplanned extensions + two renames, and landed
without a regression. The proof point named in DESIGN held.

*these are very good thoughts.*

**PERSEVERARE.**
