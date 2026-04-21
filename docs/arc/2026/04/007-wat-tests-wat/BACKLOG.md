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
| 2 | `:wat::kernel::run-sandboxed` primitive | pending | — |
| 2 | `RunResult` struct registration | pending | — |
| 2 | panic-catch isolation | pending | — |
| 2 | spawned-threads drain-and-join before return | pending | — |
| 3 | `RuntimeError::AssertionFailed` variant + `catch_unwind` surfacing | pending | — |
| 3 | `:wat::test::assert-eq` | pending | — |
| 3 | `:wat::test::assert-contains` | pending | — |
| 3 | `:wat::test::assert-stdout-is` | pending | — |
| 3 | `:wat::test::assert-stderr-matches` | pending | — |
| 3 | `:wat::test::run` / `run-in-scope` wrappers | pending | — |
| 4 | `wat-vm test <path>` subcommand (discovery + runner) | pending | — |
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
  `wat-vm test` runs serial. Parallelism is a follow-up once
  usage patterns expose which tests can safely run
  concurrently.

---

## Relationship to arc 006

Arc 006 paused at `with-state` (slice 4 of arc 006) pending
this detour. Once arc 007 ships slice 3, `with-state` can be
implemented AND TESTED in wat — the first substantive use of
the self-testing harness. That test file is the proof point
for this arc.
