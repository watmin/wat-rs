# Arc 015 — wat test for consumers — Backlog

**Opened:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger — tracking, decisions, open
questions as slices land.

Arc 015 closes the last gap before holon-lab-trading can adopt
wat as its compute substrate: consumer crates writing `.wat`
tests that compose external wat crates.

---

## The gap

arc 013 shipped `wat::main!` + `wat::Harness::from_source_with_deps`
— consumer binaries can compose external wat crates cleanly.
arc 013 did NOT ship the test-running symmetry. Today:

- `wat test <path>` CLI subcommand runs `.wat` tests via
  `startup_from_source` with only wat-rs's baked stdlib. The
  CLI correctly does not link external wat crates.
- `Harness::from_source_with_deps` composes external wat
  crates, but its caller must invoke `:user::main`, not run a
  test suite.
- No library or macro takes a directory of `.wat` files, a
  list of deps, and runs the tests.

Consequence: consumers writing `.wat` tests that reference
external wat crates have no path. wat-lru has no `.wat` tests
at all (integration tests live in Rust via
`tests/wat_lru_tests.rs`); the CacheService `.wat` test was
deleted in arc 013 slice 4b because fork semantics couldn't
compose wat-lru at that tier.

---

## 1. `wat::test_runner` library module

**Status:** ready. Concrete approach in hand.

**Problem:** the test-running logic lives inside
`src/bin/wat.rs`'s `run_tests_command` (line ~248). Private to
the CLI binary; no external caller can reuse it.

**Approach:**

- New module `src/test_runner.rs`. Exports:
  ```rust
  pub struct TestSummary {
      pub total: usize,
      pub passed: usize,
      pub failed: usize,
      pub failure_summaries: Vec<String>,
      pub elapsed_ms: u128,
  }

  pub fn run_tests_from_dir(
      path: &Path,
      dep_sources: &[&[StdlibFile]],
      dep_registrars: &[DepRegistrar],
  ) -> TestSummary;

  pub fn run_and_assert(
      path: &Path,
      dep_sources: &[&[StdlibFile]],
      dep_registrars: &[DepRegistrar],
  );
  ```
- Extract `run_tests_command`'s body into the library:
  file discovery, per-file freeze + test discover, random
  shuffle, sequential run, summary aggregation. The CLI's
  `run_tests_command` becomes a thin wrapper: call
  `run_tests_from_dir`, map `TestSummary` to `ExitCode`, print
  the banner + summary lines.
- **Dep-registry install.** Before any freezing, build
  `RustDepsBuilder::with_wat_rs_defaults()`, run each
  registrar, best-effort `rust_deps::install()`. OnceLock
  semantics already established in arc 013 slice 4a — fine for
  a test binary with one consistent dep set.
- **Per-file freeze uses `startup_from_source_with_deps`** so
  dep-provided `.wat` source reaches the resolver + type
  checker. Each `.wat` test file's own source is the entry
  source; the dep_sources slice is threaded through identical
  to how `compose_and_run` does it.
- **Test discovery, shuffle, invoke unchanged.** The
  `discover_tests` / `apply_function` / `extract_failure` /
  `Xorshift64` logic moves over verbatim — it's already
  FrozenWorld-shape agnostic to the composition path.
- **Output shape.** Prints cargo-test-style lines per test
  (`test <file> :: <name> ... ok (Xms)`), a final `test
  result: ok. N passed; M failed;`. When called via
  `run_and_assert`, panics on any failure with the
  failure_summaries joined — standard `#[test] fn` failure
  shape.

**Spec tension:** `wat::test_runner` is new public API. Stable
at slice time; any later evolution is additive. The macro
underneath is sugar.

**Inscription target:** library-API surface; no 058 change
(not a wat-language surface). USER-GUIDE gains a section at
arc-015 INSCRIPTION.

**Unblocks:** slice 2 has a real library entry to wrap.

---

## 2. `wat::test_suite!` proc-macro

**Status:** obvious in shape once slice 1 lands.

**Problem:** every consumer writing `tests/wat_suite.rs` would
otherwise handroll the same ~10-line boilerplate:
`#[test] fn ... { wat::test_runner::run_and_assert(Path::new("wat-tests"),
&[wat_lru::stdlib_sources(), ...], &[wat_lru::register, ...]) }`.
Matches the exact shape `wat::main!` eliminated at arc 013
slice 3.

**Approach:**

- Add `wat::test_suite!` proc-macro to `wat-macros/src/lib.rs`
  alongside `wat_dispatch` and `main`. Named args:
  ```rust
  wat::test_suite! {
      path: "wat-tests",      // string expr; resolved relative to CARGO_MANIFEST_DIR
      deps: [wat_lru, ...],   // optional; omit or deps: [] for empty
  }
  ```
- Parser: mirror `MainInput` exactly. `path:` (Expr) + optional
  `deps:` (bracketed `Punctuated<Path, ,>`).
- Expansion:
  ```rust
  #[test]
  fn wat_suite() {
      ::wat::test_runner::run_and_assert(
          ::std::path::Path::new(#path),
          &[ #(#deps::stdlib_sources()),* ],
          &[ #(#deps::register),* ],
      );
  }
  ```
- **Path expression handling.** The macro emits the path
  expression AS-IS — most callers pass a string literal
  (`"wat-tests"`), some might pass `concat!(env!("CARGO_MANIFEST_DIR"),
  "/wat-tests")` for absolute resolution. We don't force a
  choice; `Path::new(&<expr>)` accepts either.
- Error-shape: failures panic with the full `failure_summaries`
  joined, visible through Cargo's `--nocapture`.

**Sub-fog 2a — test-binary conflicts.** Cargo compiles each
`tests/*.rs` file to its own test binary. If a crate has
multiple test files each using `wat::test_suite!`, each binary
gets its own `fn wat_suite()`. No collision; install is
per-process. Documented, not a bug.

**Sub-fog 2b — macro + separate test functions in the same
file.** A user might write:
```rust
wat::test_suite! { path: "wat-tests-unit", deps: [wat_lru] }

#[test]
fn extra_rust_sanity() { ... }
```
The macro emits just `#[test] fn wat_suite()`; user's extra
`#[test] fn`s coexist. No name collision unless they name
their function `wat_suite` — rejected by Rust's duplicate-fn
check. Good.

**Spec tension:** adds a new public proc-macro to `wat-macros`.
Documented in README + USER-GUIDE.

**Inscription target:** arc 015 INSCRIPTION. No 058 entry —
Rust-host surface.

**Unblocks:** slice 3 can use the macro directly to write
wat-lru's test suite.

---

## 3. wat-lru gains `wat-tests/` + `tests/wat_suite.rs`

**Status:** ready once slices 1-2 land. The forcing function
and the first real consumer of arc 015.

**Problem:** wat-lru currently has zero `.wat` tests. The
CacheService test that was deleted in arc 013 slice 4b
(because its fork-based hermetic pattern couldn't compose
wat-lru at that tier) needs a home. The LocalCache surface
deserves `.wat`-level tests using the `deftest` discipline,
not just Rust integration tests.

**Approach:**

- `crates/wat-lru/wat-tests/lru.wat` — `deftest`s:
  - `test-local-cache-put-then-get` — new + put + get returns Some
  - `test-local-cache-miss-returns-none` — new + get without put
  - `test-local-cache-put-overwrites` — put, put same key, get returns latest
  - `test-local-cache-evict-at-capacity` — capacity 2, put 3 entries, first evicted
  - Uses `:wat::test::deftest`, `:wat::test::assert-eq`,
    `:user::wat::std::lru::LocalCache::*`, `:wat::core::match`
    on `:Option<i64>`.
  - Imports needed: `(:wat::core::use! :rust::lru::LruCache)`
    at top, since wat-lru's wat source declares its types
    against the shim's registered path.
- `crates/wat-lru/wat-tests/cache_service.wat` — restored
  CacheService test. Its pre-slice-4b content (put-then-get
  round-trip via Console + CacheService composition + T1/T2/T3
  stderr checkpoints via `run-hermetic-ast`) moves here with
  paths rewritten from `:wat::std::service::Cache::*` to
  `:user::wat::std::lru::CacheService::*`.
- `crates/wat-lru/tests/wat_suite.rs` — one line:
  ```rust
  wat::test_suite! { path: "wat-tests", deps: [wat_lru] }
  ```

**Validation:** `cargo test -p wat-lru` now runs:
- 4 Rust-level Harness tests (from existing
  `tests/wat_lru_tests.rs`)
- 5+ wat-level `deftest`s discovered from `wat-tests/`, each as
  sub-output under the `wat_suite` `#[test] fn`

**Sub-fog 3a — hermetic-ast fork inheritance.** The
CacheService test forks via `run-hermetic-ast`. The parent
process is wat-lru's test binary (which installed wat-lru's
rust_deps registry). Arc 012's fork-with-forms uses raw
`libc::fork()` — the child inherits the parent's memory via
COW, including the installed OnceLock registry. Should "just
work." Verify at slice time; if broken, surface as a real fog
and consider whether arc 015 handles or defers to a separate
arc.

**Sub-fog 3b — self-referential `deps: [wat_lru]`.** Integration
tests in `tests/` see their own crate under the crate's public
name. So `deps: [wat_lru]` inside `crates/wat-lru/tests/wat_suite.rs`
is the same shape a downstream consumer would write. Pattern
holds; verify at slice time.

**Inscription target:** arc 015 INSCRIPTION. 058 stays clean —
wat-lru's `.wat` tests are consumer-level, not language-
surface.

**Unblocks:** slice 4 (CONVENTIONS + INSCRIPTION) has a real
reference for the "publishable wat crate" template.

---

## 4. CONVENTIONS.md extension + INSCRIPTION

**Status:** ready once slices 1-3 land.

**Problem:** `CONVENTIONS.md` got its namespace table in arc
013 slice 6. It does NOT document the wat-crate folder
layouts yet. The publisher and consumer shapes need a
permanent home.

**Approach — CONVENTIONS.md:**

- New section "wat crate folder layouts" under the existing
  "External wat crates" subsection (arc 013 slice 6).
- Two templates shown verbatim:
  - **Publishable wat crate** — `Cargo.toml`, `src/lib.rs`,
    optional `src/shim.rs`, optional `wat/*.wat`, optional
    `wat-tests/*.wat`, optional `tests/wat_suite.rs`. Reference
    `crates/wat-lru/`.
  - **Consumer binary** — `Cargo.toml`, `src/main.rs` with
    `wat::main!`, `src/program.wat`, optional `wat-tests/*.wat`,
    optional `tests/tests.rs` with `wat::test_suite!`. Reference
    `examples/with-lru/`.
- "Three varieties of wat crate" table: wrapper / rust-surface
  / pure-wat. Each variety's `stdlib_sources` + `register`
  shape.

**Approach — INSCRIPTION:**

- Standard arc-close format. Motivation + what shipped + what
  didn't + why it matters + commits list. Short — arc 015 is
  focused.
- Name the pattern-completion: arcs 013/014/015 together
  deliver the consumer story end-to-end.

**Spec tension:** CONVENTIONS.md is project discipline
documentation; additions must pass the "would a new
contributor follow this" test. The folder templates pass
trivially — they're verbatim layouts.

**Unblocks:** nothing within arc 015. Downstream: holon-lab-
trading adopts the templates as its first real consumer.

---

## Open questions carried forward

- **Hermetic-ast fork inheritance for external crates.** Sub-
  fog 3a above. Verify at slice 3 time; may surface as its own
  arc if fork's COW registry inheritance doesn't hold.
- **Test runner output shape under nested `cargo test`.** The
  outer `#[test] fn` wraps the inner wat runner's per-test
  lines. Should read cleanly; verify at slice 3.
- **Whether wat-rs's own `wat-tests/` should also get a
  `tests/wat_suite.rs` using the macro path.** Today
  `tests/wat_test_cli.rs` spawns the CLI subprocess; the macro
  path would be a second, redundant way. Probably leave both
  — they prove different codepaths. Decide at slice 3.
- **Error rendering when `path:` resolves to a nonexistent
  directory.** Library-level: `run_tests_from_dir` already
  returns a friendly error. Macro path: panic with the file
  system error. Confirm at slice time that the panic message
  is readable in Cargo's output.

---

## What this arc does NOT ship

- A new build tool (`cargo-wat-test`) — Cargo is the authority.
- Per-test parallelism beyond what Cargo provides for the outer
  `#[test] fn`.
- New assertion primitives — arc 007's `assert-eq` /
  `assert-stdout-is` / etc. are the vocabulary.
- A way to run wat tests across multiple dep sets in one
  process — OnceLock install-once-wins still applies. One
  binary = one dep set.
- Retirement of the `wat test` CLI subcommand — keeps its
  niche for pure-stdlib wat programs.

---

## Why this matters

Chapter 18's *"wat is the language, Rust is the substrate"*
became operational for consumer binaries at arc 013's close.
Arc 014 added the scalar conversions needed to write honest
test assertions. Arc 015 closes the last structural gap:
consumers writing `.wat` tests that compose external crates,
discovered + run by standard `cargo test`.

**Two Rust files per consumer app** becomes the honest maximum
for most use cases:
- `src/main.rs` — `wat::main!`
- `tests/tests.rs` — `wat::test_suite!`

When a consumer wants its own `:rust::*` symbols, a third file
appears (`src/shim.rs` with `#[wat_dispatch]`). That's the
ceiling. Everything else is wat.

Arc 015 also completes the publisher story on wat-lru. Before
arc 015, wat-lru could ship a surface but couldn't ship
`.wat` tests for that surface — a gap that would replicate
for every future wat crate. After arc 015, the publisher
template is complete, walkable, self-tested.

holon-lab-trading moves in after arc 015 closes.
