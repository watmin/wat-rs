# Arc 022 — Holon namespace consolidation

**Status:** opened 2026-04-22.
**Motivation:** "all the holon things should be in holon" (builder,
mid-sweep). Two parallel namespaces held holon-related surface
with no principled distinction:

- `:holon::HolonAST` — the algebra's universal value type.
- `:wat::algebra::*` — the algebra's primitives + measurements
  that PRODUCE HolonAST values.

Both are about holon algebra. They belong in one namespace under
wat's owned surface.

Builder framing (two moments):
> i think we need to move the holon things into :wat::holon::*
>
> i think algebra -> holon -- all the holon things should be in holon

Arc 022 consolidates: everything algebra-related moves to
`:wat::holon::*`.

---

## UX target

```scheme
;; Before (two parallel namespaces)
(:wat::core::define (build -> :holon::HolonAST)
  (:wat::algebra::Bundle
    (:wat::core::vec :holon::HolonAST
      (:wat::algebra::Atom :a)
      (:wat::algebra::Atom :b))))

(:wat::algebra::presence? h1 h2)

;; After (one namespace)
(:wat::core::define (build -> :wat::holon::HolonAST)
  (:wat::holon::Bundle
    (:wat::core::vec :wat::holon::HolonAST
      (:wat::holon::Atom :a)
      (:wat::holon::Atom :b))))

(:wat::holon::presence? h1 h2)
```

---

## Renames

Full inventory of the `:wat::holon::*` namespace post-arc-022:

**Type:**
- `:holon::HolonAST` → `:wat::holon::HolonAST`

**Algebra-core primitives** (AST constructors producing HolonAST):
- `:wat::algebra::Atom` → `:wat::holon::Atom`
- `:wat::algebra::Bundle` → `:wat::holon::Bundle`
- `:wat::algebra::Bind` → `:wat::holon::Bind`
- `:wat::algebra::Blend` → `:wat::holon::Blend`
- `:wat::algebra::Permute` → `:wat::holon::Permute`
- `:wat::algebra::Thermometer` → `:wat::holon::Thermometer`

**Measurements** (HolonAST → scalar/bool):
- `:wat::algebra::cosine` → `:wat::holon::cosine`
- `:wat::algebra::dot` → `:wat::holon::dot`
- `:wat::algebra::presence?` → `:wat::holon::presence?`

**Error type from arc 019:**
- `:wat::algebra::CapacityExceeded` → `:wat::holon::CapacityExceeded`

The `:wat::algebra::*` namespace goes away entirely.

---

## Non-goals

- **Changes to `:wat::std::*` algebra idioms** (Circular, Log,
  Sequential, Subtract, Amplify, Ngram, Reject, Project, ...).
  These are wat stdlib macros/defines built OVER the holon
  primitives. They stay at `:wat::std::*` per arc 021's rubric —
  they're expressible in wat.
- **Changes to holon-rs the Rust crate.** The Rust identifier
  `holon::HolonAST` stays where it lives in the crate. Only the
  WAT-LEVEL path renames.
- **Any algebra behavior changes.** Mechanical rename; semantics
  identical.
- **Extension of `:wat::holon::*` with new operations.** Only
  existing surface moves. Future operations are separate arcs.

---

## What this arc ships

One slice. Mechanical bulk rename.

### Code sweep

- `src/check.rs`, `src/runtime.rs`, `src/resolve.rs`, `src/types.rs`,
  `src/parser.rs`, `src/macros.rs`, `src/lower.rs` — every
  `:wat::algebra::` dispatch key + pattern match + docstring.
- `src/runtime.rs` — plus the `:holon::HolonAST` → `:wat::holon::HolonAST`
  rename for the type.
- `wat/std/*.wat` — stdlib macros/defines that reference the
  algebra primitives as `:AST<...>` params or in bodies.
- `wat-tests/std/*.wat` — tests that construct + measure via
  algebra primitives.
- `tests/*.rs` integration tests.

### Reserved-prefix gate

- `RESERVED_PREFIXES` in `src/resolve.rs` — replace `:wat::algebra::`
  with `:wat::holon::`.

### Doc sweep

- `docs/CONVENTIONS.md` — namespace table row: rename `:wat::algebra::*`
  entry to `:wat::holon::*` with updated description noting this
  is the full holon algebra surface (type + primitives +
  measurements).
- `docs/USER-GUIDE.md` — code examples throughout.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  entries updated + "moved in arc 022" notes.
- `docs/arc/2026/04/022-holon-namespace-move/INSCRIPTION.md`.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row covering arc 021 + 022 together (they're both namespace
  audits from the same builder-initiated sweep).

### Historical docs left alone

Arc 001 / 002 / 005 / earlier references to `:holon::*` or
`:wat::algebra::*` stay as-is. They record what shipped at the
time; the correction is tracked in the arc 022 record.

---

## Resolved design decisions

- **2026-04-22** — **`:wat::holon::*` unifies everything holon-
  related.** Type, primitives, measurements, error type. One
  namespace for one concept.
- **2026-04-22** — **`:wat::algebra::*` retires.** Clean break;
  no compatibility shim. Pre-publish.
- **2026-04-22** — **`:wat::std::*` algebra idioms stay.** They
  fit the "expressible in wat" rubric — they ARE wat macros. Not
  holon-primitive territory.
- **2026-04-22** — **Mechanical rename only.** No semantic
  changes; every primitive behaves identically. Bulk sed across
  every file type (Rust, wat, docs).

---

## What this arc does NOT ship

- Changes to any `:wat::std::*` macro or composition.
- Changes to holon-rs the Rust crate.
- New operations under `:wat::holon::*`.
- Retroactive doc rewrites of pre-021 arcs.
- Behavior changes of any kind.
