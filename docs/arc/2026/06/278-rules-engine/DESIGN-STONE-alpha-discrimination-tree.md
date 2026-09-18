# DESIGN-STONE — the alpha network becomes a discrimination tree (the kernel's, brought home)

> **Origin (2026-07-31).** A0 (deep-cascade) entered the grid and came back `:winner :clara` at
> `[50 100]` — all five runs, `max 0.9269`. It is the first Clara win this project has found by
> measurement rather than inherited from R4, and it was invisible for months because A0's ladder
> topped out at 300 derived facts on a 4 ms workload.
>
> Then the builder: *"the whole thing is a tree?.. only go down paths that are actually possible?"*
> and *"we should study how we did this kind of thing in the kernel."* We did. Six months ago, twice,
> at line rate — and the post says outright what it was: **"The XDP walker is the alpha network."**
> We built a rete in wat and put a linear scan where that tree goes.

## The finding — measured, not asserted

Holding derived facts **constant at 10,000** and varying only cascade depth
(`a0_depth_cost_split_at_equal_work`, test build, read for proportions):

```
phase              depth10×w500   depth50×w100      delta
alpha:match           29.495 ms      117.072 ms   +87.577      ← 79% of the whole delta
root-join              2.872          3.030        +0.158
hash-join              6.590          5.703        −0.887
accumulate             0.048          0.804        +0.756
filter                 0.039          0.804        +0.765
ROUND LOOP            59.951        170.351      +110.399
```

**The mechanism** (`kernel.rs:2009-2050`): the alpha index keys on **fact type only**.

```rust
let alphas = alpha_by_type.get(fact_class)   // ← type is the ONLY tier
for aid in alphas { alpha_match_inner(...) } // ← every alpha of that type, one at a time
```

A depth-D cascade has two fact types and 2D alpha nodes — D per type — so every fact is
match-tested against **all D levels** and exactly one can succeed. Calls = `facts × D`. At constant
work, D 10→50 predicts 5×; measured 3.97×. Confirmed.

**Two things already work and are NOT the problem** — both grounded this session, both worth stating
so nobody re-solves them:

- **Type discrimination exists.** That is tier 1 (P8). It is also the *only* tier.
- **Dead branches are already pruned downstream.** Root-join does
  `match d_alpha.get(node_id) { Some(els) if !els.is_empty() => …, _ => continue }` — an alpha that
  matched nothing costs a visit and no work. Hash-join measured **6.59 → 5.70 ms** across a 5×
  depth increase; it did not grow. The four per-round full-network scans total **+0.8 ms of +110.4**.

So the pruning is downstream. **What does not exist is discrimination at the entry**, and the entry
is 79% of the depth cost.

### ⚠ The record has been carrying a WRONG diagnosis of this axis

`REALIZATIONS.md:2038` and `PVRITAS VERVM, NON CELERITATEM` both attribute the deep-cascade
crossover to **T3 — round-based semi-naive vs per-element incremental**. That is **wrong for this
axis**: per-element propagation changes *when* a fact is pushed, not *how many alpha nodes it is
tested against*. `facts × D` survives any round-structure change untouched. T3 would not have moved
this number. The diagnosis has steered this axis since R22 and was never measured until today
(`feedback_ground_the_substrate_not_just_the_chronicle` — a wrong diagnosis in the record is worse
than a thin one).

## The reference implementation — ours, already shipped twice

`holon-lab-ddos/veth-lab/filter/src/tree.rs:75` (packet tree, 1M rules, ~5 tail calls/packet) and
`holon-lab-ddos/http-lab/proxy/src/expr_tree.rs` (L7 expression tree, 1M rules, ~1µs hit / flat 50ns
miss):

```rust
pub(crate) struct ShadowNode {
    dim_index: usize,                                  // which dimension this node branches on
    action: Option<ShadowAction>,                      // terminal
    children: StdHashMap<u32, Rc<ShadowNode>>,         // value → subtree   (equality fan-out, O(1))
    wildcard: Option<Rc<ShadowNode>>,                  // rules that don't constrain this dimension
    range_children: Vec<(RangeEdge, Rc<ShadowNode>)>,  // (op, threshold) → subtree, guard-evaluated
}
```

Three edge kinds in one node: equality hashes, unconstrained takes the wildcard, ranges are a short
guarded `Vec` walked at traversal. `Rc` gives subtree sharing; the compiler partitions rules by the
first unprocessed dimension and content-hash-dedups identical subtrees (~2 nodes per rule at 1M).

**Ranges were never meant to hash** — the builder's *"the lack of hashing… its irrelevant"* is the
design, on disk, from February.

## ★ THE ONE CONTRACT DECISION

**The tree may OVER-approximate. It may never UNDER-approximate. `alpha_match_inner` remains the
sole authority on whether a condition holds, and the sole producer of bindings.**

The walk returns a **candidate set**; the matcher runs on the survivors exactly as it does today.

For every fact: `walk(fact) ⊇ { alphas that actually match }`.

This is the whole design, and three consequences fall out of it:

1. **The analyzer's incompleteness is safe by construction.** Any clause the tree builder cannot
   prove a discriminator for — `not=`, `or`, `not`, a computed operand, a shape it does not
   recognise — is placed on the **wildcard** edge and therefore always walked. Being unable to
   understand a condition costs a wasted `alpha_match_inner` call. It can never drop a derivation.
2. **The tree cannot disagree with the matcher about semantics**, because it never decides a match.
   A superset is a perf loss; a subset would be a silent wrong answer. Only one of those is
   representable.
3. **The invariant is directly testable** — run both, compare sets. The stone carries its own oracle
   in addition to the wat one.

**The builder MUST consume `classify_rete_clause`** (`matcher.rs:365`), not re-parse conditions. That
classifier was extracted in arc 294 item 9a precisely to close a drift hole between the matcher and
the validator; a third private parser would re-open it.

## Scope — alpha only, prune only, per type

- **Alpha only.** The kernel tree bakes the *beta join* at compile time because all rules are known
  upfront and **nothing derives**. Our engine's entire point is that derived facts cascade, so the
  beta network stays runtime. We take the alpha half and leave the pre-joined DAG behind.
- **Prune only.** The walk yields candidates; bindings continue to come from `alpha_match_inner`.
  `D` calls → `~1`, which is the whole measured win, without teaching the tree to bind variables.
- **Per type.** Tier 1 is unchanged: `fact_class` selects a root; each type roots its own tree over
  its own declared fields. Their `dim_index` indexes a fixed global 9-field order; ours is the
  record's declared field order (deterministic, already available via `field_names_cache`).
- **Values, not `u32`.** `children` keys on `Value`. Already established — the join indexes are
  `HashMap<Vec<Value>, Vec<Token>>`.
- **Built once per FIRE, at setup** — beside P8's `alpha_by_type`/`alpha_cond`, from the immutable
  network, never per round. *(Corrected 2026-07-31: this line first read "at network-compile time,"
  which is a DIFFERENT time and a lifecycle we do not have — nothing survives between fires today.
  The BRIEF pointed at the fire-setup site and the rider built it there, correctly. Moving
  construction to `compile` so the artifact persists across fires is its own stone,
  `DESIGN-STONE-compile-actually-compiles.md`, and it is a prerequisite for R0 — not a phrase to
  smuggle in here.)*
  `SETUP: indexes` is 0.135–0.250 ms today, so there is room to spend there; the gate bounds it.

## Blast radius

`src/rete/kernel/` (the setup-time index construction in `kernel/arm.rs`, and step 1's inner loop in `kernel/fire/`) plus one new
module for the tree. **No `wat/` change — the oracle does not move** (R22 `OCVLI NOVI, ORACVLVM
IMMOTVM`). **No change to `matcher.rs` semantics**; it is consumed, not edited.

## The gate

1. **The invariant, as a test:** for every fact in the `[50 100]` cascade, `walk(fact)` is a superset
   of the set `alpha_match_inner` accepts. A subset anywhere is a hard fail.
2. **The candidate set actually shrinks:** mean candidates per fact at `[50 100]` is `~1`, not `D`.
   Row 1 passing while this fails means a correct tree that discriminates nothing.
3. **`alpha:match` falls materially** at `[50 100]` in `a0_depth_cost_split_at_equal_work`, and the
   depth-10 column does not regress.
4. **Setup cost stays bounded** — tree construction does not push `SETUP: indexes` past ~2 ms at
   `[50 100]`.
5. **`:accuracy :match` on every grid axis, unchanged.** The `:derived` set comparison is the
   witness that nothing was dropped; the ~24 count differentials and the full release floor hold.

**Not a gate: the `[50 100]` cell flipping off `:clara`.** That is the hoped-for outcome and it is a
*report*, not a pass condition. Gating on it would invite tuning the measurement.

## Out of scope = REJECTED (affirmative cuts)

- **Range/mask edges.** `range_children` exists in the node type and is **unpopulated by this
  stone** — conditions carrying `< > <= >=` take the wildcard edge, which is correct under the
  contract and simply unpruned. Populating it is `DESIGN-STONE-alpha-tree-range-edges` in this arc,
  drawn when a corpus needs it. The node shape does not change to add them.
- **Binding production in the walk.** The matcher stays the sole binder. Its own stone, if ever.
- **The pre-joined beta DAG.** Structurally inapplicable — we derive facts; the kernel does not.
- **Dimension-ordering optimization** (most-selective-first). Declared field order is deterministic
  and sufficient; selectivity ordering is a later refinement with its own measurement.
- **Touching T3, T2 or T4.** T3's diagnosis is corrected by this document, not superseded — the
  round-based/per-element question is real, it is simply not what this axis was paying for.
- **`wat/rete.wat`.** The oracle is never optimized.
