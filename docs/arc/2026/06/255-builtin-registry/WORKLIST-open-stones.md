# WORKLIST — arc 255's open stones, ordered. As of 2026-08-28, HEAD `517489500`+P1.

> Builder: *"it sounds like we've got a build list... get them on disk and we begin working on them"*
>
> **One ledger, replaced in place — never appended.** Every row names its brief, its measured size,
> and what it is blocked on. A row with no brief is not ready to strike; drawing it is the work.

## The order, and why it is this order
```
P1  the registry can detect a collision      ✅ STRUCK — the wall stands before the sweep
                                               puts ~130 more registrations behind it
O-iv-b  the collections sweep                ✅ STRUCK — 8 new doors + 24 collapses, -556 lines
P2  the special-form entry stops lying       ✅ STRUCK — show-source honest, `if` reports arity 3
P3  the SEVEN ignores are re-diagnosed      ✅ STRUCK — 2 un-ignored, 5 re-pointed, a lint added
P4  the silent skip is NAMED + FROZEN     ✅ STRUCK — 49 of 382, ward's 96/384 refuted
H-1a    holon declares its REAL arity        ✅ STRUCK — 35 verbs, -542 lines, 5 doc lies exposed
H-1b    the same for atom.rs               ✅ STRUCK — 52 of 60, -829 lines, 58 doc lies exposed
Q + Q-2 the value door carries a SPAN,
        and USES it                          ✅ STRUCK — 20 diagnostics move from a Rust line to the caller
O-iv-c-0 the require_* family takes a ref  ← NEXT: 9 sigs + 109 call sites; makes the sweep a DELETION
O-iv-c-1 holon sweep, the four small files    32 SHELL verbs
O-iv-c-2 holon sweep, atom.rs                 41 SHELL verbs
O-iv-c  holon sweep                            73 verbs — ⛔ BLOCKED on Q
O-iv-d  the remainder sweep                    26 verbs — same
P5  @yields becomes mandatory at expand time   the top rung; biggest change
P6-a a special form NAMES its impls        ✅ STRUCK — show-source prints check·eval·tail
P6-c the two matches collapse to lookups     the megafile: 111 eval arms + 8 tail arms
```

**P1 goes before the sweep on a dependency argument, not a preference.** O-iv-b/c/d add roughly 130
registrations to the registry. The guard against two of them colliding is currently compiled out of
the floor (P1). Adding the population first and the wall second is the wrong order.

## The rows

| id | stone | size | brief | blocked on |
|---|---|---|---|---|
| ~~P1~~ | ~~the registry can detect a collision~~ | | | ✅ **STRUCK** — see Closed |
| ~~O-iv-b~~ | ~~the collections sweep~~ | | | ✅ **STRUCK** — see Closed |
| ~~P2~~ | ~~the special-form entry stops lying~~ | | | ✅ **STRUCK** — see Closed |
| ~~P3~~ | ~~the ignore ledger re-diagnosed~~ | | | ✅ **STRUCK** — see Closed |
| ~~P4~~ | ~~the silent skip becomes NAMED+FROZEN~~ | | | ✅ **STRUCK** — 49 of 382, see Closed |
| ~~H-1a~~ | ~~holon declares its REAL arity~~ | | | ✅ **STRUCK** — see Closed |
| ~~H-1b~~ | ~~the same for `atom.rs`~~ | | | ✅ **STRUCK** — 52 of 60, 58 doc lies |
| ~~Q~~ | ~~the value door carries the CALL SPAN~~ | `ValueHandler` type · `dispatch_substrate_impl` + its 1 caller · the macro sniff · 19 twins take an ignored param | `BRIEF-STONE-Q-the-value-door-carries-the-call-span.md` | ✅ **STRUCK** — see Closed |
| ~~Q-2~~ | ~~the threaded span must be USED~~ | | | ✅ **STRUCK** — 20 sites fixed, 0 runed |
| **O-iv-c-0** | the `require_*` family takes `&Value` | 9 signatures + 109 call sites; mechanical | `BRIEF-STONE-O-iv-c-0-the-require-family-takes-a-reference.md` | — **READY** |
| **O-iv-c-1** | holon sweep — `subspace` 10 · `engram` 10 · `reckoner` 8 · `hologram` 4 | 32 SHELL verbs | *not drawn* | O-iv-c-0 |
| **O-iv-c-2** | holon sweep — `atom.rs` | 41 SHELL verbs | *not drawn* | O-iv-c-1 proving the shape |
| **O-iv-d** | the remainder — `uuid` 7 · `ambient` 7 · `string`/`reflect`/`bytes` 2 · six singles | 26 verbs | *not drawn* | same |
| **P5** | `@yields` mandatory when an `@arg` carries an Fn shape | macro-expand-time rule | *not drawn* | P4's measurement |
| ~~P6-a~~ | ~~a special form names its implementations~~ | | | ✅ **STRUCK** — see Closed |
| **P6-c** | the eval and tail matches collapse into registry lookups | 111 eval arms + 8 tail arms | *not drawn* | P6-a's mechanism + its row-0 census |

## What each row is FOR — one line, so a fresh reader does not need the NOTE

- **P1** — `mod.rs:348`'s `debug_assert!` is the only thing standing between two homes claiming one
  FQDN and a silent `HashMap` overwrite, and it is compiled out in release, which is the floor.
- **O-iv-b/c/d** — `:wat::core::apply` reaches 49 of 380 verbs; these migrate the rest that can be
  migrated. The machine is built and proven (O-iii); each wave is a commit per namespace.
- **P2** — `(:wat::core::show-source :wat::core::if)` returns `""` while that verb's own shipped
  prose promises otherwise; `metadata-of` reports `:arity -1` for a form declaring three fixed args.
- **P3** — three `#[ignore]`s carry one identical reason string covering three different truths; one
  masks a test that PASSES.
- **P4** — two gates skip every entry absent from the checker, silently, at `mod.rs:512` and `:742`.
  Nobody knows how many. The ward said 96/384 against my anchored 382 — two instruments, two
  populations, so the number is open.
- **P5** — the `@yields` gate's only measured subject is the fixture written to exercise it.

Full detail and the disk citations: `NOTE-an-absence-recorded-as-an-answer-the-class-behind-the-apply-defect.md`.

## Rules this list obeys

- ⛔ **A row is not "ready" until its brief is on disk.** Drawing is the work; "we know what to do" is
  not a work item. `[[feedback_nothing_blocks_it_is_not_a_work_item]]`
- ⛔ **No row gets struck on a size I have not measured.** Every count above traces to a command in
  the NOTE or to `wat-scripts/hunt/stone-o-*.awk`.
- ⛔ **This file is REPLACED, never appended.** A worklist with strata is a worklist nobody trusts.

## Closed, this arc, for the record

`A-i`…`F` (scalars, collections, String) · `HOME-8`…`HOME-13` · `STONE G` (provenance) ·
`STONE N` (apply's authority) · `O-i` (the arity guard) · `O-ii` (the defclause door) ·
`O-iii` (one declaration, both doors) · `O-iv-a` (the honest word) · `P4` (the checker-skip debt is a NAMED FROZEN ledger — 49 of 382, and the ward’s 96/384 refuted
by asking the gate’s own instrument the gate’s own question) · `P3` (the ignore ledger — 2 un-ignored, 5 re-pointed at their REAL blockers, and one of those
blockers turned out not to belong to this arc at all) · `P6-a` (a special form names its impls — and publishing them exposed two INVERTED doc comments
on `if`, buried since arc 258.4) · `P2` (the special-form entry — show-source stops returning "", `if` reports arity 3 not -1) ·
`O-iv-b` (the collections sweep — 8 new doors, 24 two-fn collapses, 24 `expect("arity-checked")`
sites deleted, −556 lines) · `P1` (the collision gate —
proven by planting a real duplicate and watching the floor run 5065/5065 GREEN past it).
