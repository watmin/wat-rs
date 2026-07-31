# BRIEF — cache the gather index per (alpha, join-keys) within a round

## The work

`gather_index` is a pure function of `(alpha memory, join keys)`, but it is rebuilt **per node, per
round**, and each rebuild drags a full clone of the alpha memory with it. At `G=200 W=200` that is
**five builds over two distinct `(alpha_id, join_keys)` pairs** — three are pure repetition. Add a
round-scoped cache, shared by the accumulate pass and the Negation/Exists filter pass, so the first
reader of a pair builds and the rest borrow. A RED gate is in the tree and fails at 5 builds /
200,000 elements; your job is to bring it to 2 / 80,000 without moving any other number.

## Read in order (the rooms, and why)

1. **`src/rete/kernel.rs:1228` — `gather_index`.** The function being cached. Note its return:
   `(Vec<Value> join_keys, HashMap<Vec<Value>, Vec<usize>>)` — the buckets hold **indices into the
   elements slice**, which is why the snapshot and the index must be cached and borrowed *together*.

2. **`src/rete/kernel.rs:1821-1824` — the round loop head.** Where `d_alpha`/`d_beta` are created
   fresh each round. The cache belongs here, with the same lifetime, for the reason in the contract
   below.

3. **`src/rete/kernel.rs:~2185-2195` — the accumulate pass's build.** `from_elements` is cloned from
   `wm.alpha`, then `gather_index` runs. Four of the five builds are here (count, sum, min, max).
   The `census_count`/`census_count_n` calls beside it are the gate's instrument — keep them
   reporting real builds.

4. **`src/rete/kernel.rs:~2275-2285` — the filter pass's build** (Negation/Exists). The fifth build,
   over the same Reading-`?g` alpha that `count` already indexed. Same treatment, same cache.

5. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-gather-index-cache.md`** — the measurement that
   sizes this, the contract, and the affirmative scope cuts.

## ★ THE ONE CONTRACT DECISION (this is the whole stone)

**The cache key is `(alpha_id, join_keys)`. `alpha_id` alone is NOT sufficient.**

`join_keys` is the intersection of the *sample token's* binding keys with the *element's*. Two nodes
reading the same alpha can have parents that bind different variable sets, and would then need
different key tuples. Key on `alpha_id` alone and a node receives an index bucketed by the wrong
tuple: every probe misses, every gather returns empty, and `count`/`sum` quietly emit their identity
for groups that **do** have elements. That is a wrong answer, not a crash — **and the gate will not
catch it**, because in this workload every reader happens to key on `?g`. The differentials and this
clause are what stand between the cache and a silent empty gather.

Two supporting constraints, both silent-wrong-answer risks:

- **Round-scoped, never longer.** `wm.alpha` grows during step 1 of every round. A cache that
  outlived a round serves a stale index missing that round's new elements. Create it inside the
  round loop, beside `d_alpha`/`d_beta`.
- **The snapshot travels WITH its index.** Buckets are indices into one specific `Vec<Value>`.
  Caching the index but re-deriving the elements re-introduces the clone this stone removes — and
  pairs an index with a vector it does not describe.

## Implementation sketch (the shape; fill it in)

Inside the round loop, beside `d_alpha`:

```rust
// (alpha_id, join_keys) -> (snapshot, index). Round-scoped: wm.alpha grows in step 1.
let mut gather_cache: HashMap<(i64, Vec<Value>), (Vec<Value>, HashMap<Vec<Value>, Vec<usize>>)>
    = HashMap::new();
```

At each of the two build sites, the shape is: derive `join_keys` for this node, look up
`(alpha_id, join_keys)`, and on a miss clone + build + insert. The existing `census_count(
"accum:index-builds")` / `census_count_n("accum:index-elements", ..)` must fire **only on a real
build (the miss path)** — that is exactly what the gate reads.

Note the ordering wrinkle: today `join_keys` comes *out of* `gather_index`, but the cache needs it
*before* the lookup. Deriving the key-set is the cheap half (a sample intersection + sort); building
the index over N elements is the expensive half. Split them so the key can be computed first.

## Blast radius

`src/rete/kernel.rs` only. No `.wat` changes, no corpus migration — the wat oracle does not move; it
is the reference this is checked against. Do not touch `token_element_compatible`, `accumulate_value`,
or the fold.

## STOP triggers (each is a rejection: ship nothing for it, report the gap)

1. **STOP-1** — if `join_keys` cannot be derived without building the index (i.e. the split in the
   sketch is not possible as written), STOP and report what blocks it. Do not cache on `alpha_id`
   alone as a workaround; that is the one thing this stone must not do.
2. **STOP-2** — if any existing rete differential goes red, STOP and report the test name and the
   diff. The oracle is the anchor.
3. **STOP-3** — if making the gate green appears to need the cache to outlive a round, STOP and
   report. Round-scoped is a correctness constraint, not a convenience.

## Definition of done

- **Remove the `#[ignore]`** on `gather_index_is_built_once_per_alpha_and_keyset` — it is marked
  RED-by-design only so `main` stays green while this is in flight, and a gate that ships ignored and
  stays ignored proves nothing.
- `cargo nextest run --release -E 'test(gather_index_is_built_once)'` passes (2 builds / 80,000).
- `cargo nextest run --release -E 'binary_id(wat::rete)'` passes.
- `cargo nextest run --release` — the **whole** floor, Summary line, 0 failed.
- `cargo clippy --all-targets --release` emits nothing.
- Report the gate's printed builds/elements and `git diff --stat`.

Leave the tree dirty and uncommitted; the orchestrator weighs by its own re-run and commits.

## A prior result to copy for shape

`8654b4c7` (the keyed gather) is the same file, the same passes, and the same discipline: a census
gate rather than a wall, contract clauses that name the silent-wrong-answer risks, and the wat oracle
left untouched as the anchor.
