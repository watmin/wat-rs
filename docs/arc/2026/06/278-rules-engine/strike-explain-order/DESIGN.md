# DESIGN — F1: the referee for explain is not a function

> Drawn 2026-09-05 at HEAD `7cae74e57`. Source: vigilia 2026-09-05 F1 (`conferre` L1-1, driven by
> `experiri`). **Re-driven by the orchestrator at THIS HEAD, 8 process samples**, because the rete
> code changed under A1:
>
> ```
> native   vex::aaa ×8                       stable
> oracle   ccc fff fff fff fff fff ccc ddd   THREE distinct answers, agrees with native 0/8
> control  vex::solo both, ×8                stable (single producer)
> ```
>
> `experiri` measured 2/8 agreement pre-A1; this run got 0/8. The *distribution* moves per process.
> That is the finding.

## The defect

`fire-rules-explain$oracle` attributes a derived fact to a **different rule on different runs of the
same program**. It is the reference implementation — the referee the native is checked against — and
it is not a function.

`harvest-support` (`wat/rete/oracle/explain.wat:10-49`) folds over
`(:wat::core::PersistentMap/keys network)` — HAMT order — with **no sort**, first-producer-wins. Its
own doc at `:53` claims *"First-producer-wins, matching the native index."* It does not match the
native, and it does not match itself twice running.

The differential that should catch this compares only `PersistentMap/length` — a cardinality, which
is invariant under the defect.

## ★ This exact bug was found and fixed once, in the file next door

`wat/rete/oracle/fire.wat:151-158`, verbatim:

> *"WHY sort: compile mints ids left-to-right, so ascending id IS topological. `PersistentMap/keys`
> is HAMT order — not that. … **oracle-derived changed every run, sometimes []**. Native sorts
> (`sorted_node_ids`); the spec must too."*

with the mechanism at `:159` — `(:wat::core::sort (:wat::core::into (Vector i64) (PersistentMap/keys network)))`.
The law was written down, applied in one file, and never carried across.

## ⛔ A finding names one site — and there are FIVE

Raw `PersistentMap/keys network` walks in the oracle. **Only `fire.wat:159` sorts.**

| site | shape (read as a fragment — CLASSIFY, do not assume) |
|---|---|
| `explain.wat:49` | **the driven defect.** first-producer-wins over HAMT order |
| `pass.wat:395` | conjes into a `PersistentVector` — **builds an ordered structure out of unordered keys by construction** |
| `pass.wat:177` | fold, `-1` seed, returns `node-id` or `-1` — can yield the LAST match rather than the first |
| `pass.wat:224` | same shape as `:177` |
| `fire.wat:124` | assocs into a map keyed by qname — order-insensitive **unless qnames collide** |

**I read tails, not whole functions.** Four of these are a classification task with evidence
required, not four asserted defects. A site that is genuinely order-insensitive is a fine answer —
it just has to be shown, and then runed so the next reader does not re-derive it.

## The one contract decision, pinned

**One shared verb, and a raw keys-walk over `network` has no form in the oracle.**

- **Check rung:** `explain.wat:49` sorts, exactly as `fire.wat:159` does.
- **Shape rung:** mint `:wat::rete::topological-node-ids network` (the sort + `into` in one place),
  make `fire.wat:159` call it too, and gate that no raw `(PersistentMap/keys network)` survives in
  `wat/rete/oracle/**` outside that verb — with a per-site `;; rune:lint(...)` for any walk proven
  order-insensitive.

Sorting `explain.wat` alone is the check rung and leaves four sites and the next one free to appear.

## Gates — and be honest about which one is load-bearing

**Gate A — structural, deterministic, MUST be mutation-proved.** No raw keys-walk over `network` in
`wat/rete/oracle/**` outside the verb. Deterministic in both directions.

**Gate B — behavioural, the differential.** Native and oracle attribute the same rule, ≥8 producing
rules, with the single-producer control. **Green after the cure; at HEAD it is red only
probabilistically** (`experiri` saw it agree 2/8). A probabilistic red is exactly the "known flake"
shape this repo forbids, so **Gate B is not the proof — Gate A is.** Gate B earns its place by being
deterministic *after* the cure and catching a regression in attribution that Gate A cannot see.

State that split in the SCORE. Do not present Gate B's red as the demonstration.

## Scope

**IN:** the cure, the shared verb, all five sites classified, both gates. Floor GREEN at the end.

**OUT, affirmatively cut:** `wat/rete/syntax.wat:23` (keys of `params`, not `network`),
`accum-pass.wat:261/300` (keys of an accumulator map, not `network`), and F2 (retract multiplicity).
