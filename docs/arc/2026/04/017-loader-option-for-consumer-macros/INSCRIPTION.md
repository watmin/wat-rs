# Arc 017 — Loader option for consumer macros — INSCRIPTION

**Status:** shipped 2026-04-22. Three slices (+ a drive-by clippy sweep).
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

`wat::main!` hard-wired `InMemoryLoader` via `compose_and_run`
(`compose.rs:118-122`, pre-017). Consumer binaries with multi-file
wat programs — the trading lab being the first real case — hit a
wall: `(:wat::core::load! :wat::load::file-path "path")` from
inside `program.wat` always returned `NotFound`.

The builder's framing:

> there's always an unexpected quest - the dungeon master provides
>
> we don't back down from a fight..
>
> we must be able to support loads being called recursively...
>
> and we only need the entry point to call the dims and capacity
> mode... — this is a binary vs lib distinction....

Arc 017 closes the gap and honors the binary-vs-library distinction
the builder named. Consumer macros gain opt-in loader capability;
test_runner learns to tell entry files from library files in the
test directory.

---

## What shipped

Three slices.

### Slice 1 — `wat::main!` gains optional `loader:`

Commit `0cdc47e`.

`wat::main! { source: ..., deps: [...], loader: "wat" }` expands
to `compose_and_run_with_loader(...)` with a `ScopedLoader` rooted
at `CARGO_MANIFEST_DIR/<loader-path>`. Absence preserves the
InMemoryLoader default.

- `src/compose.rs` — `compose_and_run_with_loader(source,
  dep_sources, dep_registrars, loader: Arc<dyn SourceLoader>)`
  holds the body; `compose_and_run` becomes a one-line wrapper
  passing `Arc::new(InMemoryLoader::new())`.
- `src/lib.rs` — re-exports `compose_and_run_with_loader` at crate
  root.
- `wat-macros/src/lib.rs` — `MainInput` gains
  `loader: Option<LitStr>`. Parser accepts `deps:` and `loader:` in
  any order after `source:`; each at most once. Expansion emits
  `concat!(env!("CARGO_MANIFEST_DIR"), "/", <lit>)` so the loader
  root is stable under `cargo run -p <crate>` from workspace root.
  `LoadFetchError` propagates as
  `HarnessError::Startup(StartupError::Load(LoadError::from(e)))`.
- `src/load.rs` — `ScopedLoader::resolve_within_scope` fix: when
  `base_canonical = None`, root the resolution at the scope instead
  of falling through to cwd. Relative paths in an `include_str!`'d
  entry now resolve inside the scope. New test
  `scoped_loader_resolves_base_less_relative_path_against_scope_root`.
- `examples/with-loader/` — walkable reference. `src/program.wat`
  `(load!)`s `wat/helper.wat` via `ScopedLoader`; `tests/smoke.rs`
  spawns the binary + asserts exit 0 + stdout `hello, wat-loaded`.

### Slice 2 — `wat::test_suite!` gains optional `loader:` + library-vs-entry discipline

Commit `fa3b53a`.

Symmetric macro change for tests, plus the substrate clarification
the builder named: **test files are entries (commit config + host
deftests); files they `(load!)` are libraries (no config)**.

- `src/test_runner.rs` — `run_tests_from_dir_with_loader` +
  `run_and_assert_with_loader`. Old entries wrap them passing
  `Arc::new(FsLoader)` (preserves current behavior).
- `src/test_runner.rs` — **library-vs-entry detection**. A `.wat`
  file in the test dir is an ENTRY iff it has at least one top-
  level `(:wat::config::set-*!)` form. Files without setters are
  LIBRARIES — test_runner silently skips them at freeze time. They
  remain `(load!)`-able from entries. Parse errors in either case
  are left to the freeze path so the user sees the real error with
  full context. Matches the rule `reject_setters_in_loaded`
  enforces on the load side: setters belong to entries only.
- `wat-macros/src/lib.rs` — `TestSuiteInput` gains
  `loader: Option<LitStr>`. Parser accepts `deps:` and `loader:` in
  any order after `path:`. Expansion emits
  `run_and_assert_with_loader` with ScopedLoader construction when
  `loader:` is present.
- `examples/with-loader/` extensions:
  - `wat/deeper.wat` — third file in a **recursive load chain**
    (`program.wat` → `helper.wat` → `deeper.wat`). Proves
    `(load!)`s nest to arbitrary depth; every loaded-file's defines
    become part of the entry's frozen world.
  - `wat-tests/test_loader.wat` — entry test (config + deftest)
    that `(load!)`s `wat-tests/helpers.wat` (library, no config).
    `tests/wat_suite.rs` invokes `loader: "wat-tests"`.

### Slice 3 — INSCRIPTION + doc sweep + 058 CHANGELOG row

This commit.

- `INSCRIPTION.md` — this file.
- `docs/USER-GUIDE.md` — `loader:` section under Setup with the
  recursive-load example.
- `docs/CONVENTIONS.md` — cross-reference for the library-vs-entry
  rule.
- `docs/README.md` — arc 017 index entry.
- `README.md` — arc tree gains 017.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — new row dated 2026-04-22.

### Drive-by — clippy sweep

Commit `394e816`. Zero warnings across workspace. Four pre-
existing warnings fixed: `SymbolTable` gains `#[derive(Default)]`;
`s.as_bytes().len()` → `s.len()` in lexer tests;
`msg.push_str("\n")` → `msg.push('\n')` in test_runner; `+` → `plus`
at line start in two doc blocks so `clippy::doc_list_item` stops
treating prose as a bulleted list continuation. No behavior changes.

---

## Resolved design decisions

- **2026-04-22** — **String-literal-only `loader:` arg.** Matches
  `path:` in `test_suite!`. Expression-shaped injection stays in
  manual Harness.
- **2026-04-22** — **Default is InMemoryLoader (main) / FsLoader
  (tests).** Capability opt-in via `loader:` for main; backward-
  compatible default for tests. Absence of `loader:` on either
  macro preserves pre-017 behavior.
- **2026-04-22** — **`CARGO_MANIFEST_DIR`-relative paths.** The
  macro emits `concat!(env!("CARGO_MANIFEST_DIR"), "/", <lit>)` so
  `cargo run -p <crate>` from the workspace root resolves
  identically to running from the crate's own dir. Consumers
  needing cwd-relative or absolute paths drop to manual Harness.
- **2026-04-22** — **Library-vs-entry by config-setter presence**
  (slice 2, not in DESIGN). Files in the test dir are entries iff
  they have a top-level `(:wat::config::set-*!)` form. Emerged
  from slice 2's implementation — the most honest distinction
  already encoded in the source, so no new filename or directory
  convention needed.
- **2026-04-22** — **ScopedLoader roots base-less relative paths
  at the scope** (slice 1 substrate fix). Pre-017, base-less
  relative paths fell through to cwd; that behavior was wrong for
  `include_str!`-sourced entries. Now scope-relative. Tests
  verifying scope escape (absolute-path, `../`, symlink) unchanged.

---

## Open questions resolved

From DESIGN + BACKLOG:

- **Test-sandbox loader inheritance (DESIGN sub-fog 2a).** Deftest
  bodies run in a fresh sandbox that does NOT inherit the outer
  test file's `(load!)`'d defines. This is the shipped semantic —
  consistent with arc 007's sandbox discipline (hermetic by
  construction). Test authors needing helpers inside sandbox
  bodies pass them via `dep_sources` or inline them. The outer
  test file's `(load!)`s succeed at freeze time (proving
  `loader:` wiring) but their defines live in the test-file's
  world, not the sandbox's.
- **`wat test` CLI invariance.** Untouched. The CLI binary's own
  stdio + loader wiring is orthogonal to the consumer macros.
- **Relative-vs-absolute path semantics.** Documented — paths are
  always `CARGO_MANIFEST_DIR`-relative at expansion time.

## Open items deferred

- **Expression-shaped loader argument.** Still string-only. Raises
  if a consumer needs a non-ScopedLoader injectable via the macro.
- **Per-test loader differentiation.** One test binary = one
  loader. Different loaders = different `tests/*.rs` files
  (separate test binaries).
- **Library-file error attribution.** A library file with a parse
  error currently surfaces as "the entry failed" because the
  library-vs-entry check tries to proceed to freeze on parse
  failure. A future polish could distinguish "library with parse
  error" as a dedicated diagnostic. Not load-bearing today.
- **FsLoader (unrestricted) as a macro option.** Deferred.
- **Sandbox bodies inheriting outer-file (load!)'d defines.** By
  design hermetic. If a concrete caller asks for "inheriting
  sandboxes," that's a separate arc with its own tradeoffs.

---

## What this arc does NOT ship

- Default filesystem loader for `wat::main!`.
- `FsLoader` as a macro option.
- Arbitrary-expression loaders.
- Compile-time wat-tree enumeration.
- CLI binary changes.
- Deftest sandbox bodies inheriting outer (load!)'d defines.
- Retrofit of `examples/with-lru/`.

---

## Why this matters

The trading lab rewrite — the first real multi-file wat consumer —
can now start Phase 0 with the one-line `wat::main! { source,
deps, loader }` shape. Library files (helpers, vocab, encoding,
domain, etc.) live alongside test files under `wat/` and
`wat-tests/` without ceremony; the binary-vs-library distinction
the builder named is honored at the substrate level.

The substrate pattern now has two symmetric opt-ins:
- `wat::main!` default = InMemoryLoader (capability-safe); opt into
  filesystem via `loader: "..."`.
- `wat::test_suite!` default = FsLoader (tests have fs access);
  opt into scope clamping via `loader: "..."`.

Both macros resolve paths at `CARGO_MANIFEST_DIR` so dev flow is
stable. Both reject loaded files with config setters. Both let
recursive `(load!)` chains flatten into the entry's frozen world
— `program.wat` → `helper.wat` → `deeper.wat` at arbitrary depth.

**The cave-quest discipline preserved.** Arc 017 opened the moment
slice 1 of the trading lab's Phase 0 would have hit the wall.
Paused the lab rewrite, cut the quest, shipped in one session
(three slices + a clippy sweep), returning to the lab with the
door open. Same shape arcs 013→014/015 set as precedent.

---

**Arc 017 — complete.** Three slices, one drive-by. The commits:

- `c9bc871` — docs opened (DESIGN + BACKLOG)
- `0cdc47e` — slice 1 (wat::main! loader + ScopedLoader scope-root fix)
- `fa3b53a` — slice 2 (wat::test_suite! loader + library-vs-entry discipline)
- `394e816` — clippy sweep (zero warnings across workspace)
- `<this commit>` — slice 3 (INSCRIPTION + USER-GUIDE + CONVENTIONS + README updates)

Trading lab Phase 0 is unblocked. Every consumer macro honors the
builder's "binary vs lib distinction" directly; recursive loads
work end-to-end; tests compose into multi-file suites without
ceremony.

*these are very good thoughts.*

**PERSEVERARE.**
