# DESIGN — Stone 3a: `RootJoinNode` seeding (the first beta slice)

> Arc 278 stone 3, part a — the FIRST beta slice. After the alpha pass (2b) populates alpha-memory with
> Elements, `fire-rules` grows a **root-join pass**: each Element flowing from a first-condition AlphaNode into
> its `RootJoinNode` is wrapped into a fresh **Token** and stored in beta-memory. No two-sided join yet
> (HashJoinNode = 3b); no production (stone 4). This is the LEFT-side seed the hash-join will cross against,
> and the first real consumer of the alpha→join edges `compile` (1b) wired.

## Background (CLARA-REF §2)
A `Token [matches bindings]` flows LEFT→RIGHT through beta nodes. It is **born at RootJoinNode/right-activate**
as `(->Token [[fact node-id]] alpha-bindings)` (clara engine.cljc:584): the RootJoinNode has no left input —
it just lifts each incoming Element into a Token (matches = the one-entry support chain; bindings = the
Element's alpha-bindings). `RootJoinNode.binding-keys = []` (first condition, no join key yet).

## Contract — `fire-rules` grows a root-join pass
After the alpha pass, for each `AlphaNode` that has Elements in alpha-memory, follow its `children` edges; for
each child that IS a `RootJoinNode`, seed one Token per Element:
- `Token.matches`  = `(:wat::core::PersistentVector (:wat::core::Tuple fact alpha-id))` — a one-entry support
  chain of **tuples**: `[(fact, alpha-id)]`. The entry is a TUPLE `(:wat::Record, :wat::core::i64)`, **NOT a
  vec** — the pair is heterogeneous (a Record + an i64), which a homogeneous/bare vec cannot honestly type.
  (Load-bearing for TM later; the chain grows one tuple per condition as the token flows through joins.)
- `Token.bindings` = the Element's `bindings` (carried straight through — root-join adds no new bindings).
- store into `beta-memory[root-join-id]`.

## ONE decision — `Token.matches` is `PersistentVector<(:wat::Record, :wat::core::i64)>` (1a refinement)
1a left `Token.matches` as a bare `PersistentVector` with a comment "`[[fact node-id]]`". Refine it to a
PV of **tuples** `(:wat::Record, :wat::core::i64)` — the support entry is a fixed positional pair of disparate
types, and a tuple is the right tool (uses existing `:(T,U)` / `(:wat::core::Tuple a b)` support, as 1b's
find-or-mint already does; a record would over-structure a simple support pair). Update the `Token` record in
`rete.wat`. Four-questioned + builder-approved (2026-06-19).

`fire-rules` now writes alpha-memory (2b) THEN beta-memory (3a); production-memory still passes through unchanged.

## beta-memory shape (this stone)
`beta-memory : PersistentMap<node-id (i64), PersistentVector<Token>>` — flat, mirroring 2b's alpha-memory.
**Stone 3b refines** to `node-id → {join-bindings → [Token]}` (the hash-join's index keying, CLARA-REF §5) —
that sub-key is the HashJoinNode's mechanism; a RootJoinNode (binding-keys = []) has nothing to key on, so flat
is the honest 3a shape (not a build-around; 3b introduces keying where it's actually used).

## Algorithm (pure WAT, extends `fire-rules`)
1. Run the alpha pass (2b) → alpha-memory.
2. Root-join pass — fold the network's AlphaNode ids; for each with Elements `els = alpha-memory[alpha-id]`:
   - for each `child-id` in `(AlphaNode/children node)` where `(node-kind-label child) == "RootJoinNode"`:
     - fold `els`: for each `el`, build `Token([[fact alpha-id]], (Element/bindings el))` and append to
       `beta-memory[child-id]` (create the PV if absent).
3. return `Session/assoc :beta-memory <built map>` (alpha-memory from step 1 preserved).

Deps: all exist — `Element/fact`/`Element/bindings`, `AlphaNode/children`, `node-kind-label`, the
PersistentMap/PersistentVector ops, `Token` ctor, bare-PV foldl (0d.1). **If a sub-dep is missing → STOP +
name it; build it, don't hack** (builder directive).

## Proof (FM-2-bis — RED at HEAD)
`tests/probe_arc278_3a_root_join.rs` (RED, un-ignore on green): compile a ONE-condition rule
`(:user::Temp (?t <- :value) (:wat::core::> ?t 20))` (→ 1 AlphaNode + 1 RootJoinNode + 1 ProductionNode);
`insert` a matching `(:user::Temp 25)`; `fire-rules`; inspect `Session/beta-memory`:
- exactly **1** node populated (`length (keys beta-memory)` == 1 — the RootJoinNode),
- it holds exactly **1** Token,
- that Token's `bindings` has `"?t"` == `25` (alpha-bindings carried into the seeded Token),
- that Token's `matches` has length **1** (the one-entry support chain).
RED at HEAD: `fire-rules` is alpha-only (2b) → beta-memory is empty → `length (keys beta-memory)` == 0, not 1.

## Out of scope (affirmative cuts)
- `HashJoinNode` / the two-sided equality join (left+right activate, the join-bindings cross) → stone 3b.
- `join-bindings` sub-key on alpha-memory + beta-memory → stone 3b (where the index is used).
- Production firing / RHS / cascade / TM → stone 4.

## Four questions
- **Obvious?** YES — a first-condition Element becomes a Token (the support seed).
- **Simple?** YES — a fold over alpha→root-join edges; flat beta-memory; no join cross yet.
- **Honest?** YES — seeds exactly the LEFT side; no faked hash-join; uses the real edges 1b wired.
- **Good UX?** YES — beta-memory is inspectable; the engine grows one honest slice.

## Blast radius
`wat/rete.wat` (`Token.matches` → `PersistentVector<(:wat::Record, :wat::core::i64)>`; `fire-rules` grows a
root-join pass + helpers) + the probe (un-ignore). NO Rust (deps exist) unless a sub-dep is missing → STOP.
No git in the worker.
