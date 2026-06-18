# DESIGN — Stone 3b: `HashJoinNode` (the two-sided equality join)

> Arc 278 stone 3, part b — THE HEART. After root-join seeding (3a) puts Tokens in beta-memory, `fire-rules`
> grows a **join pass**: a Token (LEFT) crosses against Elements (RIGHT) at each HashJoinNode; compatible
> pairs produce an EXTENDED token carrying both conditions' bindings + support. This is where partial matches
> across DIFFERENT facts unify — multi-condition rules now match end to end. CLARA-REF §2, hazard #1.

## The join semantics (the crux — get this EXACTLY right)
A `HashJoinNode J` has two inputs (both encoded in the network 1b built, as forward `children` edges):
- **RIGHT** = the Elements of the AlphaNode feeding J (the alpha whose `children` contains J — reverse-lookup).
- **LEFT**  = the Tokens of the upstream beta node feeding J (reached by forward traversal: we arrive at J from
  the beta node B whose `children` contains J).

For each `(token, element)` in `LEFT × RIGHT`:
- **compatible?** — for every `?var` present in BOTH `token.bindings` and `element.bindings`, the values must
  be EQUAL. (Fold `element.bindings`: if a key exists in `token.bindings` with a different value → incompatible.)
  Vars present in only one side never conflict. This dynamic intersection IS the equality join — no precomputed
  `binding-keys` needed (that field stays empty; precompute is a perf optimization, deferred).
- on compatible → **extend the token** (monotonic, CLARA-REF §2):
  - `matches`  = `(conj token.matches (:wat::core::Tuple element.fact alpha-id))` — append the support tuple.
  - `bindings` = `token.bindings` with every `element.bindings` entry merged in (assoc each; the agreed shared
    vars are idempotent, new vars added).
  - store the new Token in `beta-memory[J]`.
- on incompatible → produce nothing.

⚠ **HAZARD #1**: the two memories must be crossed correctly — LEFT=tokens, RIGHT=elements, compatibility by
shared-var agreement. Swap them, or treat a missing var as a conflict, or forget that vars-in-one-side-only are
fine → joins silently drop or duplicate. The probe's no-match case is the canary.

## Traversal (`fire-rules` grows a join pass)
After alpha (2b) + root-join seed (3a), propagate tokens LEFT→RIGHT through the hash-join chain to fixpoint:
- for each beta node `B` (RootJoinNode or HashJoinNode) that has tokens in beta-memory:
  - for each `child-id` in `B.children` where the child is a `HashJoinNode J`:
    - `RIGHT = alpha-memory[alpha-feeding(J)]` (reverse-lookup the alpha whose children contains J).
    - cross `beta-memory[B] × RIGHT`; each compatible pair → extended Token appended to `beta-memory[J]`.
  - the new tokens at J propagate to J's children → repeat until beta-memory stops growing (monotone fixpoint;
    finite tokens → terminates). (For the v1 chains — linear per rule — a topological pass over the join chain
    suffices; a stable-iteration loop is the robust general form.)

`fire-rules` now: alpha pass → root-join seed → **join pass**; production-memory still passes through unchanged.

## What stays / is deferred
- **Flat memories** (`node-id → [Token]` / `[Element]`): kept. The join does a full cross. The `{join-bindings
  → […]}` INDEX (CLARA-REF §5) is a perf optimization to avoid the O(n×m) cross — **deferred** (note it; build
  when a perf probe demands it). Flat is correct.
- **`binding-keys` precompute**: 1b leaves it empty; the join computes the key dynamically. Deferred perf.
- **No 1a/1b record change** — reverse-lookup reads the edges 1b already wired.

## Deps
`PersistentMap/keys`/`get`/`assoc`, `PersistentVector/conj`, `Tuple` ctor, `Token`/`Element` accessors,
`node-kind-label`, `AlphaNode/children`, bare-PV/`foldl`, `values_equal` via `:wat::core::=`. All exist. **If a
core primitive is genuinely missing → STOP + name it; build it, don't hack** (builder directive).

## Proof (FM-2-bis — RED at HEAD) — the cold-and-windy `?loc` join, end to end
`tests/probe_arc278_3b_hash_join.rs` (RED, un-ignore on green): a TWO-condition rule joining on `?loc`:
```
(:Temperature (?loc <- :location) (?t <- :celsius))
(:WindSpeed   (?loc <- :location) (?w <- :kph))
```
compile → AlphaNode(Temp) + RootJoinNode + AlphaNode(Wind) + HashJoinNode + ProductionNode.
- **JOIN (match)**: insert `Temperature{celsius:15, location:"Oslo"}` + `WindSpeed{kph:45, location:"Oslo"}`;
  fire; the HashJoinNode's beta-memory holds exactly **1** Token whose bindings have `?loc="Oslo"`, `?t=15`,
  `?w=45` (both conditions unified) and whose `matches` length is **2** (both supporting facts).
- **NO JOIN (different loc)**: `Temperature{…,"Oslo"}` + `WindSpeed{…,"Bergen"}`; fire; the HashJoinNode's
  beta-memory holds **0** Tokens (the `?loc` keys disagree — the join correctly drops it).
RED at HEAD: `fire-rules` does root-join seeding only (3a) → the HashJoinNode's beta-memory is empty in both cases.

## Four questions
- **Obvious?** YES — a token meets an element; if their shared vars agree, they unify into a longer match.
- **Simple?** YES for v1 — full cross + dynamic intersection; no index, no precompute, no record change. (The
  perf index is the deliberate deferral, not hidden complexity.)
- **Honest?** YES — the equality join is real (the no-match probe proves drops); reads the real edges; monotonic.
- **Good UX?** YES — multi-condition rules now match; beta-memory inspectable; the engine is one step from firing.

## Blast radius
`wat/rete.wat` (`fire-rules` grows a join pass + helpers: `alpha-feeding`, `token-element-compatible?`,
`extend-token`, the cross + propagation) + the probe (un-ignore). NO 1a/1b record change. NO Rust unless a
sub-dep is missing → STOP. No git in the worker.
