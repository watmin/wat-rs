# DESIGN — census I: one family, two units

Census audit section I (`../vigilia-2026-09-05/recon/census-name-audit.md:112-118`), re-grounded at
HEAD 2026-09-06.

## ⚠ Severity, stated first — this is the mildest row in the census

**Latent, and the audit's own word is "benign but real."** No number is wrong and the consumer is
impeccable. Say that plainly rather than dress it up: the whole defect is that a key's name implies
the wrong unit, inside a family where its two siblings carry the other one.

It is still worth closing — it is a one-key rename with a real gate behind it — but the record
should not imply more than a reader's second glance was ever at risk.

## The finding

Three keys in one `seed:` family, `src/rete/kernel/fire/pass/alpha.rs`:

| key | site | unit |
|---|---|---|
| `seed:batch-class-uniform` | `:272` | **per CLASS** |
| `seed:batch-class-mixed` | `:266` | **per CLASS** |
| `seed:mixed-class-activate` | `:363` | **per FACT** — it sits inside `for (i, fact) in input_facts.iter()` |

Two count classes; the third counts facts. All three read as class-shaped, and the odd one is the
only one whose name does not say so.

## The consumer already knows — and that is the point

`pass_semantics.rs:713-757` reads all three and spells the units out in its own assertion messages:

> *"{mixed} class(es) did"* · *"it activated {activated} fact(s)"* · *"{uniform} class(es) did.
> **Class-uniformity is per CLASS, not per session**"*

The knowledge exists, correctly, one file away. That is census H's shape exactly — and H is what
happens when the meaning lives only downstream. Here nothing has gone wrong **yet**.

## THE ONE CONTRACT DECISION

**Rename `seed:mixed-class-activate` → `seed:mixed-fact-activate`.**

`class` is what implied the unit, so `fact` replaces it; `mixed` still names which classes the facts
belong to; `activate` still names the path. The two siblings are correct and do not move.

Not `seed:mixed-class-fact-activate` — it carries both nouns and reads as though it counted pairs.

## Also ships

State the family's units at the sites: two lines noting that the two `batch-class-*` keys increment
once per class while this one increments once per fact of a mixed class. The audit's standing
complaint across this whole census has been that units live in the consumer; this is the cheapest
possible place to stop repeating it.

## Out of scope = REJECTED

- Renaming the two correct siblings.
- Changing what is counted, or where the bump sits.
- Census J–M.

## Mutation proof — available, and therefore required

`pass_semantics.rs:749` asserts `activated == 3`. **Delete the bump → `0 != 3` → RED.**

Unlike census D and G, where no red existed and saying so was the honest report, this counter is
genuinely gated. Drive the live test, quote the failure, restore.

## STOP triggers

1. The mutation does not RED → STOP and report; the counter is not gated and this becomes census
   D's shape rather than F's.
2. Any measured value changes → STOP. This renames one key.
3. A sibling's count moves → STOP; they are correct.
