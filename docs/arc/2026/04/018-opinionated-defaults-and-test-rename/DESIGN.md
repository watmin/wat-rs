# Arc 018 — Opinionated defaults + `wat::test!` rename

**Status:** opened 2026-04-22.
**Motivation:** arc 017 gave consumers multi-file wat via `loader:`
but the minimal form was still verbose — `wat::main! { source:
include_str!("program.wat"), deps: [...], loader: "wat" }`.
Four-line declarations for what should be one line.

The builder:
> honestly... i think wat::main! { deps: [...] } is the ideal
> expression?... with wat/main.wat and wat/**/*.wat being where
> we load files from?... that's an amazingly minimal boilerplate

Arc 018 ships the opinionated-defaults shape. Convention over
configuration for the common case; explicit overrides for the
unusual one.

---

## UX target (locked before design)

### Minimal consumer — the new happy path

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs        → wat::main! { deps: [...] }
├── tests/
│   └── test.rs        → wat::test! { deps: [...] }
├── wat/
│   ├── main.wat       → entry (config + :user::main)
│   └── **/*.wat       → library tree
└── wat-tests/
    └── **/*.wat       → test files
```

Two single-line macro invocations. Everything else is wat.

### Defaults

**`wat::main!`**
- `source:` omitted → `include_str!(<crate-root>/wat/main.wat)`
- `loader:` omitted AND `source:` omitted → `"wat"` (ScopedLoader)
- `loader:` omitted AND `source:` explicit → no loader (InMemoryLoader — current behavior preserved)
- Any explicit value always wins

**`wat::test!`**
- `path:` omitted → `"wat-tests"`
- `loader:` omitted AND `path:` omitted → `"wat-tests"` (ScopedLoader)
- `loader:` omitted AND `path:` explicit → no loader (FsLoader — current behavior preserved)
- Any explicit value always wins

### Rename

`wat::test_suite!` → `wat::test!`. Pre-publish; no back-compat
shim. Rationale: symmetric with `wat::main!` (noun-verb ending in
the word the macro is about); shorter at every call site.

---

## Non-goals

- **Changing what wat/main.wat must contain.** It's still the
  entry file: commits config, defines `:user::main`. The rule for
  entry-vs-library (arc 017) is unchanged.
- **Adding a way to skip `wat/` entirely.** If a consumer genuinely
  doesn't want a wat tree, they write `wat::main! { source:
  "(:wat::config::set-dims! 1024) ..." }` with inline source — the
  explicit path. No new "inline-only" mode.
- **Changing test_runner's library-vs-entry detection.** Files in
  the test directory without config setters are still libraries.
- **Retrofit of `tests/wat_suite.rs` → `tests/test.rs` in wat-rs's
  own repo.** We migrate consumer examples (with-lru, with-loader)
  + wat-lru's self-tests to the new shape as part of slice 3. The
  filename is a consumer choice; the convention `tests/test.rs` is
  recommended but not enforced.

---

## What this arc ships

Three slices.

### Slice 1 — `wat::main!` defaults

- `wat-macros/src/lib.rs` — `MainInput` parser accepts zero args
  (e.g. `wat::main! { deps: [wat_lru] }` with no `source:`).
  `source: Option<syn::Expr>` replaces the required field.
- Macro expansion logic:
  - `source` present → use as-is; `loader` defaults stay as pre-
    018 (absent = InMemoryLoader).
  - `source` absent → emit `include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/wat/main.wat"))` as source; `loader` defaults to `"wat"`
    (ScopedLoader) when not explicit.
- Preserves all back-compat for explicit `source:` callers.

### Slice 2 — `wat::test!` rename + defaults

- `wat-macros/src/lib.rs` — `pub fn test(...)` replacing
  `pub fn test_suite(...)`. `TestInput` replaces `TestSuiteInput`.
- `path: Option<syn::Expr>` replaces the required field. When
  absent, emit `"wat-tests"`. Same loader-default rule as main.
- `src/lib.rs` — re-export changes from `test_suite` to `test`.

### Slice 3 — migration + INSCRIPTION + doc sweep + 058 CHANGELOG

- Migrate `examples/with-lru/` to the minimal form:
  - `src/program.wat` moves to `wat/main.wat`.
  - `src/main.rs` becomes `wat::main! { deps: [wat_lru] }`.
  - `tests/smoke.rs` unchanged (spawns the binary).
- Migrate `examples/with-loader/` to the minimal form:
  - `src/program.wat` moves to `wat/main.wat`.
  - Existing recursive chain (wat/helper.wat → wat/deeper.wat) unchanged.
  - `src/main.rs` becomes `wat::main! {}` (zero deps).
  - `tests/wat_suite.rs` → `tests/test.rs` with `wat::test! {}` (defaults).
- Migrate `crates/wat-lru/tests/wat_suite.rs` → `tests/test.rs`
  with `wat::test! { deps: [wat_lru] }`.
- INSCRIPTION.
- USER-GUIDE — rewrite Setup section to lead with the minimal
  form; verbose form as override.
- CONVENTIONS — add the `wat/main.wat` + `wat-tests/` convention
  under a new "Consumer layout" subsection.
- `docs/README.md` arc 018 entry.
- `README.md` arc tree gains 018.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  row dated 2026-04-22.

---

## Resolved design decisions

- **2026-04-22** — **`wat/main.wat` is the entry default.** Locked.
  Matches the sibling `wat/` directory already documented in
  CONVENTIONS for multi-file trees.
- **2026-04-22** — **`wat-tests/` is the test-suite default.**
  Matches the convention wat-rs's own tests already use.
- **2026-04-22** — **Default loader follows source implicitness.**
  Source explicit → InMemoryLoader stays the default (back-compat).
  Source implicit → loader defaults to the same directory
  (ScopedLoader at `wat/` for main, ScopedLoader at `wat-tests/`
  for tests).
- **2026-04-22** — **Rename: `test_suite` → `test`.** Pre-publish,
  no back-compat shim. Symmetric with `main`.
- **2026-04-22** — **Migrate existing examples + wat-lru self-
  tests** to demonstrate the minimal form. Shows the walkable
  shape every future consumer follows.

---

## Open questions to resolve as slices land

- **Filename convention `tests/test.rs`.** Recommended, not
  enforced by Cargo. Document in USER-GUIDE + CONVENTIONS; don't
  police. Matches the `wat::test!` macro name symmetrically.
- **Consumers with no wat tree at all.** They can still write
  `wat::main! { source: "<inline source>" }` — the explicit-source
  path preserves the InMemoryLoader default.
- **Back-compat for external crates using `test_suite!`.**
  Pre-publish; only wat-rs's own crates use the name (wat-lru
  self-tests, examples/with-loader). All updated in slice 3.

---

## What this arc does NOT ship

- Changes to the entry-vs-library rule.
- Changes to test_runner's discovery model.
- Compile-time wat-tree enumeration or manifest files.
- A `wat::bench!` macro or similar.
- Changes to `wat::main!`'s signal-handler installation.
- Migration of any lab code (the trading lab's Phase 0 is the
  FIRST consumer of the new shape — it doesn't yet exist).

---

## The thread this continues

Arc 013 shipped consumer macros with explicit args. Arc 015 closed
the test side. Arc 017 added `loader:` for multi-file consumers.
Arc 018 makes the multi-file shape the opinionated default — the
common consumer writes two one-line macros total.

The trading lab's Phase 0 opens with `wat::main! { deps: [] }` +
`wat::test! { deps: [] }` + `wat/main.wat` + `wat-tests/**/*.wat`.
As sibling crates ship (wat-holon, wat-rusqlite, wat-parquet), the
deps list grows — the rest stays identical. Opinionated defaults
for the path every wat consumer walks.
