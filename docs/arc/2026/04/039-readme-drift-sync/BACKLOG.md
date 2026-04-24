# Arc 039 — BACKLOG

Status markers:
- **ready** — region known, edits obvious in shape.
- **obvious in shape** — clear once a prior slice lands.
- **foggy** — needs re-read or verification first.

Slices grouped by section to minimize re-touch of the same lines.
Each slice is a small set of `Edit` calls; never a full-file rewrite.

---

## Slice 1 — Header / What wat is / Dependency stack

**Status: ready.**

Lines 1-55. Mostly fine; touches:
- Line 32-33: `:wat::algebra::Atom`, `:wat::algebra::Bind` references in
  the INTERPRET path description → migrate to `:wat::holon::*` (arc 022).

## Slice 2 — §Status forward sync

**Status: ready.**

Lines 56-126. The shipped-state block. Six rolled-up updates:

- `:wat::algebra::*` → `:wat::holon::*` everywhere it appears in
  status prose (arc 022 — 5 occurrences in this section).
- Add a brief paragraph naming arcs 023-037's themes (coincident?,
  sigma knobs, container surface unification, eval-coincident
  family, loader option, load/eval root hoist, macro
  factory/expand, sandbox config inheritance, typealiases,
  ReciprocalLog, length polymorphism, wat-lru promotion,
  multi-tier dim-router). Brief — one sentence per cluster, not
  per-arc.
- Test count line — flag for verification rather than pasting a
  guess; leave the existing count if unchanged.

## Slice 3 — Module tour + Rust interop

**Status: ready.**

Lines 127-278.

- Line 169: `runtime` description mentions
  `:wat::core::*` / `:wat::algebra::*` dispatch — `:wat::algebra::*` →
  `:wat::holon::*`.
- Line 173: `:wat::algebra::Atom accepts Value::wat__WatAST` →
  same migration.
- Rest of §Module tour and §Rust interop — spot check; likely fine
  on shape, but verify no lurking retired-form mentions.

## Slice 4 — §wat binary + §Self-hosted testing

**Status: ready.**

Lines 279-459. Test-shape and binary-contract updates:

- Line 344, 371: `(:wat::config::set-dims! 1024)` in entry-file
  examples → drop per arc 037 (config setters are optional under
  the multi-tier dim-router).
- §Writing a test — `deftest` signature: drop `1024 :error` per
  arc 031. Outer file's `set-capacity-mode!` covers the sandbox
  via inheritance.
- §Fork/sandbox tests, §Services that spawn threads — verify the
  `run-ast` / `run-hermetic-ast` examples don't carry retired
  `set-dims!` setters in inner programs.
- Possibly add a brief `make-deftest` mention (arc 029) or defer
  to §Stdlib mention.

## Slice 5 — §Stdlib + §Capacity guard

**Status: ready.**

Lines 460-542. Heaviest namespace + retired-mode work:

- Stdlib catalog: add `coincident?` (arc 023), `eval-coincident?`
  family (arc 026), `ReciprocalLog` (arc 034), maybe namespace
  Holons + BundleResult typealiases (arcs 032, 033).
- Capacity guard subsection: drop `:silent` / `:warn` rows from
  the mode table (arc 037 retired both); only `:error` and
  `:abort` remain. Update prose around capacity-mode default
  (the new default is `:error`).
- `:wat::algebra::Bundle` / `:wat::algebra::CapacityExceeded` →
  `:wat::holon::*` (arc 022).
- Bundle return type: use `:wat::holon::BundleResult` typealias
  (arc 032).

## Slice 6 — §Namespace discipline + §Workspace layout

**Status: ready.**

Lines 543-642.

- §Namespace discipline: should mention arc 028's load/eval root
  hoist (`:wat::load-file!` etc. as siblings to `:wat::core::*`).
- §Workspace layout: spot-check paths. wat-lru promoted in arc 036
  (`:user::wat::std::lru::*` → `:wat::lru::*`); README was already
  partially updated by `47ea30a` but verify no stragglers.

## Slice 7 — §What's next + §See also

**Status: ready.**

Lines 643-687.

- §What's next likely names completed arcs as future. Audit:
  arc 003 (TCO) shipped, multi-tier dim-router (arc 037) shipped.
  Mark anything completed that's still listed as pending.
- §See also: spot-check links and references. Likely fine.

## Slice 8 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md` summarizing what shipped per slice with commit
  refs.
- `docs/README.md` arc index extended with arc 039 entry.
- 058 FOUNDATION-CHANGELOG row in the lab repo.

---

## Cross-cutting

- **Verification after each slice:** `wc -l`, header grep, optional
  spot-read of touched region. Confirm no new retired-form
  introductions via grep.
- **Commit per slice.** Push per commit. Same rhythm as arc 038.
- **Test count strategy:** only update if verified from current
  `cargo test`. Otherwise leave the line as-is and flag in INSCRIPTION.

## Sub-fogs

- **Does §Status need an arc-by-arc bullet list, or one rolled-up
  paragraph?** Arc 038's INSCRIPTION model is per-slice; here the
  scale is per-arc and 15+ arcs is too many bullets. Lean: one
  paragraph naming the major themes; let interested readers click
  through to the arc INSCRIPTIONs. Confirm at slice time.
- **§Stdlib organization.** If the section currently lists primitives
  by name, reorder/add carefully — don't reshape the existing list.
- **Test count.** If outdated, propose a fresh `cargo test` run as
  part of slice 5 verification rather than guessing.
