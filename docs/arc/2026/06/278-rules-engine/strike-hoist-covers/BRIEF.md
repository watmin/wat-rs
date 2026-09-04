# BRIEF — hoist a token-independent lookup out of a per-token loop

One conjunct in the filter's inner loop is a function of `tid` alone and is recomputed for every
token. Hoist it. The correctness proof already exists and must be shown to still bite.

## Read in order

1. `src/rete/kernel/fire/mod.rs:2020-2058` — `use_tree` and the inner loop. Note that `use_tree`
   **already** walks `covers` over `tids`; your precompute replaces that walk.
2. `src/rete/where_tree.rs:177` — `covers` is `self.ids.contains(&id)`. Nothing token-dependent.
3. `src/rete/kernel/tests/where_tree_branch_differential.rs` — **the correctness proof.** It compares
   the tree branch against the reference branch over 115 fixtures. Read its two mutation arms; they
   are what must still fire.
4. `src/rete/kernel/tests/node_share_cost.rs`, arms **G–L** — the instrument for before/after.
   `J−I` is the tid loop. ⚠ Its header explains why the small rungs are below resolution and why
   negative deltas are printed rather than hidden — **do not pin a rung**.

## The change

```rust
let covered: Vec<bool> = tids.iter().map(|id| sink.where_tree.covers(*id)).collect();
let use_tree = covered.iter().any(|c| *c);
...
        for (i, &tid) in tids.iter().enumerate() {
            if covered[i] && !proven.contains(&tid) && !maybe.contains(&tid) {
```

That is the whole edit. **No ordering change, no set intersection, no new concept.**

## Driven by the orchestrator at HEAD `5f0b2f1b1`

`node-share [50 200]`: the inner loop runs **10,000 (token, tid) pairs × 3 hash lookups**;
**10,000 of those are `covers`** and token-independent. `J−I` ≈ **290 µs** of a ~**414 µs** phase.

## STOP triggers

1. **⛔ If the branch-pair differential REDs, stop and report** — you have changed filter semantics,
   which this edit must not do.
2. **If the measured gain is inside the arms' noise**, stop and report the numbers. **A change that
   cannot be measured is not landed on faith** — report it and let the orchestrator decide.
3. **If `covers` turns out not to be token-independent** (some interior mutability, some `&mut`
   path), stop — the premise is wrong and the whole strike dies.
4. **If this needs any change beyond `dispatch_where_tests`**, stop and report.

## Mutation proofs — run all three, report all three

1. ★ **Prove the differential still bites on the NEW code.** Break the hoisted version the way
   mutation 1 broke the old one (drop `&& !maybe.contains(&tid)`) → the differential must RED with
   **dropped** facts. ⛔ **Landing an optimisation without re-proving its gate is shipping under a
   green light nobody re-earned.**
2. **Invert the hoist** — build `covered` from `!covers(tid)` → the differential REDs. Proves the
   precomputed vector is actually the value being read, not a coincidence.
3. **Desynchronise the index** — index `covered` with a fixed `0` instead of `i` → the differential
   REDs on a fixture where tids differ in coverage. ⛔ **If it does NOT red, the corpus has no
   mixed-coverage fixture and this mutation is unprovable — say so**; that is a corpus finding, not
   a pass.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The before/after on arms G–L, **six samples each side**, with spread. Six or no number.
- All three mutation results, and for mutation 3 whether a mixed-coverage fixture exists.
- The differential's result on the new code.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Eight consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **six were the orchestrator's own artifacts**, the
  last a "zero callers" claim asserted in bold three times that one grep refuted. Assume there is a
  ninth.

Do not commit.
