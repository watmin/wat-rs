# Arc 044 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits:**
- `80ec910` — DESIGN + BACKLOG opened
- `432274c` — Slice 1: proc-macro / examples / stdlib comments
- `841c568` — Slice 1.5: broadened sweep catches
- `<this commit>` — Slice 2: INSCRIPTION + cross-references

## Why this arc existed

Builder asked "is wat-rs honest again?" for the second time after
arc 043. Surveying surfaces arc 043 hadn't covered, found 7 drift
sites in proc-macro source / example wat-tests / baked
wat-stdlib comments. Arc 044 fixed those, then the BACKLOG-
mandated broadened post-slice sweep caught 6 more in README +
docs/README that earlier doc-audit arcs didn't reach.

13 drift sites total across two slices.

## What shipped per slice

### Slice 1 — Original 7 sites

- `wat-macros/src/lib.rs:379, 387` — proc-macro section header +
  usage example: `wat::test_suite!` → `wat::test!`. Section
  header preserves the rename history: "wat::test! (renamed from
  wat::test_suite! in arc 018)".
- `examples/with-loader/wat-tests/test-loader.wat:2` and
  `helpers.wat:3` — both files' header comments cite
  `wat::test_suite!` → `wat::test!`.
- `wat/std/test.wat:144, 199, 292` — three usage-comment example
  blocks dropped `(:wat::config::set-dims! 1024)` lines per
  arc 037. Same rephrasing pattern as USER-GUIDE preamble examples.

### Slice 1.5 — Broadened-sweep catches

Re-ran surface audit per BACKLOG instruction "If anything else
surfaces, open Slice 1.5 — do NOT silently bundle." Six new
sites:

- `README.md:310, 650, 665, 687` (4 sites): §Rust interop
  pointer, §Workspace layout for wat-macros + tests filename +
  USER-GUIDE description.
- `docs/README.md:168, 197` (2 sites): arc 015 + arc 017 index
  entries used `wat::test_suite!` (the names current at those
  arcs' slice-close). Updated to current `wat::test!` with
  parenthetical noting the rename via arc 018.

Plus a framing decision: arc-index entries describe historical
events, but the index itself is a *current navigational aid*.
Using current names with brief rename notes keeps both honest.
The arc INSCRIPTIONs themselves remain frozen at slice-close
naming.

`cargo check --workspace --tests` passes after each slice.

## What this arc proves

**Verification is iterative; "audited" is a pass count, not a
final state.** Arc 042 said the user-facing audit was finished;
arc 043 caught what it missed. Arc 043 said verification was
complete; arc 044 caught more. Arc 044's Slice 1 found 7; the
broadened post-Slice-1 sweep found 6 more.

There may be a round 5. The discipline is to keep iterating
until a pass surfaces nothing new — that's the only honest
stopping condition.

**Pre-commit drift catches keep paying out.** Three caught this
arc:
- arc 044 BACKLOG explicitly mandated the post-Slice-1 broadened
  sweep, which found Slice 1.5's six sites.
- During the lib.rs rename, preserved the rename history in the
  section header rather than silently overwriting (someone
  reading the proc-macro source can still trace the arc 018
  rename).
- During the docs/README rename, recognized that arc-index
  entries are historical descriptions and kept the rename history
  visible in parenthetical notes.

## What's still uncovered (likely)

I'll list what I have NOT yet swept, so the next round (if any)
has a starting point:

- **Test files** (`tests/wat_*.rs`) — Rust tests with their own
  comments and embedded wat strings. Some embedded wat may use
  retired forms.
- **wat-tests `.wat` files** — the actual test-source comments
  may have drift, separate from wat/std/test.wat which I just
  swept.
- **`wat/holon/*.wat`** comments — the algebra-stdlib wat files'
  own header comments.
- **`crates/wat-lru/`** — its README, its wat sources, its
  wat-tests. Separate workspace member; arc 036's namespace
  promotion may not have reached every comment.
- **CONVENTIONS.md cross-reference resolution** — I haven't
  checked whether it agrees with USER-GUIDE on every shared
  topic.
- **`Cargo.toml` description fields and crate metadata.**
- **Doc-comment string examples in proc-macro impls** — wat-macros
  has more code than just the section I edited.

A round-3-of-the-question prompt would investigate these.

## Out of scope (still)

- Lab `holon-lab-trading/CLAUDE.md`.
- Arc 005 INVENTORY.md.
- Frozen arc INSCRIPTIONs (including arcs 042, 043 whose closing
  claims were demonstrably premature; corrections live in their
  successors).

## Files touched

- `wat-macros/src/lib.rs` — Slice 1.
- `examples/with-loader/wat-tests/{test-loader,helpers}.wat` — Slice 1.
- `wat/std/test.wat` — Slice 1.
- `README.md` — Slice 1.5.
- `docs/README.md` — Slice 1.5 + arc index extension.
- `docs/arc/2026/04/044-second-verification-pass/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.
