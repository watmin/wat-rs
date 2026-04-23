# Arc 017 — Loader option for consumer macros

**Status:** opened 2026-04-22.
**Motivation:** `wat::main!` hard-wires `InMemoryLoader::new()`
via `compose_and_run` (`compose.rs:118-122, 160`), so
`(:wat::load-file! "path")` from inside a consumer's wat program
returns `NotFound`. The ~10,000-LoC trading lab — the first real
multi-file consumer — cannot live in one inline `program.wat`.
This arc gives the consumer macros the `ScopedLoader` surface
already exposed by `Harness::from_source_with_deps_and_loader`.

Builder framing (2026-04-22):
> there's always an unexpected quest - the dungeon master provides

First consumer after shipping: the trading lab rewrite (Phase 0 of
`holon-lab-trading/docs/rewrite-backlog.md`).

---

## UX target (locked before design)

Today:

```rust
wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru],
}
```

After this arc:

```rust
wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru],
    loader: "wat",          // optional — string, ScopedLoader root
}
```

Absence preserves today's behavior (InMemoryLoader, no filesystem).
Presence expands to `::wat::load::ScopedLoader::new(<path>)?` and
threads into the loader-capable harness path. Bad paths surface as
`HarnessError::Startup` from `fn main()`.

Same symmetric shape for `wat::test_suite!`:

```rust
wat::test_suite! {
    path: "wat-tests",
    deps: [wat_lru],
    loader: "wat",
}
```

### Why a string literal, not an arbitrary expression

Matches `path:` in `wat::test_suite!`. Consumers needing `FsLoader`
(unrestricted) or a custom `SourceLoader` impl drop to the manual
`Harness::from_source_with_deps_and_loader` escape hatch — the
same escape that exists today. If real consumer demand for
expression-shaped injection surfaces, a follow-up arc extends.

---

## Non-goals

- **Default filesystem loader.** InMemoryLoader stays the default —
  absence of `loader:` = zero filesystem reach. `loader: "..."` is
  the explicit capability declaration.
- **`FsLoader` as a macro option.** Unrestricted filesystem access
  is not exposed here. Manual Harness is the path if needed.
- **Arbitrary-expression loaders in the macro.** String-only this
  arc.
- **Compile-time wat-tree enumeration.** Build-script territory,
  not substrate.
- **Retrofit of `examples/with-lru/`.** Its inline `program.wat`
  doesn't need a loader. Stays as-is.

---

## What this arc ships

Three slices.

### Slice 1 — `wat::main!` gains optional `loader:`

- `wat-macros/src/lib.rs` — `MainInput` parser accepts
  `loader: <string-literal>` after `deps:`. When present, macro
  emits `::wat::compose_and_run_with_loader(source, &[...], &[...],
  ::wat::load::ScopedLoader::new(<path>).map(::std::sync::Arc::new)?)`.
  When absent, unchanged — current `compose_and_run` call.
- `src/compose.rs` — new public
  `compose_and_run_with_loader(source, dep_sources, dep_registrars,
  loader)`. `compose_and_run` becomes a thin wrapper calling
  `compose_and_run_with_loader(..., Arc::new(InMemoryLoader::new()))`.
- Integration proof: new `examples/with-loader/` workspace member
  with a multi-file wat tree (`wat/main.wat` `load!`s
  `wat/helper.wat`). Smoke test asserts the binary runs and prints
  expected output.

### Slice 2 — `wat::test_suite!` gains optional `loader:`

- `wat-macros/src/lib.rs` — `TestSuiteInput` parser accepts
  `loader:` symmetric with main.
- `src/test_runner.rs` — `run_tests_from_dir` gains a loader
  parameter; threads ScopedLoader into each sandboxed test's
  freeze.
- Test: extend `examples/with-loader/`'s `tests/tests.rs` to
  exercise a test-side `(load!)`.

### Slice 3 — INSCRIPTION + doc sweep + 058 CHANGELOG row

- `docs/arc/2026/04/017-loader-option-for-consumer-macros/INSCRIPTION.md`.
- `docs/USER-GUIDE.md` — extend Setup section with the `loader:`
  option and an example; note capability-opt-in framing.
- `docs/CONVENTIONS.md` — one-line cross-reference.
- `docs/README.md` — arc 017 index entry.
- `README.md` — arc tree + "What's next" update.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` — row documenting arc 017.

---

## Resolved design decisions

- **2026-04-22** — **String-literal-only arg.** Matches `path:` in
  `test_suite!`. Expression-shaped injection stays in manual Harness.
- **2026-04-22** — **Default is InMemoryLoader.** Capability-opt-
  in — absence of `loader:` preserves zero-filesystem-reach shape.
- **2026-04-22** — **`?`-chained expansion, not `unwrap`.** Bad
  paths surface as `HarnessError::Startup` — same shape every
  startup failure uses.
- **2026-04-22** — **Both consumer macros get the arg.** Symmetric;
  the trading lab needs both for main and tests.

---

## Open questions to resolve as slices land

- **Test-sandbox loader inheritance.** Inside a test run through
  `deftest`, does the inner sandbox's `(load!)` see the
  `test_suite!`-configured loader? The sandbox creates a fresh
  freeze via `startup_from_forms`; verify the new path threads
  correctly. Pin at slice 2.
- **`wat test` CLI invariance.** The standalone CLI
  (`src/bin/wat.rs`) already takes a `<path>` arg and has its own
  loader wiring. This arc doesn't touch it — confirm at slice 3.
- **Relative-vs-absolute path semantics.** `ScopedLoader::new("wat")`
  canonicalizes against binary cwd. For `cargo run -p my-app`, cwd
  is workspace root. Document at slice 3.

---

## What this arc does NOT ship

- `FsLoader` as a macro option.
- Arbitrary-expression loaders in the macro.
- Compile-time wat-tree enumeration.
- Changes to the existing `wat test <path>` CLI's loader.
- Retrofit of `examples/with-lru/`.

---

## The thread this continues

Arc 013 shipped the consumer-shape macros (`wat::main!`,
`wat::test_suite!`). Arc 015 closed the test-side of external-crate
composition. Arc 016 closed failure ergonomics. Arc 017 closes the
multi-file-consumer gap — the first real consumer (trading lab)
needs it, and the substrate deserves the cleanup before the lab
writes 15 lines of boilerplate at every consumer's main.

The cave-quest discipline from arcs 013 → 014 → 015 applies: when
a slice surfaces real substrate debt that blocks honest progress,
pause, name the key, cut the quest arc, return. This one's named
before the lab's Phase 0 slice even opened — the lab can't start
until 017 ships.
