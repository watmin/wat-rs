# DESIGN-STONE — the gate must tell a declared red from a rotted one

> **Origin (2026-08-31).** The builder asked one question of a file the orchestrator had just
> banked: ***"where does this file live such that it does not run?"*** Everything below followed.

## Why

`tests/lint/wat_scripts_fixes_load.rs` states the doctrine in its own header:

> *"A stale exemplar that no longer runs is a graveyard that reads like live code (it trapped a
> prior session). This gate closes that blind spot: **ALL wat must remain correct, always**."*

It walks `wat-scripts/` **only**. `docs/arc/**` is exempt by omission, and holds 8 orphaned `.wat`.

**The escape hatch from that gate is a directory with no gate**, and the reasoning that put files
there is written down and is *correct*. `probes/red-owner-signals-child.wat`, months before this
session:

> *"⛔ THIS FILE IS RED BY DESIGN, TODAY. It lives under `docs/…/probes/` (NOT `wat-scripts/`)
> precisely because `every_wat_scripts_file_loads` walks `wat-scripts` only — a deliberately-failing
> probe parked there would break that gate."*

Sound reasoning; graveyard consequence. Deliberately-red and genuinely-rotted files now sit in one
directory and **nothing can tell them apart.** The workaround for the gate created the situation
that needs another gate.

## The measurement — all 8 driven on the current runtime, HEAD `819c79b9a`

| file | verdict |
|---|---|
| `probes/enum-holds-record.wat` | runs, prints — alive |
| `probes/red-send-cause-is-not-matchable.wat` | runs, prints — alive |
| `probes/red-owner-signals-child.wat` | fails — **red by design**, declared in its header |
| `probes/surface-field-dispatch.wat` | fails — **ROTTED, silently, ~8 weeks** |
| `harness-experiri/experiri-acc-wrapped.wat` | fires — alive |
| `harness-experiri/experiri-when-match.wat` | loads — alive |
| `harness-experiri/experiri-acc-head.wat` | refuses — red by design (the A3 repro) |
| `harness-experiri/experiri-then-match.wat` | refuses — red by design (the D5 repro) |

`surface-field-dispatch.wat` (2026-07-05) says in its own header *"PROVES the storage-abstraction
model — **prints 142**. Run: `cargo wat <this>`."* It dies at startup: `defsurface` gained a
required `:nature` and this file still says `:holder`.

**The disposition is cheap and the proof is recoverable** — driven before drawing this stone:
`:holder` → `:nature`, and it prints **142** again. It does not need deleting or excusing; it needs
migrating.

## The algorithm

⚠ **THE WALK IS ALL OF `docs/arc/**/*.wat`, NOT A DIRECTORY-NAME PATTERN.** The first draft of this
stone said `probes/` and `harness-*/` — and driving the tree found **two more `.wat` in a different
arc**, under `130-cache-services-pair-by-index/complected-2026-05-02/`, which that pattern would
have missed. A name-based enumeration is the same mistake as a hand-listed build-input set: it is a
second copy of "where wat lives" and it rots. **10 files total.**

A new gate walks every `.wat` under `docs/arc/` and requires each to
**either** load on the current runtime (`startup_from_source` + `FsLoader`, exactly as its sibling
does) **or** carry an explicit marker in its header. Non-vacuity guard on the walk — `complectens`
found 10 of the 15 file-walking gates in `tests/lint/` have none, and a moved root would make this
one pass forever while checking nothing.

Marker form, following the repo's convention (`rune:lint(<category>) — <reason>`), with a **CLOSED
two-category set** — because `excusare` found on 2026-08-30 that `rune:purgare`'s categories are
undefined and one of them names a mechanism absent at all three of its sites. A closed set with a
discriminating question is what `no_unknown_sequi_rune` already enforces for its family.

```
;; rune:lint(red-by-design) — <what the FAILURE PROVES>
;; rune:lint(historical)    — <what past state this preserves; it must NOT be migrated>
```

**The discriminating question:** does the file fail because failing is the point (`red-by-design`),
or because it is a photograph of a substrate that no longer exists (`historical`)?

⚠ **THE SECOND CATEGORY IS NOT SPECULATIVE — it was found by driving.**
`130-cache-services-pair-by-index/complected-2026-05-02/{substrate,test}.wat` fail on
angle-bracket type parameters, a retired syntax. They are **deliberately preserved**: their own
README calls them *"the calibration set for the complectēns spell… the failed state of the arc 130
slice 1 sonnet sweep"*, kept at the builder's verbatim instruction — *"we need to know what bad
looks like to make good - keep it here - we'll rebuild from it… we must not forget what bad looks
like."* **A gate that forced those to load would destroy the record they exist to be.** Without
this category the gate would be wrong, and a rider following the first draft would have either
migrated them or been stuck.

## ★ THE ONE CONTRACT DECISION

**The marker states WHY the file must fail, and a ROTTED file may not wear it.**

The entire value of this gate is the distinction between *declared red* and *rotted red*. The cheap
way to make it green is to rune everything — which rebuilds the graveyard **inside** the gate and
leaves it looking enforced. So: `surface-field-dispatch.wat` is **migrated, not marked**, and any
future file that cannot load must justify itself in a sentence a reader can check against the
file's own behaviour.

A marker whose reason is "it fails" is not a reason. It must name the thing being proven by the
failure.

## Blast radius

- **new**: `tests/lint/docs_wat_loads_or_declares_why_not.rs`. ⚠ **No registration edit** —
  `tests/lint/mod.rs` is a stub and `build.rs` auto-generates the module list from sibling `.rs`.
  Its own header says it: *"Add a test: drop a .rs here."*
- **migrated (1)**: `probes/surface-field-dispatch.wat` — `:holder` → `:nature`. Driven before this
  stone was written: it prints **142** again, which is what its own header promises.
- **marked `red-by-design` (3)**: `probes/red-owner-signals-child.wat` (prose already there,
  formalised), `harness-experiri/experiri-acc-head.wat`, `harness-experiri/experiri-then-match.wat`
- **marked `historical` (2)**: `130-…/complected-2026-05-02/{substrate,test}.wat`, quoting their
  own README
- **untouched (4)**: they load

## Out of scope — AFFIRMATIVELY CUT

- **Moving these files under `wat-scripts/`.** That is what the prior hand correctly refused, and
  it would break the existing gate. The gate learns the distinction; it is not hidden from it.
- **`.rs.txt` / `.wat.txt`** — the seven banked strike probes. They are invisible to any `.wat`
  walk **by the same mechanism that makes them safe from tooling**, and they are short-lived by
  design (a rider lands them within hours). ⚠ **Stated as residue, not fixed:**
  `harness-experiri/positions-3-4.rs.txt` has now sat since 2026-08-30, and its README claimed it
  was a working gate when it holds ONE assertion across EIGHT tests. A `.rs.txt` cannot be
  type-checked by this gate; that is a real hole and it is named rather than papered over.
- **Migrating the `historical` pair.** Explicitly forbidden — see the category note above.
- **Any `.wat` this walk finds that is not in the table.** The table is the state at HEAD
  `819c79b9a`; if the walk turns up more, **that is a finding to surface, not a reason to narrow
  the walk.**
