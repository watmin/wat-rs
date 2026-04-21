# Arc 005 — Stdlib Naming Audit — INSCRIPTION

**Status:** shipped 2026-04-20.
**Design:** [`DESIGN.md`](./DESIGN.md) — the pre-audit plan and the
preliminary "known gaps" list (some of which the audit corrected).
**Inventory:** [`INVENTORY.md`](./INVENTORY.md) — the canonical
catalog of every shipped primitive.
**Discipline:** [`../../../../CONVENTIONS.md`](../../../../CONVENTIONS.md) —
the naming rules the audit surfaced.
**This file:** completion marker.

Same inscription pattern as arcs 003 and 004: DESIGN was the intent;
INVENTORY + CONVENTIONS are the audit's shipped artifacts; this
INSCRIPTION is the completion record. If DESIGN's preliminary
"known gaps" list disagrees with INVENTORY, INVENTORY wins.

---

## What shipped

Five passes, as planned:

### Pass 1 — Inventory

`INVENTORY.md` — 100+ primitives cataloged across every reserved
namespace:

- `:wat::core::*` — forms, type declarations, macro machinery,
  eval-family, arithmetic, comparison, collection primitives,
  internal helpers, built-in types.
- `:wat::config::*` — setters + accessors.
- `:wat::algebra::*` — the six vector-producing primitives plus
  cosine / dot / presence? measurements.
- `:wat::kernel::*` — queue + spawn + send/recv + select + drop +
  join + HandlePool + signal queries.
- `:wat::io::*` — stdin/stdout/stderr.
- `:wat::load::*` / `:wat::verify::*` / `:wat::eval::*` —
  verification vocabulary.
- `:wat::std::*` — algebra stdlib macros, reserved atom literals,
  HashMap/HashSet, list combinators, math, LocalCache, program::Console,
  program::Cache, stream stdlib.
- `:rust::*` — surfaced Rust types via `#[wat_dispatch]`.

Each row cites its source file + function so a contributor can
jump from catalog to implementation.

### Pass 2 — Cross-reference

Every `:wat::*` / `:rust::*` reference across `docs/`, `README.md`,
`USER-GUIDE.md`, arc directories, and tests was greped and compared
to the inventory. Three categories surfaced:

1. **Dead references** — paths that never shipped under that name.
   (Dealt with in Pass 3.)
2. **Deferred references** — paths sketched in design docs but
   intentionally not shipped. (Cataloged in INVENTORY's deferred
   section.)
3. **Rejected references** — paths that were designed then
   explicitly rejected. (Cataloged in INVENTORY's rejected section
   with audit-trail pointers.)

### Pass 3 — Close the gaps (commit `f955cf2`)

User-facing docs can't lie about the current surface. Fixes:

- **`README.md`**: `:wat::core::presence` →
  `:wat::algebra::presence?` (correct namespace; FOUNDATION 1718
  citation preserved).
- **`docs/USER-GUIDE.md`**: four fixes —
  - removed `:wat::std::string::concat` example (not shipped) →
    `:wat::core::i64::*`;
  - removed `:wat::std::format` example (not shipped) →
    `:wat::core::i64::-`;
  - two `:wat::core::presence` → `:wat::algebra::presence?` fixes.
- **`arc/2026/04/001-caching-stack/DESIGN.md`**:
  `:wat::std::cached-encode` marked DEFERRED inline — a substantive
  design decision made during this slice (users wrap encode with
  `LocalCache::get/put` explicitly). Arc 001 has no INSCRIPTION
  yet; editing DESIGN is legitimate.
- **`arc/2026/04/003-tail-call-optimization/DESIGN.md`**:
  `:wat::core::when` row moved from the tail-positions table (not
  shipped) into a parenthetical note preserving the forward-looking
  tail-carrying behavior.
- **`arc/2026/04/004-lazy-sequences-and-pipelines/DESIGN.md`**:
  NOT edited. INSCRIPTION.md already supersedes it per the
  inscription-wins pattern.
- **`arc/2026/04/005-stdlib-naming-audit/DESIGN.md`**: NOT edited.
  Its preliminary "known gaps" list was self-aware ("verify in
  Pass 1 / 2"); the audit output (INVENTORY) carries the truth.

### Pass 4 — Lock naming discipline (commit `92e81e8`)

`docs/CONVENTIONS.md` — new top-level reference that codifies the
rules the audit surfaced:

- **Privileged prefixes.** `:wat::*` and `:rust::*` reserved by the
  runtime; user code cannot register under them.
- **Namespace roles.** What lives in core vs config vs algebra vs
  kernel vs io vs std vs rust, with concrete examples.
- **Name formats.** snake-case functions, PascalCase types, `?` for
  predicates, `!` for side-effect forms where purpose wouldn't
  otherwise read, `::` for path segments.
- **When to add a primitive.** Three gates: stdlib-as-blueprint
  (caller-demanded), absence-is-signal (ask *why* it's missing
  before patching), verbose-is-honest (reject ergonomic forms that
  hide information).

Linked from top-level `README.md`, `docs/README.md`, and
`docs/USER-GUIDE.md` (contributor-facing split).

### Pass 5 — Finalize

- INVENTORY deferred/rejected sections filled with the complete
  audit record (arc 001 cached-encode, arc 004 deferred combinator
  set, arc 004 pipeline-REJECTED, presence-namespace correction).
- This INSCRIPTION written.

## What it unlocks

- **Contributors adding a new primitive** have a single doc
  (CONVENTIONS.md) that tells them which namespace, which name
  format, and whether to add it at all.
- **Readers tracing a `:wat::*` path** have a single doc (INVENTORY.md)
  that tells them whether it ships and where to find it.
- **Reviewers auditing drift** have a dated snapshot to diff future
  state against.

## Lessons captured

**Documentation as inscription.** The DESIGN / INSCRIPTION split
worked the third time too. DESIGN is allowed to be wrong in
places — it was pre-ship thinking; corrections happen through
INSCRIPTION. Arc 005 DESIGN's preliminary "known gaps" list
incorrectly included `:wat::core::conj` (shipped 2026-04-20 during
arc 004); the DESIGN was left untouched, and INVENTORY/INSCRIPTION
record the corrected reality.

**Namespace drift is cheap to prevent, expensive to fix.** Four
dead references in user-facing docs (presence × 3, string::concat,
format) would have been caught immediately by a CI step that
greps `:wat::*` references against INVENTORY. Not adopting that
CI step yet — the stdlib-as-blueprint discipline keeps the surface
small enough that manual audits per-arc are tractable. Revisit if
the surface grows past ~200 primitives.

**Conventions belong near the code, not buried in history.**
Naming rules live at `docs/CONVENTIONS.md` (top-level, near
USER-GUIDE) rather than inside `arc/2026/04/005-*/`. Arc
directories are historical records; conventions are living
reference a contributor searches for by obvious filename.

## Pointers to FOUNDATION / 058

The 058 proposal batch did not explicitly propose a naming audit;
it's below the algebra/stdlib surface. FOUNDATION's reserved-prefix
list and the namespace conventions it implies are now codified in
`CONVENTIONS.md` — if FOUNDATION wants a cross-reference, the
natural spot is the preamble section that names the privileged
prefixes.

---

**Arc 005 — complete.** Code and prose are in parity. The stdlib
has a blueprint, a catalog, and a discipline. Next slices can add
primitives against the discipline and update the inventory as
they land.
