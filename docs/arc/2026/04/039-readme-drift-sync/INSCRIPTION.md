# Arc 039 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits (in order):**
- `e64d336` — DESIGN + BACKLOG opened
- `83943d4` — Slice 1: §What wat is namespace migration
- `a60b2b2` — Slice 2: §Status forward sync
- `cc48c2b` — Slice 3: §Module tour runtime entry
- `c294985` — Slice 4: §wat binary + §Self-hosted testing
- `5fb037d` — Slice 5: §Stdlib + §Capacity guard
- `b40f05d` — Slice 6: §Namespace discipline + §Workspace layout
- `69280c5` — Slice 7: §What's next signed-form references
- `<this commit>` — Slice 8: INSCRIPTION + cross-references

## What this arc fixed

The wat-rs `README.md` had drifted across 15 arcs since its last
substantive sync at commit `467a3d4` (arc 022 era, 2026-04-22). 
arc 036's `47ea30a` had touched only the wat-lru namespace; arcs
022, 023, 024, 025, 026, 027, 028, 029, 030, 031, 032, 033, 034,
035, 037 were unreflected or partially-reflected.

**Drift, not corruption.** Builder confirmed via GitHub browse the
file was readable; just stale. Standard targeted-edit forward
sync per arc — same discipline as arc 038's recovery, smaller
scope.

## What shipped per slice

### Slice 1 — §What wat is

Two `:wat::algebra::*` references in the INTERPRET path
description migrated to `:wat::holon::*` (arc 022).

### Slice 2 — §Status forward sync

Three updates to the §Status block:
- *Programs-as-holons paragraph:* `:wat::algebra::*` →
  `:wat::holon::*`, added arc 023 `coincident?` as the dual
  predicate, updated noise-floor framing from fixed "5σ" to
  function-of-d threshold (arc 024 + 037).
- *Capacity-guard paragraph:* `:wat::algebra::*` →
  `:wat::holon::*`, Bundle return type uses `BundleResult`
  typealias (arc 032), capacity modes reduced to `:error` /
  `:abort` with `:silent`/`:warn` retired (arc 037).
- *New "Subsequent arcs (023-037)" paragraph* naming the major
  themes shipped since arc 022's namespace consolidation: algebra
  surface (coincident?, eval-coincident?, ReciprocalLog,
  typealiases, sigma knobs); container surface unification
  (025, 035); macros (029, 030); load/eval root hoist (028);
  test ergonomics (027, 031); multi-tier dim-router (037);
  wat-lru namespace promotion (036).
- *Test count:* dropped specific 731 / 31 numbers (pre-arc-029,
  unverifiable without running cargo test). Replaced with
  "zero regressions across every shipped arc" + the per-arc
  test-suite list. Live counts live in each arc's INSCRIPTION.

### Slice 3 — §Module tour

`runtime` module description updated:
- `:wat::algebra::*` → `:wat::holon::*` dispatch list (arc 022).
- `:wat::algebra::Atom accepts Value::wat__WatAST` → `:wat::holon::Atom`.
- Added "the hoisted load/eval families at `:wat::*` root
  (arc 028)" to the dispatch list.

§Rust interop had no retired-form references.

### Slice 4 — §wat binary + §Self-hosted testing

Two examples updated for arc 037's `set-dims!` retirement:
- §wat binary `echo.wat`: drop `set-dims!` line; capacity-mode
  kept.
- §Self-hosted testing deftest example: drop `set-dims!` from
  preamble.

Prose around the deftest example updated to describe the inherited
config in current terms — capacity-mode (and any
`set-dim-router!` or sigma-fn overrides) instead of
"capacity-mode + dims" since the old `dims` setter is retired.

### Slice 5 — §Stdlib + §Capacity guard

§Stdlib (the heaviest namespace cluster):
- *Algebra conveniences:* `:wat::std::*` → `:wat::holon::*`
  (arc 022). Added arc 034 ReciprocalLog. Added measurements
  subsection (cosine, dot, presence?, coincident?,
  eval-coincident? family). Added typealiases line for Holons
  (arc 033) + BundleResult (arc 032).
- *Test harness:* added `run-hermetic-ast` (arc 012 — fork-based)
  and `make-deftest` (arc 029 — factory). Updated `deftest` entry
  to note the dropped dims/mode args (arc 031).

§Capacity guard:
- `:wat::algebra::*` → `:wat::holon::*` across the section
  (arc 022).
- Bundle return type uses `:wat::holon::BundleResult` typealias
  (arc 032).
- Mode table reduced from four to two: `:silent` and `:warn`
  retired (arc 037). Default is now `:error`. Brief prose
  explains why the two were retired.
- Capacity bound formula reframed in terms of `d` (the dim picked
  by the active DimRouter) instead of a single global `dims`.

### Slice 6 — §Namespace discipline + §Workspace layout

§Namespace discipline:
- Stragglers from §Capacity guard prose:
  `:wat::std::Ngram/Bigram/Trigram` → `:wat::holon::*`.
- Added `:wat::lru::*` to the sub-namespace enumeration
  (arc 036).
- Stretched the `:wat::std::*` description to enumerate what
  actually lives there post-arc-022 (streams, hermetic, test,
  service/Console).

§Workspace layout:
- New `wat/holon/` directory entry listing the algebra wat
  sources (arc 022 + ReciprocalLog from arc 034).
- `wat/std/` trimmed to the post-arc-022 contents (`stream.wat`,
  `hermetic.wat`, `test.wat`, `service/Console.wat`).
- Arc directory listing extended from 018 → 039 (this arc). 21
  arcs added to the index.

### Slice 7 — §What's next signed-form references

§What's next "Signature verification" paragraph used pre-arc-028
form names (`:wat::signed-load!`, `:wat::eval-signed!`). Both
moved to root with file/string variants in arc 028 — updated to
the file/string variants explicitly.

§See also has no retired-form references.

### Slice 8 — INSCRIPTION + cross-references

This document. `docs/README.md` arc index extended with arc 039
entry. 058 FOUNDATION-CHANGELOG row in the lab repo.

## What this arc proved

**Drift-only sync works under the same discipline as recovery.**
arc 038 was the high-stakes file-corruption recovery; arc 039 was
the lower-stakes drift-only sync of a sibling user-facing doc.
Same shape: targeted edits per section, never a sweep, commit per
slice, push per commit. The discipline scales down to "this is
just stale" without losing audit-trail value.

**The stdlib-as-blueprint discipline keeps user-facing docs
honest** when applied in real time. Each shipped arc that surfaces
user-visible change should land its README-edit alongside the
substrate commit. Arc 039 caught up to arc 037; future arcs land
their README touches as part of their own commit batches, not as
post-hoc audits.

**The "checkpoint diligence" framing extends to the audit set.**
Builder named it: arcs and proposals are trusted by construction;
docs that *describe* arcs and proposals are drift candidates. Arc
039 demonstrates the per-doc audit shape (one doc, one arc, one
INSCRIPTION). Subsequent arcs (040, 041, …) will apply the same
shape to the remaining audit set (`CONVENTIONS.md`,
`wat-tests/README.md`, `INVENTORY.md`, `ZERO-MUTEX.md`, lab
`CLAUDE.md`).

## Out of scope (future arcs)

- **`wat-rs/docs/CONVENTIONS.md`** (arc 040 candidate) — same
  partial-overwrite profile as `README.md`; touched by arc 036
  but earlier arcs unreflected.
- **`wat-rs/wat-tests/README.md`** (arc 041 candidate) — last
  touched in `5b5fad8` only; every arc since is unreflected.
- **`wat-rs/docs/arc/.../005-stdlib-naming-audit/INVENTORY.md`**
  (arc 042 candidate) — touched by arc 035; arcs 036/037
  unreflected.
- **`wat-rs/docs/ZERO-MUTEX.md`** (arc 043 candidate) — pre-5b5fad8
  baseline; arcs 028-037 all unreflected.
- **`holon-lab-trading/CLAUDE.md`** (lab-side arc) — last touched
  2026-04-16; way stale, predates arcs 023+. Different repo, will
  ship as a lab arc rather than a wat-rs arc.

## Files touched

- `README.md` — seven slices' worth of targeted edits.
- `docs/arc/2026/04/039-readme-drift-sync/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record itself.
- `docs/README.md` — arc index extended (this slice).
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row (this slice).
