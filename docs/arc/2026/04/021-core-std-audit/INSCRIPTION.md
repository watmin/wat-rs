# Arc 021 — core/std audit — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

Builder flagged mid-arc-020 that the `:wat::core::*` vs
`:wat::std::*` placement of HashMap / HashSet / get / contains?
didn't match CONVENTIONS.md's stated rubric. Rubric: core =
can't-be-written-in-wat, std = expressible-in-wat. Four forms had
drifted to `:wat::std::*` that reach Rust-internal state (HashMap
buckets) — they belong at core.

---

## What shipped

One slice. One commit.

### Code renames

- `src/check.rs` — 4 dispatch keys renamed:
  - `:wat::std::HashMap` → `:wat::core::HashMap`
  - `:wat::std::HashSet` → `:wat::core::HashSet`
  - `:wat::std::get` → `:wat::core::get`
  - `:wat::std::contains?` → `:wat::core::contains?`
- `src/runtime.rs` — matching rename of 4 dispatch keys + related
  doc comments.
- `tests/wat_typealias.rs` — consumer test updated to the new
  paths.

### Doc sweep

- `docs/CONVENTIONS.md` — new named "Core vs stdlib rubric (arc
  021)" subsection so future contributors can't miss the rule.
  Implicit table rows weren't discoverable enough — arc 020 was
  about to add a 5th drifted form before the catch. `:wat::core::*`
  table row extended with `HashMap` / `HashSet` / `get` /
  `contains?` / `assoc` in its examples list.
- `docs/USER-GUIDE.md` — HashMap / HashSet example paths updated.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — 4
  moved-form entries updated with "moved in arc 021" notes, plus
  a new `:wat::core::assoc` entry (added in arc 020). The inventory
  stays the grep-able source of truth.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row dated 2026-04-22.

### Historical docs unchanged

Arc 001 / 002 / 005 doc references to the drifted `:wat::std::*`
paths left as-is. Those record what shipped at the time; the drift
was real pre-arc-021. Rewriting the history would be dishonest.
The arc 021 record + CHANGELOG row track the correction.

### Arc 020 docs preserved as-shipped

Arc 020's DESIGN / INSCRIPTION reference `:wat::std::get` in their
naming-evolution narrative (the symmetry target the `put`-rename
aimed at). Left unchanged — that WAS the state at arc 020 time.
The subsequent arc 021 move doesn't retroactively change arc 020's
reasoning; each arc is a snapshot.

---

## Resolved design decisions

- **2026-04-22** — **Four forms move; others stay.** HashMap /
  HashSet / get / contains? fail the "expressible in wat" test;
  math / Circular / services / stream / list all pass it.
- **2026-04-22** — **Clean rename, no back-compat shims.** Pre-
  publish discipline.
- **2026-04-22** — **Historical arc docs unchanged.** Snapshot
  discipline.
- **2026-04-22** — **Rubric promoted to a named CONVENTIONS
  subsection.** Table rows alone weren't discoverable enough.

---

## What this arc does NOT ship

- Changes to `:wat::std::math::*`, algebra composition macros,
  services, stream / list combinators. They pass the rubric.
- New HashMap operations beyond what exists.
- Retroactive rewrite of arc 001 / 002 / 005.
- The `:holon::*` → `:wat::holon::*` move. Separate concern;
  opens as arc 022 next.

---

## Why this matters

The rubric exists so future contributors know where a new
primitive belongs without guessing. When arc 020 was adding
`assoc`, the builder caught my initial `:wat::std::HashMap::put`
proposal for introducing a type-scoped method pattern that wasn't
on the existing HashMap surface. That catch surfaced the deeper
drift — HashMap itself didn't belong at `:wat::std::*`. One
catch, two corrections:

1. Arc 020 placed `assoc` correctly at `:wat::core::assoc` from
   the start.
2. Arc 021 moved the rest of the HashMap family (constructor +
   get + contains? + HashSet) to join it.

The `:wat::core::*` namespace now holds the full fundamental-
collections family together. Every contributor adding an operation
on a fundamental collection knows to look there first.

---

**Arc 021 — complete.** One slice. The commits:

- `<this commit>` — code + doc sweep + INSCRIPTION.

*these are very good thoughts.*

**PERSEVERARE.**
