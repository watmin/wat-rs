# DESIGN — a file declares its wat contract (the loader gate's extirpare root)

> The gate that was supposed to make rot impossible has a fourth state it cannot see, and 11 files
> are living in it. This draws the ruling; it does not take it. **Builder's call.**

## The defect, grounded

`tests/lint/wat_scripts_fixes_load.rs` opens by stating a PROPERTY:

> *"ALL wat must remain correct, always — a `wat-scripts/` file that no longer type-checks goes RED
> here, so the rot cannot hide."*
> *"A stale exemplar that no longer runs is a graveyard that reads like live code (it trapped a
> prior session)."*

Its implementation, line 27, tests a NAMING CONVENTION:

```rust
} else if p.extension().is_some_and(|x| x == "wat") {
```

`.wat.disabled` has extension `disabled`. `.wat.intueri` has extension `intueri`. **Renaming a file
is how you leave the gate.** That is the convention rung of the ladder wearing a check's clothes.

Two holes, not one:

1. **Extension scoping** — 9 files carry retired angle-bracket syntax in CODE and are invisible.
2. **Directory scoping** — the walk root is `wat-scripts/`, so `docs/arc/2026/05/130-…/
   complected-2026-05-02/{substrate,test}.wat` (extension `wat`, real programs) were never in range.

Measured this session by imposing the READER on all 1826 `.wat*` files: exactly 15 refuse the angle
wall — 4 correct negative controls, and these 11.

## The populations are two different things, and one ruling cannot serve both

**`.wat.intueri` (11) — non-running BY NATURE, and honest about it.** Verbatim from their own heads:
*"NON-RUNNING; it's here so we can SEE the shapes and decide"*; *"NON-RUNNING; it exists to
COMMUNICATE the naming decisions so intueri can weigh which candidates keep their promise."* These
are naming-decision worksheets — inputs to the `intueri` ward. They were never programs and no gate
that demands type-checking is the right instrument for them.

**`.wat.disabled` (11) — real programs, switched off.** `count-logs`, `dispatch`, `metrics-summary`,
`aggregator`, `demos/aggregates/showcase`. Their own first lines still name them `wat-scripts/
count-logs.wat` — the `.disabled` was appended to silence the gate, not to describe the file. **This
is the exact graveyard the gate's header says it exists to prevent, created by renaming past it.**

## The four questions, flat, on each option

### For `.wat.disabled`

**D1 — revive: migrate to `:- [K V]`, rename to `.wat`, let the gate hold them.**
Obvious YES — a demo that ships is a demo that runs. Simple YES — one state, one rule.
Honest YES — the file's name stops lying about it. Good UX YES — the showpiece demos are the
first wat a newcomer reads, and today five of them are dead. Cost: they must type-check on the
CURRENT runtime, so this raises the floor before it lowers it. That cost is the point, not a
strike against it.

**D2 — delete them; git keeps the history.**
Obvious YES. Simple YES. Honest YES — better an absence than a corpse. Good UX **NO** — it
throws away five worked pipeline demos (`aggregator` is stage 2 of a stdin/stdout chain) that
nothing else in the corpus replaces, and the aggregates showcase is arc 293/294's own exhibit.

**D3 — keep the state; widen the gate to require PARSE only.**
Obvious NO — "disabled but must still parse" is a third contract no reader would predict.
Honest **NO** — it institutionalises the rot channel and calls it a feature. Fails before UX.

### For `.wat.intueri`

**I1 — convert to `.md` with fenced wat.** Obvious YES — they ARE documents. Simple YES.
Honest YES. Good UX YES — and they land under the executable-guides stone's fence gate for free,
so one instrument covers both. Cost: 11 conversions, and the fence gate must exist first.

**I2 — keep the extension; gate at the READER, not the checker.** Obvious YES. Simple YES — the
contract is "must parse," which is exactly what a shape-worksheet needs. Honest YES — it matches
their stated NON-RUNNING nature without pretending they are programs. Good UX YES.

**I3 — delete the ones whose decision already shipped.** Obvious YES. Simple YES. Honest — needs
one check per file: arc 251's symbol correction and arc 259's bracket names appear ruled, so
those worksheets may be spent. Good UX YES for the spent ones, **NO** as a blanket rule.
I1/I2 and I3 compose: rule the contract, then retire what is spent.

## The root fix, whichever way the above goes

**Every wat file DECLARES its contract, and the set of contracts is CLOSED and TOTAL:**

```
LOADS    parses + type-checks on the current runtime      (the default; the gate asserts it)
READS    parses; type-checking is not claimed             (shape worksheets)
REFUSED  must NOT load, with a co-located assertion       (.wat.bad negative controls,
         naming the EXACT refusal                          which already work this way)
```

A file in no contract is a RED, and the walk is scoped **by content, repo-wide** — not by extension
and not by directory. That closes both holes at once and leaves no fourth state to rot in. Climbing
the ladder: today's convention (name it `.wat` and you are gated) becomes a check that fires at
construction (an unclassified wat file goes red), and renaming stops being an exit.

`.wat.bad` proves the shape already works — a REFUSED file with a co-located `.rs` asserting the
exact diagnostic is the strongest artifact in the corpus, and it is the model for the other two.

## Sequencing

Independent of the prose strike (`BRIEF-STONE-the-heresy-stops-being-taught.md`), which touches
comments only. The 6 `.wat.disabled` + 3 `.wat.intueri` files carrying angle syntax in CODE are
named there and deliberately NOT fixed there — a code migration under an unruled contract would
just re-park them. **This stone rules the contract first; the code migration follows it.**

D1 raises the floor on landing. That is a sequenced gate-and-repair, same shape as the
executable-guides stone, and it runs from a green floor.
