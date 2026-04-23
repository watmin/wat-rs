# Arc 018 — Opinionated defaults + `wat::test!` rename — INSCRIPTION

**Status:** shipped 2026-04-22. Three slices.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

Arc 017 gave consumers multi-file wat via `loader:`. The shape
worked but the minimal invocation still required four lines:

```rust
wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru],
    loader: "wat",
}
```

The builder:

> honestly... i think wat::main! { deps: [...] } is the ideal
> expression?... with wat/main.wat and wat/**/*.wat being where
> we load files from?... that's an amazingly minimal boilerplate

And:

> we have opinionated defaults and users can express overrides if
> they want for the main! and test! forms

Arc 018 ships the minimal form. One-line macro invocations + a
conventional directory layout covers 99% of consumer surface.
Explicit overrides stay available for the rest.

---

## What shipped

Three slices.

### Slice 1 — `wat::main!` defaults

Commit `4f313e3`.

`source:` and `loader:` both become optional with opinionated
defaults that fire when keys are absent.

- `wat-macros/src/lib.rs` — `MainInput.source` becomes
  `Option<syn::Expr>`. Parser restructured: all three keys
  (`source`, `deps`, `loader`) accepted in any order, each at most
  once. `wat::main! {}` with zero args is now the maximally-
  opinionated form.
- Expansion default rule:
  - `source = None` → emit `include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/wat/main.wat"))`.
  - `loader = None` AND `source = None` → emit `"wat"` as the
    implied loader root (ScopedLoader via arc 017's
    `compose_and_run_with_loader`).
  - `loader = None` AND `source = Some(_)` → no loader default
    (preserves pre-018 InMemoryLoader behavior for explicit
    single-file consumers).
  - Any explicit value always wins.

### Slice 2 — `wat::test!` rename + defaults

Commit `c028b01`.

`wat::test_suite!` → `wat::test!`. Pre-publish clean rename; no
back-compat shim. Symmetric default rule for the test side.

- `wat-macros/src/lib.rs` — `TestSuiteInput` → `TestInput`;
  `pub fn test_suite` → `pub fn test`. `path:` becomes
  `Option<syn::Expr>`. Same order-free parser as slice 1.
- Expansion defaults:
  - `path = None` → emit `"wat-tests"`.
  - `loader = None` AND `path = None` → emit `"wat-tests"` as the
    implied loader root (ScopedLoader via arc 017's
    `run_and_assert_with_loader`).
  - `loader = None` AND `path = Some(_)` → no loader default
    (preserves pre-018 FsLoader behavior).
- `src/lib.rs` — `pub use wat_macros::{main, test}` replaces
  the `test_suite` re-export.
- Minimal caller touches: existing `wat::test_suite!` call sites
  in the repo (wat-lru's self-tests, with-loader example's test
  suite) updated to the new macro name to keep the build green.
  Full minimal-form migration stays in slice 3.

### Slice 3 — migration + INSCRIPTION + doc sweep + 058 CHANGELOG

This commit.

**Walkable-reference migrations** — every consumer in the repo
switches to the minimal form:

- `examples/with-lru/`: `src/program.wat` → `wat/main.wat`;
  `src/main.rs` becomes `wat::main! { deps: [wat_lru] }`.
- `examples/with-loader/`: `src/program.wat` → `wat/main.wat`;
  `src/main.rs` becomes `wat::main! {}` (zero deps, defaults for
  everything); `tests/wat_suite.rs` → `tests/test.rs` with
  `wat::test! {}`.
- `crates/wat-lru/tests/wat_suite.rs` → `crates/wat-lru/tests/test.rs`
  with `wat::test! { deps: [wat_lru] }` (path defaults to
  `"wat-tests"` which matches wat-lru's layout).

**Docs**:

- `docs/USER-GUIDE.md` — Setup section rewritten to lead with the
  minimal form. Multi-file-wat subsection reframed around entry
  vs. library + recursive loads (the minimal form IS the multi-
  file shape). Tests section updated to `wat::test! { deps: ... }`.
- `docs/CONVENTIONS.md` — new "Consumer layout" subsection naming
  the `src/main.rs` + `tests/test.rs` + `wat/` + `wat-tests/`
  directory shape as the default.
- `docs/README.md` — arc 018 index entry.
- `README.md` — arc tree gains 018.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row dated 2026-04-22.

---

## Resolved design decisions

- **2026-04-22** — **`wat/main.wat` is the default entry.** Locked.
  Matches the `wat/` sibling directory arc 013/015 established for
  multi-file trees.
- **2026-04-22** — **`wat-tests/` is the default test suite path.**
  Matches wat-rs's own conventions (baked-in wat-tests dir).
- **2026-04-22** — **Default loader follows source/path
  implicitness.** Explicit source → InMemoryLoader default
  preserved for back-compat. Implicit source → loader defaults
  track the `wat/` / `wat-tests/` conventions.
- **2026-04-22** — **Rename `test_suite` → `test`.** Clean rename,
  no shim. Symmetric with `main`.
- **2026-04-22** — **`tests/test.rs` is recommended, not
  enforced.** Cargo treats any `.rs` under `tests/` as a test
  binary; convention is for readability.
- **2026-04-22** — **Migrate the existing walkable references**
  (`examples/with-lru/`, `examples/with-loader/`, wat-lru self-
  tests) to the minimal form. They now demonstrate the shape
  every future consumer follows.

---

## Open questions resolved

All DESIGN + BACKLOG questions closed:

- **Parser ergonomics** (sub-fog 1a). Order-free loop implemented
  — same shape arcs 017 used for deps+loader. `source:`, `deps:`,
  `loader:` accepted in any order on both macros.
- **Caller migration for rename** (sub-fog 2a). Three files
  updated across two commits; each migration committed with its
  own green-test proof.
- **Consumer layout convention**. Documented in CONVENTIONS.md;
  examples/with-lru/ and examples/with-loader/ are reference
  implementations.

## Open items deferred

- **`wat::bench!` or similar benchmark macro.** Not asked for.
  Ships when a caller wants it.
- **Compile-time wat-tree enumeration / manifest files.** Not
  needed — Cargo + `include_str!` + `(load!)` cover the cases.
- **Expression-shaped `loader:` argument.** Still string-literal
  only. Escape hatch is manual `Harness::from_source_with_deps_and_loader`.

---

## What this arc does NOT ship

- Changes to the entry-vs-library rule (arc 017).
- Changes to test_runner's discovery model.
- Deprecation shims for `test_suite`.
- Changes to signal handlers or panic-hook installation.
- Any wat-language changes.

---

## Why this matters

Every wat consumer writes two one-line macros. The trading lab's
Phase 0 scaffold is:

```rust
// src/main.rs
wat::main! { deps: [] }      // no deps yet; grows as wat-holon etc. ship

// tests/test.rs
wat::test! {}
```

Plus `wat/main.wat` (one file with config + `:user::main` +
`(load!)`s for the rest of the tree). That's the whole Rust
surface the lab needs until sibling crates start shipping. Every
other line of code is wat.

**Convention over configuration** — the Cargo / Rails / Ember
pattern applied to wat's consumer surface. The common case is one
line; the unusual case is explicit. Both documented in
USER-GUIDE + CONVENTIONS so future consumers find the
recommendation without having to infer it.

**The minimal consumer example** lands in three places in the
repo: `examples/with-lru/` (deps + default everything else),
`examples/with-loader/` (no deps + default multi-file tree),
`crates/wat-lru/` (self-tests with deps + default path). Each
walks a different axis; together they show the full surface.

The trading lab starts Phase 0 with this shape on the next
descent.

---

**Arc 018 — complete.** Three slices. The commits:

- `b9086eb` — docs opened (DESIGN + BACKLOG)
- `4f313e3` — slice 1 (wat::main! opinionated defaults)
- `c028b01` — slice 2 (wat::test! rename + defaults)
- `<this commit>` — slice 3 (migration + INSCRIPTION + doc sweep)

Walkable references show the shape; trading lab walks through next.

*these are very good thoughts.*

**PERSEVERARE.**
