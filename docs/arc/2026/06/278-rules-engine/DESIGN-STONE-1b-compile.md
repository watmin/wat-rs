# DESIGN — Stone 1b: `compile` (rule-set → shared network)

> Arc 278 stone 1b — the engine's first ALGORITHM: `(:wat::rete::compile rules) -> Session`. Walks each
> rule's conditions left-to-right and builds the network (id→Node) with **node SHARING** — the
> non-redundancy DAG. No fire yet; this turns rules-as-data into the compiled network-as-data. Pure wat on
> the stone-1a records + stone-0 collections.

## What it delivers

`(:wat::rete::compile [rules <- :wat::core::PersistentVector] -> :wat::rete::Session)` — a fresh `Session`
whose `network` is the compiled RETE network for `rules` (empty memories, `facts` empty, `next-id` set).
`compile` IS the session constructor (Clara's `mk-session`): `(compile (collect-rules :ns))` → insert →
fire → query. **Sharing is required, not a later add** ("do not make a wasteful tree"): a condition used by
N rules yields ONE alpha node; rules with a shared condition-prefix share their beta nodes to the divergence.

## The condition shape (from arc 098 `form::matches?`)

A `Rule.lhs` is a `PersistentVector` of CONDITIONS; each condition is a `form::matches?` clause-form:
`(:TYPE (= ?var :field) (> ?qty 10) (where …) …)` — head keyword = the fact TYPE, the rest = the
binding/constraint clauses. Read the head via `(:wat::core::ast-name (:wat::core::first (:wat::core::ast->children cond)))`
(the ast-walkers from `wat/lint.wat`). The alpha node's `tests` = the condition's clauses; its identity (for
sharing) = the whole condition form.

## The algorithm (CLARA-REFERENCE §4 — the unified node-dedup)

The ONE mechanism that gives BOTH alpha + beta sharing: **build left-to-right; reuse any node whose
(structure + parent) matches, else mint a new id.** Per rule:

```
parent-id := <none>                         ; the running beta node id (left input)
for each condition C (left → right) in rule.lhs:
    alpha-id  := find-or-mint AlphaNode whose structure == (tests=C)         ; ALPHA sharing (dedup by condition)
    join-kind := (if first-condition RootJoinNode else HashJoinNode)
    join-id   := find-or-mint <join-kind> whose (condition==C ∧ parent==parent-id ∧ binding-keys==…) match
                                                                              ; BETA-prefix sharing (dedup by struct+parent)
    link: alpha-id.children ∪= join-id ;  parent-id(prev).children ∪= join-id
    parent-id := join-id
mint ProductionNode(rule.name) as a child of parent-id   ; productions are NOT shared (one per rule)
```

- **find-or-mint** = scan the network-so-far for a node equal on (kind, structure, parent); if found reuse
  its id, else `next-id` (increment). Structural equality on the condition form via `:wat::core::=`
  (forms/values are `=`-comparable).
- `binding-keys` = the variables shared between the running token's bindings and this condition (the join
  key) — for v1 compute the intersection of `?var`s bound so far ∩ `?var`s in C. (If that intersection
  logic is non-trivial, v1 may store the condition's `?var`s and refine the join-key at fire — STOP-2.)
- The network is the `PersistentMap` id→Node; `compile` threads it + `next-id` through the fold, returning
  the final `Session`.

## The ONE contract decision

**Conditions are stored as raw `form::matches?` clause-FORMS** (homoiconic WatAST in the `PersistentVector`),
NOT pre-parsed into a Condition record. Rationale: the alpha test IS the clause (reuse arc-098 `form::matches?`
directly at fire); structural `=` on the form gives sharing for free; rules-as-data stays literal. (A parsed
Condition record can come later if the fire stage needs it — not now.)

## Proof (FM-2-bis — RED at HEAD)

`tests/probe_arc278_1b_compile.rs` (RED, un-ignore on green): build TWO rules whose FIRST condition is
IDENTICAL (e.g. both start `(:Temperature (= ?t :value))`) but diverge after. `(:wat::rete::compile rules)`
→ assert the network contains exactly ONE AlphaNode for that shared condition (count AlphaNode-kind nodes
whose tests == that condition == 1, not 2) AND one shared RootJoinNode → **proves sharing**. Plus: a
single-rule compile produces a well-formed chain (alpha → root-join → production). RED at HEAD: `compile`
unknown.

## Out of scope (affirmative cuts)

- **fire / insert / query** — stones 2–4.
- non-MVP conditions (`:not`/`:exists`/accumulators/`:test`-only) — stones 6–8 extend compile's per-condition
  dispatch then. v1 compiles plain fact-conditions (`(:Type clause…)`).
- the agenda / activation ordering — fire (stone 4).

## Four questions

- **Obvious?** YES — `compile` turns rules into the network; the algorithm reads as "build left-to-right,
  reuse matching nodes."
- **Simple?** The unified dedup is ONE mechanism for both alpha + beta sharing (not two) — as simple as
  correct sharing gets. (Not trivial, but it earns its complexity — it IS the non-redundancy.)
- **Honest?** YES — real sharing (the probe proves a shared node appears once); no wasteful duplication.
- **Good UX?** YES — `(compile (collect-rules :ns))` is the whole construction step; the network is
  inspectable via `render-dag`.

## Blast radius

`wat/rete.wat` (add `compile` + find-or-mint helpers; pure wat) + un-ignore the probe + optionally a
`wat-tests/` deftest. NO Rust changes. No git in the worker. (`render-dag` from 1a verifies the output.)
