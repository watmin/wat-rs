# Arc 022 — holon namespace consolidation — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

Two parallel namespaces held holon-related surface with no
principled distinction:

- `:holon::HolonAST` — the algebra's universal value type.
- `:wat::algebra::*` — the primitives that produce it plus the
  scalar measurements that read it.

Builder framing, two moments:

> i think we need to move the holon things into :wat::holon::*
>
> i think algebra -> holon -- all the holon things should be in holon

One namespace for one concept.

---

## What shipped

One slice. Two renames, applied mechanically across every file
type.

### Rename 1 — the type

- `:holon::HolonAST` → `:wat::holon::HolonAST` at every wat-level
  use (keyword contexts, `:Vec<...>` / `:Result<...>` / `:AST<...>`
  type parameters, return types, struct fields).

The Rust crate name stays `holon` (the `holon-rs` crate). The
Rust identifier `holon::HolonAST` is preserved at its `use`
sites in `src/lower.rs` and elsewhere — only the wat-level path
renames.

### Rename 2 — the namespace

Every member of the algebra surface moves from `:wat::algebra::*`
to `:wat::holon::*`:

**AST-producing primitives** (6):
- `Atom`, `Bind`, `Bundle`, `Blend`, `Permute`, `Thermometer`

**Measurements** (3):
- `cosine`, `dot`, `presence?`

**Error type** (1):
- `CapacityExceeded`

The `:wat::algebra::*` namespace retires entirely. No
compatibility shim — pre-publish clean break.

### Reserved-prefix gate

- `src/resolve.rs::RESERVED_PREFIXES` — `:wat::algebra::` entry
  retired, `:wat::holon::` entry present (already added during
  earlier sweeps).

### Code sweep

All dispatch keys, pattern matches, and docstrings updated in:

- `src/check.rs`, `src/runtime.rs`, `src/resolve.rs`, `src/types.rs`,
  `src/parser.rs`, `src/macros.rs`, `src/lower.rs`, `src/freeze.rs`,
  `src/lexer.rs`, `src/lib.rs`, `src/ast.rs`
- `wat/std/*.wat` — Circular, Log, Sequential, Subtract, Amplify,
  Ngram, Bigram, Trigram, Reject, Project
- `wat-tests/std/*.wat` — Circular, Reject, Sequential, Subtract,
  Trigram
- `tests/*.rs` — every integration test carrying holon-related type
  annotations (`wat_bundle_capacity`, `wat_eval_result`,
  `wat_hermetic_round_trip`, `wat_run_sandboxed`, `wat_structs`,
  `wat_variadic_defmacro`, `wat_cli`, `mvp_end_to_end`)

### Doc sweep

- `docs/CONVENTIONS.md` — namespace table row rewritten to reflect
  the unified holon surface (type + primitives + measurements +
  error type in one entry).
- `docs/USER-GUIDE.md` — code examples throughout, forms-table
  entries, Bundle capacity section.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — the
  grep-able source-of-truth reference table updated to
  `:wat::holon::*` for every algebra entry.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — one row covering arcs 021 + 022 together (same
  builder-initiated sweep).

### Historical docs preserved

Arc 001 / 002 / 005 / 014 / 016 DESIGN + INSCRIPTION files keep
their original `:holon::*` / `:wat::algebra::*` references — they
record what was current at the time. The arc-022 record is the
correction.

Arc 022's own DESIGN.md intentionally shows `:wat::algebra::*` in
its "Before" examples.

---

## Tests

- Full workspace: 527 lib + all integration suites green.
- Release binary: `wat test wat-tests/` — 37 wat-level tests
  green.
- Zero `:wat::algebra::` remnants in live code or live docs.
- Zero bare `:holon::HolonAST` (keyword form) remnants.
- Zero clippy warnings.

---

## Scope NOT shipped (potential next slice)

The builder's final pre-compaction message expanded arc 022's
scope:

> so i think we need wat/std/&lt;holon-thing&gt;.wat -> wat/holon/*.wat

Moving the holon-related `.wat` stdlib files (Circular, Log,
Sequential, Subtract, Amplify, Ngram, Bigram, Trigram, Reject,
Project) from `wat/std/` to `wat/holon/` is deferred to a later
slice — file reorganization distinct from the namespace rename
shipped here. When it lands, it will likely carry a namespace
move as well (`:wat::std::Circular` → `:wat::holon::Circular` …)
per the file-path-matches-namespace convention, pending builder
confirmation.

Scope kept tight: this inscription covers the namespace
consolidation ONLY. File reorganization is its own decision.

---

## The thread

Arc 021 moved `:wat::core::*` vs `:wat::std::*` drift back into
the rubric. Arc 022 unifies what arc 021 left in two places: the
type and the algebra surface. One namespace for one concept, now
that the rubric has somewhere to place it.

Four cave quests in a row — 019 (round), 020 (assoc), 021
(core/std audit), 022 (holon namespace) — each paused the lab's
Phase 3 descent for a substrate correction. Phase 3.3
`scaled-linear` resumes against the now-consolidated surface.

*these are very good thoughts.*

**PERSEVERARE.**
