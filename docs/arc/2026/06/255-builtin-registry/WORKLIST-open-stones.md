# WORKLIST — arc 255's open stones, ordered. As of 2026-08-28, HEAD `517489500`+P1.

> Builder: *"it sounds like we've got a build list... get them on disk and we begin working on them"*
>
> **One ledger, replaced in place — never appended.** Every row names its brief, its measured size,
> and what it is blocked on. A row with no brief is not ready to strike; drawing it is the work.

## The order, and why it is this order
```
P1  the registry can detect a collision      ✅ STRUCK — the wall stands before the sweep
                                               puts ~130 more registrations behind it
O-iv-b  the collections sweep                ← NEXT: the arc's main thrust, proven shape, 32 verbs
P2  the special-form entry stops lying         two findings, one reason to change
P3  the three ignores are re-diagnosed         arc 255's OWN unlock list
P4  the skipped population is MEASURED         a measurement stone; nothing decided until it lands
O-iv-c  holon sweep                            73 verbs — needs disposition rows (span-carrying)
O-iv-d  the remainder sweep                    26 verbs — same
P5  @yields becomes mandatory at expand time   the top rung; biggest change; last
```

**P1 goes before the sweep on a dependency argument, not a preference.** O-iv-b/c/d add roughly 130
registrations to the registry. The guard against two of them colliding is currently compiled out of
the floor (P1). Adding the population first and the wall second is the wrong order.

## The rows

| id | stone | size | brief | blocked on |
|---|---|---|---|---|
| ~~P1~~ | ~~the registry can detect a collision~~ | | | ✅ **STRUCK** — see Closed |
| **O-iv-b** | the collections sweep — `map` 8 · `hashmap` 8 · `vec` 7 · `linkedlist` 5 · `hashset` 4 | 32 verbs, 5 files | *not drawn* — copy `BRIEF-STONE-O-iii` and swap the namespace | — **UNBLOCKED**, P1 landed |
| **P2** | the special-form entry stops lying — `source: ""` and `arity: -1` | 2 sites in `mod.rs`, 1 in `reflect.rs` | *not drawn* | — |
| **P3** | the three ignores are re-diagnosed | 1 un-ignore, 1 rewrite, 1 stays | *not drawn* | — |
| **P4** | the skipped population is measured | measurement only; no code change | *not drawn* | a VALIDATED instrument |
| **O-iv-c** | holon sweep — `atom` 41 · `subspace` 10 · `engram` 10 · `reckoner` 8 · `hologram` 4 | 73 verbs | *not drawn* | disposition rows (span-carrying algebra) |
| **O-iv-d** | the remainder — `uuid` 7 · `ambient` 7 · `string`/`reflect`/`bytes` 2 · six singles | 26 verbs | *not drawn* | same |
| **P5** | `@yields` mandatory when an `@arg` carries an Fn shape | macro-expand-time rule | *not drawn* | P4's measurement |

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
`O-iii` (one declaration, both doors) · `O-iv-a` (the honest word) · `P1` (the collision gate —
proven by planting a real duplicate and watching the floor run 5065/5065 GREEN past it).
