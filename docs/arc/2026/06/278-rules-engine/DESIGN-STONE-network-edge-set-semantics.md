# DESIGN-STONE — network edges are a SET: `network-add-child` becomes idempotent

> **Origin (2026-07-30, grid axis A8):** the node-share axis is the one cell Clara wins (57×), and at
> N=20/M=500 the wat side needed >4 GiB to join 500 facts against 20 rules — it took a workstation
> down. The compile-time census (`wat-scripts/scratch-pad/probe-node-share-dedup.wat`) counted the
> network at `4 + 2N` nodes and cleared the compiler. **That census was blind to edges.** The fire
> census built this session (`a8_node_share_fire_census`, `src/rete/kernel/tests/node_share_cost.rs`) counts both, and the
> defect is exact: **nodes are shared perfectly; edges are duplicated once per rule.**

## The measurement (the RED gate, on the disk, currently failing)

`cargo nextest run --release -E 'test(a8_node_share_fire_census)' --no-capture`

```
  N | nodes | edges | RootJoin tokens | HashJoin tokens | derived
  1 |   6   |   5   |       50        |       50        |   50
  2 |   8   |  10   |      100        |      400        |   50
  4 |  12   |  20   |      200        |     3200        |   50
  8 |  20   |  40   |      400        |    25600        |   50
```

`nodes = 4 + 2N` (optimal sharing). `edges = 5N`. Twenty nodes carrying forty edges at N=8 is the
whole finding: every rule that REUSES a shared node wires its child edge again.

The fire path then multiplies it, because `fire_fixpoint_delta` iterates `node_children(node)` and
does the work once **per edge**:

- `Alpha_A → RootJoin` appears N times → each element seeds N tokens → **RootJoin = M·N**
- `RootJoin → HashJoin` and `Alpha_B → HashJoin` each appear N times, over an already N×-inflated
  left side → **HashJoin = M·N³**

`50·N³` fits every row exactly (50, 400, 3200, 25600). At the size that killed the machine —
N=20, M=500 — the one shared join materialises `500 × 20³ = 4,000,000` tokens, each owning a
`Vec<(Value, i64)>` plus an rpds bindings map. **That is the 4 GiB, accounted for.**

## The root, grounded at `file:line` (not inferred)

- `find-or-mint-alpha` (`wat/rete.wat:427`), `find-or-mint-root-join` (`:457`), and
  `find-or-mint-hash-join` (`:482`) are all **correct**: on a dedup hit they return the existing id
  and touch neither the network nor the children.
- `network-add-child` (`wat/rete.wat:409`) is a bare `PersistentVector/conj` with **no membership
  check** — it appends unconditionally.
- `compile-condition` then wires **once per rule** against those shared nodes: step 3 alpha→join
  (`:742`) and step 4 prev-parent→join (`:752`). N rules ⇒ N identical edges.

## Why no existing guard could see it

Worth keeping, because each blindness is a different shape:

1. **The compile-time probe counted NODES.** A shared node reached by N duplicate edges is
   indistinguishable from one reached once if nodes are all you count.
2. **Accuracy stayed `:match` on every axis.** Duplicate edges produce duplicate *tokens*;
   production dedups through `seen`, so the derived set is correct at every N (the table's `derived`
   column is flat at 50). Correct results, catastrophic cost — invisible to every correctness gate.
3. **The dual-impl differential is structurally blind here.** `compile` is wat, shared by BOTH
   impls; the oracle carries the identical duplicate edges. The anchor holds the two *fire* paths to
   each other and does not cover the compiler upstream of both.

## The one contract decision

**`network-add-child` becomes IDEMPOTENT: adding a child-id already present in the node's children
returns the network UNCHANGED.** Set semantics, through the only door that creates an edge.

Grounded that no legitimate duplicate exists: a rete edge means "propagate to this child", so a
second identical edge can only mean "propagate twice", which no caller wants. The three
never-shared node kinds (TestNode `:562`, ProductionNode `:784`, and the Negation/Exists/Accumulate
nodes) are minted fresh per rule, so their in-edges are unique by construction and unaffected — the
shared join's fan-out to N *distinct* TestNodes is correct and must survive.

**Rejected — guard at the two call sites (`:742`, `:752`).** A stem-cut: the next wiring site added
for a new node kind reintroduces the class. The helper is the root.

**Rejected — type the `children` field as a set.** That is the higher rung (the duplicate becomes
unrepresentable rather than checked), and it is the wrong trade here: a `PersistentVector` is
**ordered**, and fire order follows children order, so an unordered set would put derived-fact
ordering in play — which output-cursor resume depends on (`DESIGN-service-io-budgets.md` CRUX-3).
Idempotent insert into an ordered vector dedups **and** preserves order. Named here so it is not
re-derived as an improvement later.

**No intueri cast is owed.** No name is minted or changed; `add-child` remains honest under set
semantics (adding a present child is a no-op, exactly as set insert reads).

## Blast radius

`wat/rete.wat`, one function body. The `children` field's SHAPE is unchanged, so every reader —
`node-children-ids` (`:311`), the native `node_children`, all five fire passes — is untouched.

## Expected effect

The A8 gate goes GREEN: `edges` collapses to the node count's shape, `RootJoin` and `HashJoin`
tokens go flat in N at M each, `derived` stays 50. Everything else on the floor is unchanged —
**derived facts do not move**, because they were already correct.

The interesting risk is a test whose expected count silently encoded the duplication. That is a
STOP, not an edit (see the brief): an assertion that only passed because of the bug is a finding.

## Out of scope (affirmatively cut, not deferred)

- Re-running the grid axis / re-measuring against Clara. The gate proves the mechanism; the fresh
  A8 numbers are a separate measurement, and it must ride the memory guard in
  `wat-scripts/perf/grid/run-axis.sh` (a workstation died for that lesson).
- The `children`-as-a-set type change (rejected above, with its reason).
- Extending `probe-node-share-dedup.wat` to count edges. The Rust census now counts them and is
  gated; a second edge counter in wat would be a duplicate instrument.
