# Arc 015 — wat test for consumers

**Status:** opened 2026-04-21.
**Motivation:** arc 013 proved external wat crates compose
cleanly into consumer binaries via `wat::main!`. Arc 014 shipped
scalar conversions during a cave-quest pause of 013. Both closed
same day. Before holon-lab-trading can adopt the substrate as a
real consumer, one gap remains: **consumers cannot write wat
tests that use external crates.**

Today's `wat test <path>` subcommand runs `.wat` tests via
`startup_from_source` with only wat-rs's baked stdlib. An
external crate's surface (`:user::wat::std::lru::*`) is
unreachable — the CLI binary correctly does not link any
external wat crate. The symmetric gap with arc 013's
`Harness::from_source_with_deps` is obvious in retrospect:
the consumer-composition path exists for `:user::main` but not
for the test runner.

Arc 015 closes the symmetry.

---

## The shape — Cargo's authority, one more time

Rust users write `#[test] fn foo() { ... }` and `cargo test`
handles discovery, binary compilation, invocation, and result
aggregation. Users don't invoke `rustc` or configure a custom
test orchestrator.

wat inherits the same authority. The user writes `(:wat::test::deftest
:foo::test-bar ...)` inside `.wat` files. To make `cargo test`
pick them up — with external wat deps composed in — they add
one Rust file:

```rust
// tests/tests.rs
wat::test_suite! {
    path: "wat-tests",
    deps: [wat_lru],
}
```

That expands to a `#[test] fn` Cargo already knows how to find
and run. The library underneath (`wat::test_runner`) runs each
discovered `.wat` file through the same freeze + test-discover
+ random-shuffle logic the CLI already has — but with
`dep_sources` + `dep_registrars` threaded through.

---

## Non-goals (named explicitly)

- **A new build tool.** No `cargo-wat-test` binary, no parallel
  test orchestrator. Cargo is the authority. The macro emits
  `#[test] fn` and Cargo does everything else.
- **Per-test parallelism.** The CLI's test runner is
  sequential-per-file, randomized-within-file. Arc 015
  preserves that shape. Cargo's `--test-threads` affects only
  the outer `#[test] fn`, not the wat tests inside.
- **A different test-discovery convention.** `test-` prefix +
  `:wat::kernel::RunResult` return + zero args stays as the
  discovery rule. Arc 007's `deftest` macro + arc 012's fork-
  based hermetic already produce the right shapes.
- **Hermetic-ast fork semantics across external crates.** The
  fork child inherits the parent's rust_deps registry via COW —
  should "just work" post-arc-012 because the parent test
  binary was the one that installed the registry. Verify at
  slice 3 when wat-lru's CacheService test lands; flag if the
  fork-inheritance story breaks. Not in arc 015's explicit
  scope to fix.
- **Sandboxed test-binary rebuilds.** `cargo test` already
  rebuilds test binaries when sources change. No wat-level
  caching story needed.
- **Running wat tests from a wat program** (meta-testing). The
  test runner is Rust-callable via the macro or its library
  entry point. A wat-level `(run-wat-tests! ...)` primitive is
  out of scope; callers that need in-wat test orchestration
  already have `run-hermetic-ast`.

---

## What this arc ships

Four slices. Ordered so each lands on a live substrate.

### Slice 1 — `wat::test_runner` library module

Port the logic from `src/bin/wat.rs`'s `run_tests_command`
(line ~248) into a library function in `src/test_runner.rs`:

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

/// Wraps `run_tests_from_dir`, panics with all failure_summaries
/// joined if any test failed. Standard `#[test] fn` failure shape.
pub fn run_and_assert(
    path: &Path,
    dep_sources: &[&[StdlibFile]],
    dep_registrars: &[DepRegistrar],
);
```

The CLI's `run_tests_command` refactors to a thin wrapper:
parse argv, call `run_tests_from_dir`, map `TestSummary` to
`ExitCode`. Single source of truth.

**Dep-registry install.** `run_tests_from_dir` builds a
`RustDepsBuilder` from `with_wat_rs_defaults()`, calls each
registrar, best-effort installs via `rust_deps::install()`.
Same OnceLock discipline as `compose_and_run` — first-call-wins
is fine for a test suite with one consistent dep set, and the
suite runs in its own test-binary process so install doesn't
collide with other Cargo tests.

### Slice 2 — `wat::test_suite!` proc-macro

In `wat-macros`, sibling of `wat::main!`. Named args:

```rust
wat::test_suite! {
    path: "wat-tests",      // or path: some::expr
    deps: [wat_lru, ...],   // optional; omit or deps: [] for no deps
}
```

Expands to:

```rust
#[test]
fn wat_suite() {
    ::wat::test_runner::run_and_assert(
        ::std::path::Path::new("wat-tests"),
        &[wat_lru::stdlib_sources(), ...],
        &[wat_lru::register, ...],
    );
}
```

Error shape on failure: the `run_and_assert` panic carries all
`failure_summaries` joined, so the `#[test] fn`'s output prints
every failing wat test. Cargo's `--nocapture` / `--test-threads`
flags pass through naturally.

**Path resolution.** `path:` is a string expression resolved
relative to `CARGO_MANIFEST_DIR` — same rule `include_str!`
uses. That keeps the macro user's path literal stable whether
they run `cargo test` from the workspace root or the crate's
directory.

### Slice 3 — wat-lru gains its own wat tests

The forcing function and the proof target. Three files added
to `crates/wat-lru/`:

- `wat-tests/lru.wat` — `deftest`s for the LocalCache surface:
  `new` + `put` + `get` round-trip, overwrite-same-key,
  evict-at-capacity, miss-returns-None, zero-capacity
  rejection.
- `wat-tests/cache_service.wat` — restores the previously-
  deleted CacheService test. Uses `run-hermetic-ast` to fork a
  child that spins up Console + CacheService, round-trips a
  put + get, asserts on stdout / stderr checkpoints. The child
  inherits the parent test-binary's registered rust_deps via
  COW, so wat-lru's shim reaches the child process.
- `tests/wat_suite.rs` — one line: `wat::test_suite! { path:
  "wat-tests", deps: [wat_lru] }`.

`cargo test -p wat-lru` now runs:
- 4 existing Rust-level Harness integration tests
  (`tests/wat_lru_tests.rs`)
- N wat-level tests discovered from `wat-tests/`, each as a
  sub-output under the `wat_suite` `#[test] fn`

**Self-referential `deps: [wat_lru]`.** wat-lru's own test
suite declares wat-lru as a dep. Syntactically identical to how
a downstream consumer declares it. Integration tests in
`tests/` see the crate under its own name (standard Cargo
integration-test shape), so the pattern holds.

### Slice 4 — CONVENTIONS.md + INSCRIPTION

- `wat-rs/docs/CONVENTIONS.md` gains a "wat crate folder
  layouts" section — two templates (publishable library,
  consumer binary) + the three varieties table (wrapper,
  rust-surface, pure-wat). References `crates/wat-lru/` and
  `examples/with-lru/` as walkable templates.
- Arc 015 INSCRIPTION closes the arc.

---

## Resolved design decisions

- **2026-04-21** — **No new build tool.** Cargo-visible `#[test]
  fn` is the authority. `wat::test_suite!` is the macro that
  emits it.
- **2026-04-21** — **Macro sugar + library substrate.**
  `wat::test_runner` lives as a library; the macro is sugar.
  Mirrors `wat::compose_and_run` + `wat::main!`.
- **2026-04-21** — **Three varieties of wat crate, one
  contract.** Wrapper (`stdlib_sources` + `register` both
  non-trivial), rust-surface (`stdlib_sources` empty,
  `register` populates), pure-wat (`stdlib_sources`
  non-trivial, `register` no-op). All three satisfy the same
  Rust-level signature.
- **2026-04-21** — **Prove on wat-lru, no new example crate.**
  wat-lru already has no `[[bin]]`, already has both sides of
  the contract, already has Rust integration tests. Adding its
  own `wat-tests/` + `tests/wat_suite.rs` is the strongest
  proof — syntactically identical to how a consumer writes
  `deps: [wat_lru]`, and restores the CacheService test that
  was deleted in arc 013 slice 4b because fork semantics
  couldn't compose external crates at that tier.
- **2026-04-21** — **Discovery convention unchanged.** `test-`
  prefix + zero-arg + `:wat::kernel::RunResult` return stays
  verbatim. The runner is the same; only its composition
  surface grows.

---

## Open questions to resolve as slices land

- **Hermetic-ast fork target for external wat crates.**
  `run-hermetic-ast` uses `fork-with-forms` which inherits the
  parent's rust_deps via COW. Should work for wat-lru's
  CacheService test (parent test binary installed the registry
  with wat-lru's shim; child inherits). Verify at slice 3. If
  broken, flag as sub-fog and consider whether arc 015 handles
  or defers.
- **Running wat-rs's own wat tests via the macro path.** wat-rs
  has no external deps, so `wat::test_suite! { path:
  "wat-tests" }` with no `deps:` would work — but it's
  redundant with the existing CLI-subprocess test in
  `tests/wat_test_cli.rs`. Probably leave both; they prove
  different things (CLI path vs macro path). Decide at slice 3.
- **Shared path-prefix behavior when tests imports each
  other.** Unlikely at arc 015's scope — each `.wat` file
  freezes independently, no cross-file symbol sharing. But if
  `wat-tests/common.wat` ever becomes a load-target, document
  the semantics. Out of scope today.
- **TestSummary printing shape.** Cargo's `#[test] fn` runner
  wraps everything in its own `test <name> ... ok` line. wat's
  inner runner also prints per-test `test <file> :: <name>
  ... ok (Xms)`. Both show up; the nested shape is natural but
  verify it reads cleanly at slice 3.

---

## What this arc does NOT ship

- **Publishing workflow for wat crates** — remains out of
  scope; arc 013's non-goal carries forward.
- **Per-test timeouts / memory limits** — `cargo test` has
  its own `--timeout` story; the wat test runner runs
  synchronously and inherits the outer `#[test] fn`'s
  behavior.
- **Golden-file / snapshot testing primitives** — arc 007's
  `deftest` + `assert-eq` + `assert-stdout-is` surface is
  the test-authoring vocabulary; arc 015 doesn't add new
  assertion primitives.
- **Running wat tests across multiple dep sets in one process**
  — OnceLock install-once-wins still applies. A test binary is
  one consistent dep set. Separate dep sets = separate binaries
  = separate `tests/*.rs` files.
- **Retire the `wat test` CLI subcommand.** Keeps its niche
  (wat programs that use only baked stdlib — no Rust wrapper
  crate needed). `wat-rs`'s own `tests/wat_test_cli.rs`
  continues using it.

---

## The thread this continues

Arc 013 proved consumers could compose external wat crates into
binaries. Arc 014 shipped the scalar-conversion primitives
needed to assert on test output. Arc 015 is the last piece
holon-lab-trading needs before adopting wat as its compute
substrate: **consumer crates can write `.wat` tests that
compose external crates, and `cargo test` picks them up
without ceremony.**

When arc 015 closes, the full consumer story is:

```
my-consumer-crate/
├── Cargo.toml           # [dependencies] wat + wat-lru
├── src/
│   ├── main.rs          # wat::main! { source: ..., deps: [wat_lru] }
│   └── program.wat
├── wat-tests/
│   └── *.wat            # deftests that use :user::wat::std::lru::*
└── tests/
    └── tests.rs         # wat::test_suite! { path: "wat-tests", deps: [wat_lru] }
```

Two Rust files. Everything else is wat. `cargo run` + `cargo
test` work out of the box. That's the shape.

After arc 015, the trading lab moves in.
