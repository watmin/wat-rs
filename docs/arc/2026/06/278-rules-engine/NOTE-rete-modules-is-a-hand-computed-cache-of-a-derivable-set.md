# NOTE — `RETE_MODULES` is a hand-computed cache of a set `RETE_OPS` already determines

**Found:** 2026-08-28, on the `claude-compute` integration branch (main × grok-rete), while
resolving where grok-rete's two keyword-converter rows belong.
**Status:** open. The immediate blocker is fixed; the class is not.
**Lands in:** main (it is main's invariant, its doc, and its ward). Filed here because 278
is what forced it into view.

---

## The claim the module doc already makes

`src/rete/vocabulary.rs`, the naming-rule section:

> **`rete_name` = `core_name` with `rete::` inserted immediately after `wat::`.** One rule,
> **no hand-maintained module list to drift**: `RETE_MODULES` collapses to `core::`/`holon::`
> because every `core_name` is already rooted at `:wat::core::` or `:wat::holon::`, so the
> rule **PUTS every new row inside an admitted module BY CONSTRUCTION.**

That is the rung-3 shape: admission is not checked, it is unrepresentable-otherwise. It was
true when written.

## Arc 255 falsified the premise, and the list grew back

The premise is *"every `core_name` is already rooted at `:wat::core::`"*. Arc 255's rehoming
is precisely the campaign that makes that false — `:wat::string::`, `:wat::i64::`,
`:wat::f64::`, `:wat::map::`, `:wat::vector::`, `:wat::vec::`, `:wat::linkedlist::`.

| branch | `RETE_MODULES` |
|---|---|
| `origin/grok-rete` (pre-255 premise) | `[core, holon]` — 2 |
| `origin/main` (post-255) | `[core, holon, string, i64, f64, map, vector, vec, linkedlist]` — **9** |

**The hand-maintained list the naming rule existed to delete has returned**, one entry per
rehomed family that has a row. Nothing announced this; the rule's doc still says the list
"collapses to `core::`/`holon::`".

## main is already doing the derivation — by hand, per stone

The entries are not chosen; they are *forced*, and each stone records having measured that.
From the const's own doc block:

> ⚠ E-ii adds `:wat::rete::vector::` and `:wat::rete::vec::` … Both prefixes are needed
> (unlike E-i's map-only need) because BOTH families have a moved `RETE_OPS` row here.
>
> ⚠ E-iii adds `:wat::rete::linkedlist::` ONLY … **Measured (not assumed** …**): `RETE_OPS`
> has NO HashSet row at all, so no `:wat::rete::hashset::` entry is needed here — adding one
> anyway would be exactly the unforced entry E-i's `hashmap::` note warns against.

The rule being applied is exactly *"an entry exists iff a row forces it"* — a derivation,
executed by a human, once per stone, with the measurement written down as prose. That is the
definition of a cache kept in parity by discipline.

## It is provably a cache — measured

Derived set = `{ rete-namespace(row.rete_name) : row ∈ RETE_OPS }`, computed over
`origin/main`'s 74 rows:

```
rows: 74      hand-maintained: 9      derived-from-rows: 9
in HAND but no row uses it (dead entries):        (none)
used by a row but NOT in hand (stranding hole):   (none)
```

The two sets are **identical on main today**. Replacing the const with the derivation is
therefore behaviour-preserving, not a semantic change.

## What made it visible: `keyword`

Stone E-iv moved `keyword/{to,from}-string`'s `core_name` onto `:wat::keyword::*` and
**parked** the rete ruling, on a stated premise (`src/rete/purity.rs:2450`):

> ruling on THIS axis (RETE-fireability) for **a verb nothing forces into a `where`** is out
> of this stone's scope — same restraint as **E-iii's refused `RETE_MODULES` entry**.

278 then minted rows for exactly those two converters. The premise no longer holds, and the
row lands between two invariants that now contradict:

- naming rule (`vocabulary.rs:1773`) demands `rete_name` = `:wat::rete::keyword::to-string`
- admission (`:1746`) refuses it — `:wat::rete::keyword::` is not in the list

The assertion's own message is the tell: *"the naming rule is supposed to make this
impossible."* It is supposed to; the list is why it no longer does.

## The proposal

Delete `RETE_MODULES`. Derive the admitted set from `RETE_OPS` once, in a `OnceLock`, exactly
as `rete_op_index`'s `BY_NAME` already does. `rete_vocabulary_admitted` keeps its signature and
its meaning; the "necessary, not sufficient" two-step is untouched — a namespace is admitted
because a row uses it, and a specific verb still has to resolve through `rete_op_for`.

**Four questions.** Obvious: YES — one rule, stated once, and the rule the doc already claims.
Simple: YES — a derivation replaces a list; no consumer changes (`purity.rs:1154` is the only
one). Honest: YES — the boundary becomes what it says it is and cannot lag a rehoming.
Good UX: YES — a new row can no longer be stranded, and no stone has to remember to measure.

### Consequence that must not be skipped

`every_row_is_admitted` (`vocabulary.rs:1746`) becomes **vacuous** — it would assert a property
the construction guarantees. This repo treats a vacuous test as a defect, so that ward must be
**deleted**, with its property restated as structural in the module doc, never left standing as
a tautology that reads like coverage. Its sibling `rete_name_is_core_name_with_rete_inserted_after_wat`
stays meaningful and is untouched.

## Why it was not done on the integration branch

`claude-compute` fixed the immediate case instead (paired `core_name`/`rete_name` correction on
the two converter rows, plus the forced `:wat::rete::keyword::` entry). The refactor was not
taken there because:

1. It is **main's invariant, main's doc, main's ward** — including the judgement to delete a
   ward, which should not be made on a branch main never reads.
2. It **deletes a const main edits every stone** (E-i, E-ii, E-iii each appended; more homes
   are in flight). A deleted const on the integration branch against a growing one upstream
   conflicts on every refresh, and unlike the `.wat` corpus there is no re-derive escape — the
   const either exists or it does not.

The interim fix is not wasted: the paired name correction is right under either design, and
only the one list entry becomes unnecessary when this lands.
