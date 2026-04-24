# Arc 040 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits (in order):**
- `73fede4` — DESIGN + BACKLOG opened
- `d087eaf` — Slice 1: §Sandbox Config inheritance set-dims! drop
- `f44d910` — Slice 2: wat::test_suite! → wat::test! (arc 018)
- `01c7944` — Slice 3: Namespace table + load-file! syntax fix
- `<this commit>` — Slice 4: INSCRIPTION + cross-references

## What this arc fixed

`wat-rs/docs/CONVENTIONS.md` had drifted across several arcs since
its last substantive sync. arc 036's `47ea30a` had reached the
wat-lru namespace mention, but earlier surface changes were
unreflected. Drift-only — file readable; just stale.

## What shipped per slice

### Slice 1 — §Sandbox Config inheritance

- Prose: "capacity-mode + dims" → "capacity-mode (and any optional
  set-dim-router! / sigma-fn overrides — arcs 024 + 037)" — names
  what config setters actually exist post-arc-037.
- Example dropped `(:wat::config::set-dims! 1024)`. Comment
  references in deftest invocations updated from "inherits :error
  + 1024" to "inherits :error + the active dim-router."

### Slice 2 — `wat::test_suite!` → `wat::test!` (arc 018)

Five edit points across CONVENTIONS:
- §External wat crates contract paragraph.
- §Crate folder layouts publishable template (`tests/wat_suite.rs`
  → `tests/test.rs`; example macro call updated).
- §Crate folder layouts consumer template.
- §Three varieties of wat crate compose-them-identically line.
- §Viewing per-wat-test output: macro name updated; internal
  `fn wat_suite()` KEPT (verified via `wat-macros/src/lib.rs:550`
  that the macro still emits that function name).
- §Binary vs library: `wat::test_suite!` under a test dir →
  `wat::test!`.

Pre-edit verification of the function name kept the rename surgical
(only the macro name and the conventional filename changed; the
emitted function is still `wat_suite`).

### Slice 3 — Namespace table + load-file! syntax

§Namespaces table:
- `:wat::config::*` description rewritten to enumerate actual
  setters (capacity-mode, dim-router, presence-sigma,
  coincident-sigma, global-seed) and note the compat shim
  accessors. Was: "noise floor, dimensions" (pre-arc-037).
- `:wat::holon::*` extended with `eval-coincident?` family
  (arc 026) and `Holons`/`BundleResult` typealiases.
- `:wat::std::*` description corrected: LocalCache moved to
  `:wat::lru::*` (arcs 013 + 036); `program::*` renamed to
  `service::*` (BOOK chapter 21 era).
- New `:wat::lru::*` row added.

§Binary vs library:
- `(:wat::load-file! (path argument) "...")` →
  `(:wat::load-file! "...")` — the "(path argument)" was a
  leftover iface-keyword reference from before arc 028 hoisted
  the form; corrected to the actual current shape.

### Slice 4 — INSCRIPTION + cross-references

This document. `docs/README.md` arc index extended with arc 040
entry. 058 FOUNDATION-CHANGELOG row in lab repo.

## What this arc proved

Smaller drift surface than 038/039 — the rules document tracks
substrate decisions more closely than the user guide tracks
shipped surface, so each arc's CONVENTIONS update typically lands
alongside its substrate commit. Drift here is real but slim.

The per-doc audit shape continues to scale: arc 038 was 9 slices
(recovery + sync); arc 039 was 8 slices (drift only on a busy
file); arc 040 was 4 slices (drift only on a slimmer file). Each
arc owns one file; each slice covers one section group; the
discipline holds across scope sizes.

## Out of scope (future arcs)

- `wat-rs/wat-tests/README.md` (arc 041 candidate) — last touched
  in `5b5fad8` only.
- `wat-rs/docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md`
  (arc 042 candidate) — touched by arc 035; arcs 036/037
  unreflected.
- `wat-rs/docs/ZERO-MUTEX.md` (arc 043 candidate) — pre-5b5fad8;
  arcs 028-037 all unreflected.
- `holon-lab-trading/CLAUDE.md` (lab-side arc) — last touched
  2026-04-16; very stale.

## Files touched

- `docs/CONVENTIONS.md` — three slices' worth of targeted edits.
- `docs/arc/2026/04/040-conventions-drift-sync/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `docs/README.md` — arc index extended.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.
