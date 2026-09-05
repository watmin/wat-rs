# DESIGN — STONE: two rules cannot silently disagree about one node

## ⛔ FIRST, A CORRECTION — the wall I proposed is unbuildable

I wrote, in the previous SCORE's verdict:

> *"the DESIGN's exclusivity argument is an unenforced convention … a node claimed twice should
> raise, at the same site the grandchild wall already lives."*

**It cannot be built.** `Claim` carries only `{form}`. Two rules claiming node 5 each assert
`Claim {form: 5}` — **byte-identical facts with no provenance.** Whether or not the engine dedupes,
nothing distinguishes *"two rules claimed this"* from *"one rule claimed this"*. The wall has nothing
to look at.

★ Building it would require `Claim {form, rule}` with each rule hand-writing its own name — a
schema change, resting on a hand-written string that a rule could get wrong, to detect a condition
that **may be entirely harmless**.

## ✅ THE WALL THAT IS BUILDABLE — catch the HARM, not a proxy for it

```wat
;; wat/fmt.wat, breaks-map — today
(:wat::hashmap::assoc m (:wat::fmt::Break/id b) (:wat::fmt::Break/kind b))
                      ↑ a second Break for the same node OVERWRITES, silently
```

Two rules asserting Breaks for one node is only a defect **when they disagree**. Two rules asserting
the same kind is redundant and harmless. So:

> **If a node already carries a Break of a DIFFERENT kind, raise, naming the node and both kinds.**

| | this wall | the one I proposed |
|---|---|---|
| schema change | **none** | `Claim` gains a `rule` field |
| rests on | the facts already asserted | a hand-written rule-name string |
| fires on | **a real layout conflict** | a structural proxy that may be benign |
| site | `breaks-map`, beside the grandchild wall | same |

## WHY NOW, BEFORE THE PENDING RULES

> **Builder:** *"we should address before we make progress on the pending items?"*

**Yes, and the reason is population.** The next stone adds rules — R2 (`fn`), and `foldl`/`map`/
`filter` all become files. Every new rule file is a fresh chance for two rules to reach for the same
node. **A wall is cheapest to install when the violation count is still zero**, and its value is
exactly proportional to how many rules come after it.

★ It also closes this arc's recurring failure by a different route than the last three stones did.
Those made a collision *unrepresentable* (no columns) or *unownable* (one node's children). This one
accepts that a collision may still be constructed and makes it **LOUD**. Three rungs of the same
ladder, and the top two are already in place.

## THE ACCEPTANCE

```
1  the wall FIRES — a throwaway rule asserting a different kind for an already-broken node raises,
   naming the node and both kinds. Then the rule is deleted.
2  a redundant-but-AGREEING duplicate does NOT raise — same node, same kind, silent.
3  every existing fixture unchanged and idempotent — the wall is invisible to correct rule sets.
```

⚠ **Row 2 is not decoration.** A wall that fires on agreement would forbid the legitimate case where
two rules independently reach the same conclusion, and would make the rule set brittle in exactly the
way the extensibility requirement forbids. **Both rows or the stone is not done.**

## OUT OF SCOPE

- **`Claim {form, rule}`** — rejected above, with the reason. Not deferred; rejected.
- **R11 → always-break, and `BlankBefore`** — the next stone, both already ruled.
- **A static lint over the rule files** (two rules dispatching on one head symbol). Plausible, but it
  would need to extract a dispatch key from a `defrule`'s `:when`, and this runtime wall catches the
  same class at the point of harm without parsing rules. Not built.
