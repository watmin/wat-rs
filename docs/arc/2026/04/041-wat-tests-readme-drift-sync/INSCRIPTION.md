# Arc 041 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits:**
- `ccc46ce` — DESIGN + BACKLOG opened
- `a9d41c6` — Slice 1: §Layout external-crate addition + §In-process user-facing layer
- `<this commit>` — Slice 2: INSCRIPTION + cross-references

## What this arc fixed

`wat-rs/wat-tests/README.md` had drifted by **omission, not
correction**. Standard audit set returned zero retired-form
occurrences (arcs 022 / 018 / 037 surface changes were already
reflected). The two gaps were:

1. **§Layout** described only wat-rs's internal `wat-tests/`
   tree. The pattern is portable — consumer crates (workspace
   members like `crates/wat-lru/` and out-of-tree consumers)
   ship their own `wat-tests/` trees and run via per-crate
   `cargo test -p <crate>` through `wat::test! {}` (arcs
   013 + 015 + 036). Naming this explicitly tells the reader
   that the discipline they're seeing is general.

2. **§In-process vs hermetic** described only the substrate
   primitives (`:wat::test::run`, `:wat::test::run-hermetic-ast`).
   Most readers reach for the user-facing macros first
   (`deftest`, `deftest-hermetic`, `make-deftest`,
   `make-deftest-hermetic` — arcs 029 + 031). Restating the
   section "macros first, primitives below" gives the reader a
   layered surface to start at.

## What this arc proved

**Drift can be additive.** Arcs 038-040 fixed retired-form
references and stale prose. Arc 041 was almost entirely
*adding* coverage of arcs that had shipped without touching this
file — arc 013/015/036 (external crates) and arc 029/031 (test
macros) had no reason to edit the wat-tests README at the time.
The missing references surfaced only when a reader (the
assistant, in this case) traced what the README was teaching vs
what the wat ecosystem had actually become.

The audit shape works: read the file, list every claim, walk
each claim against shipped state, name the gaps. The discipline
isn't "find typos"; it's "find the absences that mislead a fresh
reader."

## Out of scope (future arcs)

- `wat-rs/docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md`
  (arc 042 candidate) — densest audit of the remaining set;
  partial overwrite at arc 035.
- `wat-rs/docs/ZERO-MUTEX.md` (arc 043 candidate) — pre-5b5fad8
  baseline; arcs 028-037 unreflected.
- `holon-lab-trading/CLAUDE.md` (lab-side arc) — separate repo,
  separate domain; will ship as a lab arc rather than wat-rs.

## Files touched

- `wat-tests/README.md` — one slice, two additions.
- `docs/arc/2026/04/041-wat-tests-readme-drift-sync/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `docs/README.md` — arc index extended.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.
