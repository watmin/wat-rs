# REVIEW — F1: the cure holds, and it carried the defect one level down

> Weighed against my own drives and my own floor.

## Accepted — verified by me, not credited to the SCORE

| row | my verification |
|---|---|
| 1★ 2★ deterministic + agrees | **8 process samples, 8/8 green.** Pre-cure I measured three distinct oracle answers and 0/8 agreement. |
| 4★ Gate A mutation-proved | **My own mutation, in a different file than yours** (`fire.wat`, you used `explain.wat`): `wat/rete/oracle/fire.wat:124 in ':wat::rete::collect-query-memory': raw 'PersistentMap/keys network'`. Restored, diff-verified. |
| 6 one definition | `topological-node-ids` at `pass.wat:12`, six callers, `fire.wat`'s inline copy gone. |
| 7 floor | **`Summary [443.313s] 5435 tests run: 5435 passed (1 slow), 21 skipped`.** Matches yours. |
| 9 blast radius | 3 × `wat/rete/oracle/` + 1 lint + 1 test pair. **Zero `src/`.** |

You converted **more than the five** — `fire.wat:544` and `collect-query-memory` were not on my list.
That is `[[a-finding-names-one-site-enumerate-the-rest]]` done right. And tripping `no_inlined_edn`
on your own lint's `strip_prefix`, capturing it, not re-running, and fixing the cause was correct.

## ⛔ THE ONE THING: the rune's reason is narrower than its use, and the cure is half a cure

`harvest-support` stores **`Support{rule, token}`** and first-wins over `derived`.

- **`rule`** is now deterministic — the outer walk is `topological-node-ids`. ✅
- **`token`** is NOT. It comes from `tokens-from-parents(beta-mem, node-parents(node-id, network))`
  (`explain.wat:26-27`), and `node-parents` (`pass.wat:405`) is the one walk still on HAMT order —
  the runed one. `tokens-from-parents` folds parent-ids **in order**, so the token vector's order is
  the parent order, and first-wins over `derived` then picks a different token per process whenever
  two tokens of one ProductionNode derive the same fact.

**That token is observable.** `wat/rete.wat:413` reads `Support/token`, then `Token/matches` and
`Token/bindings`, and **recurses to build the user-facing derivation tree** (`DerivationNode` +
`via` steps). So the "why was this fact derived" answer still varies per process.

**Reachability:** `pass.wat:384` — *"node-parents — every node that names `child-id` as a child.
Condition `:or`…"*. An `:or` condition is exactly a multi-parent ProductionNode.

**And Gate B cannot see it** — the probe compares `Support/rule`. `[[a-cure-can-carry-the-defect-one-level-down]]`.

### The rune

```
;; rune:lint(oracle-keys-order-insensitive) — … The SET of parents/tokens does not
;; depend on HAMT order. First-producer-wins is harvest-support's ProductionNode
;; walk (topological-node-ids), not this parent list.
```

The first clause is true **of the set**. The last clause is false: there are **two** first-wins
layers, and the inner one — over `toks` — *is* this parent list. `node-parents` has **eight**
consumers (`explain.wat:27`, `fire.wat:111`, `pass.wat:470`, `:646`, `:707`, `:749`, and its own
decl); the reason accounts for one.

**Honest scope: this is a READ, not a drive.** I traced the chain end to end and did not execute an
`:or`-shaped fixture. Treat it as a mechanism argument that needs its own probe.

## What to change — one line, and the rune disappears

**Sort `node-parents` too.** `pass.wat:405` → `(:wat::rete::topological-node-ids network)`, and
**delete the rune**. Then:

- the parent list is deterministic → `toks` is deterministic → `Support/token` is deterministic →
  the derivation tree is deterministic;
- the last raw walk in the oracle is gone and Gate A's exemption list is **empty**, which is a
  stronger gate than one with a subtle carve-out;
- you already showed sorting the converted sites changed no test (484/484), so the risk is measured.

A rune making a subtle claim about eight consumers is worse than no rune: it tells the next reader
the question is settled.

**Then extend Gate B** to compare the derivation the token yields, not only `Support/rule` — an
`:or` rule whose two branches derive the same fact, asserting native == oracle across ≥8 process
samples. Without that row, this exact defect returns invisibly.

## What I am NOT asking for

Do not re-open the four sites you classified as calling the verb — those are right. Do not touch
`src/`. Do not widen to F2.
