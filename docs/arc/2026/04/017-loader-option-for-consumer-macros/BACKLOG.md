# Arc 017 — Loader option for consumer macros — Backlog

**Opened:** 2026-04-22.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger.

Arc 017 adds optional `loader: <string>` to `wat::main!` and
`wat::test_suite!`. String = `ScopedLoader` root. Absence =
existing `InMemoryLoader` default. First consumer: the trading
lab.

---

## 1. `wat::main!` gains optional `loader:`

**Status:** ready.

**Approach:**

- `src/compose.rs` — add `compose_and_run_with_loader(source:
  &str, dep_sources: &[&'static [WatSource]], dep_registrars:
  &[DepRegistrar], loader: Arc<dyn SourceLoader>) ->
  Result<(), HarnessError>`. Factor the existing `compose_and_run`
  body into it; the old function becomes a one-liner forwarder
  defaulting to `Arc::new(InMemoryLoader::new())`.
- `wat-macros/src/lib.rs` — `MainInput` gains an optional
  `loader: Option<syn::LitStr>` field after `deps:`. Parser reads
  the key if present; error at parse time if the key appears but
  value is not a `LitStr`.
- Macro expansion:
  - If `loader` absent: current expansion (calls
    `compose_and_run`).
  - If `loader` present: expansion constructs the loader via
    `::wat::load::ScopedLoader::new(<path>).map(::std::sync::Arc::new)`
    — a `Result<Arc<ScopedLoader>, LoadFetchError>`; the macro
    threads a `?` to propagate, mapping the LoadFetchError to
    `HarnessError::Startup` via an `Into` impl (or a manual map if
    that doesn't exist yet). Then calls
    `compose_and_run_with_loader(source, &[...], &[...], loader)`.

**Sub-fog 1a — `LoadFetchError` → `HarnessError` conversion.**
Check whether the existing `HarnessError` enum has a `From<LoadFetchError>`
impl. If not, either add one (keep the macro output clean) or
emit an explicit `.map_err(...)` in the expansion. Both are fine;
pick whichever keeps the macro output readable.

**Sub-fog 1b — loader-capable rust_deps install.** `compose_and_run`
installs rust_deps via `rust_deps::install(builder.build())`
before `startup_from_source`. Confirm this remains single-call
semantically (first-call-wins OnceLock) when the caller uses
`compose_and_run_with_loader` instead. Install path should be
shared between the two functions — factor once, call from both.

**Proof:** new `examples/with-loader/` workspace member. Minimal
multi-file wat tree:
- `wat/main.wat` — entry. `(:wat::core::load! "types.wat")`,
  defines `:user::main`, prints a value pulled from the loaded
  types module.
- `wat/types.wat` — stdlib-tier defines for the main file to
  reference.
- `src/main.rs` — one `wat::main! { source: include_str!("program.wat"),
  loader: "wat" }`. (Actually — `source:` would be the entry; does
  that include `main.wat` or the initial `(:wat::core::load!
  "main.wat")`? Clarify at slice time.)
- `tests/smoke.rs` — spawn the binary, assert expected stdout.

**Unblocks:** slice 2 can mirror the shape for `test_suite!`.

---

## 2. `wat::test_suite!` gains optional `loader:`

**Status:** obvious once slice 1 lands.

**Approach:**

- `wat-macros/src/lib.rs` — `TestSuiteInput` parser accepts
  `loader:` symmetric with `MainInput`.
- `src/test_runner.rs` — `run_tests_from_dir` gains a
  `loader: Arc<dyn SourceLoader>` parameter. Threads into each
  test's freeze instead of the current implicit default. New
  variant `run_tests_from_dir_with_loader` OR extend the existing
  signature (the existing one has few callers — extending is
  probably cleaner).
- Macro expansion: symmetric with `wat::main!`. Absence = current
  behavior; presence = ScopedLoader-threaded test freezes.

**Sub-fog 2a — deftest sandbox loader inheritance.** A `deftest`
expansion calls `run-sandboxed-ast` which creates a nested
sandbox via `startup_from_forms`. Verify the outer test's loader
threads into the inner sandbox — or if not, document the scope
of the loader as "outer freeze only; inner sandbox still uses
`InMemoryLoader` unless the test explicitly provides one." Pin
the semantics at slice time.

**Sub-fog 2b — one binary = one loader.** `test_suite!` currently
emits one `#[test] fn wat_suite` per invocation. Each `cargo test`
binary therefore has one test_suite = one loader config. If two
test files need different loaders in the same crate, they'd
expect to be in separate test binaries. Note at doc time if it
matters.

**Proof:** extend `examples/with-loader/tests/tests.rs` to test a
`(load!)` inside a deftest. 1-2 wat test files under
`examples/with-loader/wat-tests/`.

**Unblocks:** slice 3 closes.

---

## 3. INSCRIPTION + docs + 058 CHANGELOG row

**Status:** ready once slices 1-2 land.

**Approach:**

- `INSCRIPTION.md` — closing marker. What shipped, slice by slice,
  commit refs. Resolved open questions. Deferred items. Same
  shape as prior INSCRIPTIONs.
- `docs/USER-GUIDE.md` — extend Setup section (§1 "your first wat
  application crate") with the `loader:` option. Example: tree
  layout with `wat/main.wat` + `wat/helper.wat` + `src/main.rs`
  with `loader: "wat"`. Frame as capability opt-in. Show both
  shapes — absence (baked-stdlib only) and presence (filesystem
  access rooted at a directory).
- `docs/CONVENTIONS.md` — cross-reference under the External-wat-
  crates section OR under a new "Loader capability" subsection if
  the content warrants.
- `docs/README.md` — arc 017 entry in the arc list.
- `README.md` — arc tree gains 017; "What's next" bullet updated
  if relevant.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` — new row dated 2026-04-22
  documenting arc 017.

**Spec tension.** None — substrate-additive, no breaking change.

**Unblocks:** trading lab's Phase 0 can proceed with the new
one-line shape.

---

## Open questions carried forward

- **Inner-sandbox loader inheritance (sub-fog 2a).** If resolved
  as "loader outer-only," pointable at a future arc to extend.
- **Expression-shaped loader argument.** Deferred. Surface if a
  consumer needs a non-ScopedLoader injectable via the macro.
- **Per-test loader differentiation.** Deferred (sub-fog 2b). One
  test binary = one loader; separate loaders = separate binaries.

---

## What this arc does NOT ship

- Default filesystem loader.
- `FsLoader` as a macro option.
- Arbitrary-expression loaders.
- Compile-time wat-tree enumeration.
- CLI binary changes.

---

## Why this matters

The trading lab rewrite — the first real multi-file wat consumer —
cannot start Phase 0 without this arc. Cave-quest discipline (arc
013/014/015 precedent) applies: pause the downstream work, cut
the substrate quest, ship, return. 017 opens before 0.1 opens.

The broader pattern: every "absence is signal" gap gets closed the
same way. `wat::main!` defaulting to `InMemoryLoader` is safe but
surface-limiting. Adding the opt-in capability keeps the safe
default, makes the capable shape ergonomic, and preserves the
manual-Harness escape hatch for the unusual cases.
