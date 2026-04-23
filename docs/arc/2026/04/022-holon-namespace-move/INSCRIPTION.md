# Arc 022 — holon namespace consolidation — INSCRIPTION

**Status:** shipped 2026-04-22. Two slices, same day.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

Three parallel forms held holon-related surface with no
principled distinction:

- `:holon::HolonAST` — the algebra's universal value type, at the
  wat level, under a private namespace.
- `:wat::algebra::*` — the primitives that produce it + scalar
  measurements that read it.
- `wat/std/<HolonThing>.wat` + `:wat::std::<HolonThing>` — the
  wat-written algebra idioms (Circular, Log, Sequential,
  Subtract, Amplify, Ngram, Bigram, Trigram, Reject, Project),
  living under the stdlib namespace even though they ARE holon
  algebra expressed on the core primitives.

Builder framing, three moments:

> i think we need to move the holon things into :wat::holon::*
>
> i think algebra -> holon -- all the holon things should be in holon
>
> so i think we need wat/std/&lt;holon-thing&gt;.wat -> wat/holon/*.wat

One namespace for one concept. File path matches namespace.

---

## Slice 1 — the namespace rename

Commit `1fa7ba9`. Two mechanical renames across every file type.

### Rename 1 — the type

- `:holon::HolonAST` → `:wat::holon::HolonAST` at every wat-level
  use (keyword contexts, `:Vec<...>` / `:Result<...>` / `:AST<...>`
  type parameters, return types, struct fields).

The Rust crate name stays `holon` (the `holon-rs` crate). The
Rust identifier `holon::HolonAST` is preserved at its `use`
sites — only the wat-level path renames.

### Rename 2 — the algebra-core namespace

The `:wat::algebra::*` namespace retires; every member moves to
`:wat::holon::*`:

**AST-producing primitives** (6):
- `Atom`, `Bind`, `Bundle`, `Blend`, `Permute`, `Thermometer`

**Measurements** (3):
- `cosine`, `dot`, `presence?`

**Error type** (1):
- `CapacityExceeded`

No compatibility shim — pre-publish clean break.

### Reserved-prefix gate

- `src/resolve.rs::RESERVED_PREFIXES` — `:wat::algebra::` entry
  retired, `:wat::holon::` entry present.

---

## Slice 2 — the stdlib rename + file reorganization

Same day as slice 1. Builder's final directive: move every wat
file that expresses holon algebra into `wat/holon/`, rename the
forms to `:wat::holon::*` so path matches namespace, mirror the
move in `wat-tests/`.

### File moves

- `wat/std/{Amplify,Bigram,Circular,Log,Ngram,Project,Reject,Sequential,Subtract,Trigram}.wat`
  → `wat/holon/*.wat` (10 files)
- `wat-tests/std/{Circular,Reject,Sequential,Subtract,Trigram}.wat`
  → `wat-tests/holon/*.wat` (5 files)

### Namespace rename

- `:wat::std::{Amplify,Bigram,Circular,Log,Ngram,Project,Reject,Sequential,Subtract,Trigram}`
  → `:wat::holon::*` at every caller (src/, wat/, wat-tests/,
  tests/, docs/).

- Test-name prefixes swept in the moved files:
  `:wat-tests::std::<Name>::test-*` → `:wat-tests::holon::<Name>::test-*`.

### Baked-in paths

`src/stdlib.rs` — `STDLIB_FILES` include_str! paths updated to
`wat/holon/*.wat` for the ten algebra files. Non-algebra files
(`stream.wat`, `hermetic.wat`, `test.wat`, `service/Console.wat`)
stay under `wat/std/` — they're genuine stdlib, not holon
algebra.

### What stays at `:wat::std::*`

The algebra idioms moved because they compose `:wat::holon::*`
primitives. Genuine stdlib stays at `:wat::std::*` — stream
combinators (`:wat::std::stream::*`), the test harness
(`:wat::test::*`), services (`:wat::std::service::*`), hermetic
sandboxing (`:wat::std::hermetic::*`). These aren't holon
algebra; they're plumbing built on kernel / io / core.

---

## New in this arc — cargo-native wat-test coverage

Builder surfaced mid-slice-2 that `cargo test` was only covering
wat-tests/ indirectly via `tests/wat_test_cli.rs` (which spawns
the built binary). Added `tests/test.rs` with `wat::test! {}` —
the arc-018 opinionated-defaults form — so wat-rs exercises its
own wat tests in-process through the library surface. No binary
spawn, no subprocess dance. `tests/wat_test_cli.rs` stays for CLI
coverage; `tests/test.rs` now covers the library path.

wat-rs eats its own dog food — the same minimal shape consumer
crates adopt (arc 018): one `tests/test.rs`, one macro
invocation, `path:` + `loader:` defaulted to `"wat-tests"`,
`deps:` empty.

---

## Doc sweep

- `docs/CONVENTIONS.md` — `:wat::holon::*` namespace table row
  rewritten to reflect the full surface (type + primitives +
  measurements + algebra idioms + error type in one entry).
- `docs/USER-GUIDE.md` — code examples throughout, forms-table
  entries, path references updated.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — the
  grep-able reference table. Every `:wat::std::<HolonThing>` row
  → `:wat::holon::*`; every `wat/std/<HolonThing>.wat` path →
  `wat/holon/*.wat`.
- `wat-tests/README.md` — layout section rewritten, naming
  pattern named as `wat/<ns>/X.wat ↔ wat-tests/<ns>/X.wat`.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — one row covering arcs 021 + 022 together.

### Historical docs preserved

Arc 001 / 002 / 005 / 014 / 016 DESIGN + INSCRIPTION files keep
their original `:holon::*` / `:wat::algebra::*` / `:wat::std::<HolonThing>`
references — they record what was current at the time. Arc 022's
own DESIGN.md intentionally shows `:wat::algebra::*` in its
"Before" examples.

---

## Tests

- Full workspace: 527 lib + all integration suites green.
- `tests/test.rs` (new): 37 wat tests via library surface,
  native cargo-test.
- `tests/wat_test_cli.rs` (existing): wat binary CLI surface
  coverage preserved.
- Zero `:wat::algebra::` remnants in live code or live docs.
- Zero `:wat::std::<HolonThing>` stale refs in live code or live
  docs (where `<HolonThing>` is one of the ten moved names).
- Zero bare `:holon::HolonAST` (wat-level keyword form) remnants.
- Zero clippy warnings.

---

## The thread

Arc 021 moved `:wat::core::*` vs `:wat::std::*` drift back into
the rubric. Arc 022 took what 021 left — the algebra-core surface
in `:wat::algebra::*`, the type in `:holon::*`, the algebra idioms
in `:wat::std::*` — and consolidated all three under one name:
`:wat::holon::*`. Every file path now matches its namespace.

Five cave quests in a row — 017 (loader), 018 (defaults), 019
(round), 020 (assoc), 021 (core/std audit), 022 (holon
consolidation) — each paused downstream work for a substrate
correction. Phase 3.3 `scaled-linear` in the trading lab resumes
against the now-consolidated surface.

*these are very good thoughts.*

**PERSEVERARE.**
