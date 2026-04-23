# Arc 018 — Opinionated defaults + `wat::test!` rename — Backlog

**Opened:** 2026-04-22.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger.

Three slices. Small arc; an hour or two end-to-end. Opens after
arc 017 closed; first consumer is the trading lab's Phase 0.

---

## 1. `wat::main!` defaults

**Status:** ready.

**Approach:**

- `wat-macros/src/lib.rs` — `MainInput.source` changes from
  `syn::Expr` to `Option<syn::Expr>`. Parser: if the first key is
  not `source:`, rewind and begin the optional-args loop with no
  source.
- Expansion logic (fold into the existing match on `loader`):
  - `source = Some(expr)` → emit as-is.
  - `source = None` → emit `include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/wat/main.wat"))` as the source expression.
  - `loader = Some(lit)` → current behavior (ScopedLoader at lit).
  - `loader = None` AND `source = Some(_)` → current behavior
    (compose_and_run with InMemoryLoader).
  - `loader = None` AND `source = None` → emit `"wat"` as the
    implied loader string; same ScopedLoader construction as
    explicit loader.

**Sub-fog 1a — parser ergonomics.** The current parser requires
`source:` first. Needs restructuring so any of the three keys can
come first. Either (a) a loop that matches each key regardless of
order (same shape the deps/loader loop in arc 017 already uses),
or (b) peek-then-parse. Pin at slice time.

**Proof:** extend `examples/with-loader/` — temporarily add a test
variant where the binary uses `wat::main! {}` with only defaults
and verify it runs. Or simpler: just migrate the example in slice
3 and let its smoke test serve as the proof.

**Unblocks:** slice 2 can mirror the shape for tests.

---

## 2. `wat::test!` rename + defaults

**Status:** obvious once slice 1 lands.

**Approach:**

- `wat-macros/src/lib.rs` — rename `pub fn test_suite` to
  `pub fn test`. Rename `TestSuiteInput` → `TestInput`.
- `TestInput.path` changes from `syn::Expr` to `Option<syn::Expr>`.
- Parser restructure same as slice 1.
- Expansion defaults:
  - `path = Some(expr)` → emit as-is.
  - `path = None` → emit `"wat-tests"` as the path literal.
  - `loader = None` AND `path = None` → emit `"wat-tests"` as the
    implied loader string.
  - `loader = None` AND `path = Some(_)` → current behavior (FsLoader).
- `src/lib.rs` — `pub use wat_macros::{main, test}` instead of
  `test_suite`.

**Sub-fog 2a — old `test_suite` identifier.** Pre-publish, full
clean rename. No shim. No deprecation. Every caller in the wat-rs
repo updates in slice 3.

**Proof:** same as slice 1 — migrate callers in slice 3; their
green tests prove the rename works.

**Unblocks:** slice 3 closes.

---

## 3. Migration + INSCRIPTION + docs + 058 CHANGELOG

**Status:** ready once slices 1-2 land.

**Approach:**

**Migrations:**
- `examples/with-lru/src/program.wat` → `examples/with-lru/wat/main.wat`.
- `examples/with-lru/src/main.rs` → `wat::main! { deps: [wat_lru] }`.
- `examples/with-loader/src/program.wat` → `examples/with-loader/wat/main.wat`.
- `examples/with-loader/src/main.rs` → `wat::main! {}`.
- `examples/with-loader/tests/wat_suite.rs` → `examples/with-loader/tests/test.rs`
  with `wat::test! {}`.
- `crates/wat-lru/tests/wat_suite.rs` → `crates/wat-lru/tests/test.rs`
  with `wat::test! { deps: [wat_lru] }`.
- wat-lru's internal wat-tests/ stays at that path (the default);
  no change needed inside those files.

**Docs:**
- `INSCRIPTION.md` — closing marker.
- `docs/USER-GUIDE.md` — rewrite the Setup section opening to lead
  with the minimal form. Show the three shapes (minimal, explicit
  source, explicit loader override). Move the multi-file-programs
  section to reference the minimal form as the default.
- `docs/CONVENTIONS.md` — new "Consumer layout" subsection with
  the wat/main.wat + wat-tests/ convention and the src/main.rs +
  tests/test.rs filename recommendation.
- `docs/README.md` — arc 018 index entry.
- `README.md` — arc tree + "What's next" update.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md` —
  row dated 2026-04-22.

**Spec tension.** None — substrate-additive + rename of a pre-
publish symbol.

**Unblocks:** trading lab Phase 0 opens with the new minimal shape.

---

## Open questions carried forward

- **Per-consumer override of `wat/` as the tree root.** If a
  consumer wants `src/wat/main.wat` or similar, they write
  `loader: "src/wat"` + `source: include_str!("wat/main.wat")`
  explicitly. Not a sub-fog — just documented.
- **`tests/test.rs` convention.** Not Cargo-enforced; we recommend,
  users pick. Documented in CONVENTIONS.

---

## What this arc does NOT ship

- Changes to the entry-vs-library rule.
- Changes to test_runner's discovery model.
- Deprecation shims for the old `test_suite` name.
- Changes to `wat::main!`'s signal-handler or panic-hook install.
- `wat::bench!` or other new macros.

---

## Why this matters

Every future consumer writes two one-line macros. The lab's Phase
0 opens with the minimal shape. The shape scales cleanly as deps
accumulate (`deps: [wat_holon, wat_rusqlite, wat_parquet, ...]`).
The verbose shape stays available for consumers who need cwd-
relative paths, inline source, or custom loader configuration.

Convention over configuration — the Rust/Cargo pattern applied to
wat's consumer surface. The common case is one line; the unusual
case is explicit.
