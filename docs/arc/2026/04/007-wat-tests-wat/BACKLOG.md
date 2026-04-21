# Arc 007 — wat tests wat — Backlog

**Opened:** 2026-04-20 (detour from arc 006).
**Scoped:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md).

---

## Tracking

| Slice | Item | Status | Commit |
|---|---|---|---|
| 1 | file-I/O audit — confirm all reads go through Loader | **done** | audit pass |
| 1 | SourceLoader trait + source_loader field on SymbolTable (slice 1a) | **done** | `1a17e2c` |
| 1 | `:wat::eval::file-path` routes through loader (slice 1b) | **done** | this slice |
| 1 | `:wat::verify::file-path` routes through loader (slice 1c) | **done** | this slice |
| 1 | `RuntimeError::NoSourceLoader` variant + `eval_expr_with_fs` test helper | **done** | this slice |
| 1 | `ScopedLoader` impl + `LoadFetchError::OutOfScope` + 7 tests | **done** | this slice |
| 1 | USER-GUIDE capability-boundary section | **done** | this slice |
| 2a | `:wat::kernel::run-sandboxed` primitive (happy path) | **done** | `647e2b7` |
| 2a | `RunResult` struct registration | **done** | `647e2b7` |
| 2b | panic-catch isolation + `Failure` struct | **done** | `a6302c0` |
| 2b | spawned-threads drain-and-join before return | **done** | `a6302c0` |
| 2c | `:wat::kernel::run-sandboxed-hermetic` (subprocess isolation) | **done** | `5bd73c8` |
| 2c | Double-colon builtin registration bug (`:Vec` / `:Option` heads) | **done** | `5bd73c8` |
| 3 | `RuntimeError::AssertionFailed` variant + `catch_unwind` surfacing | **done** | `f03d821` |
| 3 | `:wat::kernel::assertion-failed!` primitive + `AssertionPayload` | **done** | `f03d821` |
| 3 | `:wat::test::assert-eq` (pure-wat over structural `:wat::core::=`) | **done** | `f03d821` |
| 3 | `:wat::test::assert-contains` | **done** | `f03d821` |
| 3 | `:wat::test::assert-stdout-is` | **done** | `f03d821` |
| 3 | `:wat::test::assert-stderr-matches` | **done** | `f03d821` |
| 3 | `:wat::test::run` / `run-in-scope` wrappers | **done** | `f03d821` |
| 3 | Substrate precursor: `:wat::core::string::*` + `:wat::core::regex::*` (8 primitives) | **done** | `97a1ec5` |
| 3 | `:wat::core::=` extended to structural equality (Vec/Tuple/Option/Result/Struct) | **done** | `f03d821` |
| 3b | `startup_from_forms` — split startup pipeline at parse boundary | pending | — |
| 3b | `:wat::kernel::run-sandboxed-ast` primitive (AST-entry sandbox) | pending | — |
| 3b | `:wat::test::deftest` defmacro — Clojure-style ergonomic shell | pending | — |
| 3b | Integration test — deftest expands to a working named test fn | pending | — |
| 4 | `wat test <path>` subcommand (discovery + runner) | pending | — |
| 4 | cargo-test-style report formatting | pending | — |
| 5 | `wat::Harness` Rust API | pending | — |
| 5 | crate-root re-export from `lib.rs` | pending | — |

---

## Decision log

- **2026-04-21** — Arc scoped. Five slices. Filesystem sandbox
  (slice 1) is the first move because Chapter 16 of BOOK.md
  flagged the capability bypass as future work; this arc closes
  that promise before sandboxed execution ships.
- **2026-04-21** — Main-signature for `run-sandboxed`: STRICT
  three-channel. Rejected "loose / accept any main signature"
  alternative because users who want no-channel main have
  `eval-edn!` already. Two paths, two concerns.
- **2026-04-21** — Assertion mechanism: PANIC-AND-CATCH.
  Rejected Result-return alternative because it taxes every
  assertion with match ceremony. Panic-and-catch requires one
  new `RuntimeError` variant + `catch_unwind` in run-sandboxed.
- **2026-04-21** — `:rust::*` capability allowlist: DEFERRED
  to its own arc. Slice 1 closes filesystem; network + process
  isolation is a bigger design surface. Documented in
  DESIGN.md's out-of-scope section.
- **2026-04-21** — Loader-attachment shape: **SymbolTable**
  (alongside `encoding_ctx`). Rejected new RuntimeContext struct
  (would invent a second capability-carrier abstraction when one
  already exists). Rejected removing `:wat::eval::file-path`
  entirely (pushes the problem to a new primitive). Verified
  against prior art: Common Lisp, Scheme, Clojure, Rust compiler's
  Session, Ruby globals, Haskell ReaderT, Agda backend-table — all
  carry startup-bound runtime capabilities via some structure
  accessible to primitives at dispatch. Second convergence this
  session (first was `with-state` matching Mealy 1955 / Elixir /
  Rust / Haskell). See DESIGN.md's "Why loader-on-SymbolTable"
  section.
- **2026-04-21** — Rust-runtime state isolation: OUT OF SCOPE for
  this arc, **scaffolded for future**. In-process sandboxes share
  `static` / `lazy_static` / `OnceLock` state across sandboxes and
  the outer process (same model as `cargo test`). True process-level
  isolation requires subprocess-per-test — named as a future
  "hermetic-mode" arc. Arc 007 bakes four scaffolding decisions
  (serializable TestResult, single-test addressability, CLI contract
  room for `--hermetic` + `--run-one`, parallel exit-code semantics)
  so hermetic lands as a clean extension, not a breaking change.
  See DESIGN.md "Scaffolding for hermetic-mode (future arc)".
- **2026-04-21** — `Failure` shape pinned. `Option<String>` was a
  lazy shorthand; the real `Failure` value has `message`,
  `location: Option<Location>`, `backtrace: Option<String>`,
  `actual: Option<String>`, `expected: Option<String>`. Flat struct
  with optional fields — slice 2b ships the first three; slice 3
  populates actual/expected from assertion payloads. Every field is
  a primitive, JSON-serializable for hermetic-mode. See DESIGN.md's
  "Structured failure" section.
- **2026-04-21** — RunResult.returned field DROPPED. Strict three-
  channel `:user::main` always returns `:()`; the field would be
  dead weight today. Slice 2a ships RunResult with just
  `{ stdout, stderr }`. Slice 2b extends to add `failure`. Re-add
  `returned` when a real caller needs a non-Unit return shape.
- **2026-04-21** — Parallel test execution: DEFERRED. V1 of
  `wat test` runs serial. Parallelism is a follow-up once
  usage patterns expose which tests can safely run
  concurrently.
- **2026-04-21** — Hermetic mode BROUGHT FORWARD. The "future
  hermetic-mode arc" was going to be its own thing; the SIGUSR
  subprocess-isolation pattern already in the signal tests made
  the implementation cheap enough to land in slice 2c instead.
  `:wat::kernel::run-sandboxed-hermetic` ships alongside
  `run-sandboxed`: no new CLI flag, no wat mode switch — the
  primitive name picks the semantic. Round-trip test proves
  `(eval-edn! (RunResult/stdout (run-sandboxed-hermetic inner-src ...)))`
  works end-to-end: wat generates wat, wat runs wat, wat
  evaluates wat's output.
- **2026-04-21** — Assertion raise mechanism: **wat stdlib over
  ONE kernel primitive** (Option B), not per-assertion Rust
  primitives (Option A). Single new kernel primitive
  `:wat::kernel::assertion-failed!` does the panic_any with
  `AssertionPayload`; six `:wat::test::assert-*` forms in
  `wat/std/test.wat` build on it. Adding `assert-ne`, `assert-
  not`, `assert-between`, etc. later is pure wat, no Rust delta.
- **2026-04-21** — Assertion actual/expected populated via panic
  payload downcast (design lines 322-339), NOT via
  `RuntimeError::AssertionFailed`. The Err variant still exists
  for symmetry (a future Rust-side harness that catch_unwinds
  itself can prefer structured returns) but in-process
  assertion-failed! always panics. Sandbox's
  `failure_from_panic_payload` downcasts `AssertionPayload`
  before the string fallbacks; actual/expected land in the
  Failure struct directly.
- **2026-04-21** — Substrate precursor: `:wat::core::string::*`
  (7 forms) + `:wat::core::regex::matches?` (1 form). Driven by
  slice 3's `assert-contains` + `assert-stderr-matches` needs
  but useful everywhere. Per-type namespace mirrors
  `:wat::core::i64::*` precedent. Regex lives at its own
  `:wat::core::regex::*` path because the `regex` crate is a
  distinct concern (future feature-gate target). Char-oriented
  `length` (not bytes) matches user mental model. `split` refuses
  empty separator as MalformedForm — always a bug.
- **2026-04-21** — `:wat::core::=` extended to structural
  equality. The old primitive-only comparison couldn't compare
  Vec<String> at all — a gap `assert-stdout-is` surfaced. New
  `values_equal` returns `Option<bool>`: `Some(_)` when types
  are comparable (composites included), `None` when they can't
  be meaningfully compared (e.g., Function vs. anything). `<`,
  `>`, `<=`, `>=` stay primitive-only; a Vec of structs has no
  canonical ordering worth inventing. A substrate discovery
  worth recording separately from the slice 3 test work.
- **2026-04-21** — Slice 3b opened. The `:wat::test::run` helper
  is ergonomic but still requires callers to hand-build wat
  source strings with full `:user::main` scaffolding every time.
  A Clojure-style `deftest` macro would collapse that, but the
  sandbox's current "enter from source text" API forces the
  macro to serialize its quoted body back to source — the
  honest-but-wasteful round-trip `/temper` would flag. Decision:
  split `startup_from_source` at the parse boundary, expose
  `startup_from_forms(Vec<WatAST>, ...)`, add
  `:wat::kernel::run-sandboxed-ast` that accepts `Vec<AST<()>>`
  directly. Then `deftest` is pure wat stdlib. Chose the
  AST-entry path over a simpler "expose the serializer"
  primitive because the former opens doors (dynamically
  generated tests, fuzzers, compiler passes with AST in hand)
  that the latter doesn't.
- **2026-04-21** — `deftest` signature (slice 3b, tentative):
  `(:wat::test::deftest name dims mode body)` expands to a
  `:wat::core::define` of a named function returning
  `:wat::kernel::RunResult`. `dims`/`mode` are AST<i64>/AST<keyword>
  at macro boundary so the user can pass literals or constants.
  Body is `AST<()>` — the test expression. Expansion quasi-
  quotes the config setters and a `:user::main` wrapper into a
  `Vec<AST<()>>` and hands it to `run-sandboxed-ast`. When
  slice 4 lands, the test discoverer iterates all registered
  functions under a `:*::test::*` pattern and calls each.

---

## Relationship to arc 006

Arc 006 paused at `with-state` (slice 4 of arc 006) pending
this detour. Once arc 007 ships slice 3, `with-state` can be
implemented AND TESTED in wat — the first substantive use of
the self-testing harness. That test file is the proof point
for this arc.

## Relationship to arc 008

Arc 007 slice 2a PAUSED 2026-04-21 when implementation
discovered that `:user::main` takes concrete
`:rust::std::io::Stdin/Stdout/Stderr` — can't be substituted for
in-memory buffers at run-sandboxed. Arc 008 opened as a
prerequisite to introduce `:wat::io::IOReader` + `:wat::io::IOWriter`
abstractions (Ruby StringIO model). After arc 008 ships, arc 007
slice 2a resumes on the new IO substrate. See
[`../008-wat-io-substrate/DESIGN.md`](../008-wat-io-substrate/DESIGN.md).
