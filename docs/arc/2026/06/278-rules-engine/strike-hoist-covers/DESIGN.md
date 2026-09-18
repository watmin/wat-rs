# DESIGN — `covers(tid)` does not depend on the token, and is recomputed once per token anyway

## Why

The filter phase's inner loop runs **three hash lookups per (token, tid) pair**:

```rust
if sink.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
```

`covers(tid)` is `self.ids.contains(&id)` (`where_tree.rs:177`) — **a function of `tid` alone.** It is
recomputed for every token. On `node-share [50 200]` that is **10,000 of the loop's 30,000 lookups,
spent re-deriving a token-independent answer.**

## Why this and not the larger rewrite

The obvious bigger move — iterate `(proven ∪ maybe) ∩ tids ∪ uncovered` instead of scanning all tids
— was **reasoned and rejected on the four questions**:

- **Obvious?** NO. A reader must reconstruct why iteration order stays safe.
- **Simple?** NO. It braids candidate selection, intersection with `tids` (candidates can name tids
  outside this sibling group), the hoisted uncovered set, **and order preservation** — because
  `cands.proven`/`maybe` originate in a `std::collections::HashSet` whose iteration order is
  **randomised per process** (`where_tree.rs:29`, measured by the branch-pair differential).
  Iterating them directly makes `exec_stashed_where`'s `?` error-propagation order nondeterministic
  — **C20's shape, closed this week, reintroduced.** Fixing that needs a position-sorted merge: a
  fifth concept.

**A is a strict subset of that rewrite.** If it lands and the remaining 20,000 lookups still
dominate, the larger question gets asked with numbers behind it — including whether its ordering
complexity is worth buying.

## The change

```rust
let covered: Vec<bool> = tids.iter().map(|id| sink.where_tree.covers(*id)).collect();
let use_tree = covered.iter().any(|c| *c);
...
        for (i, &tid) in tids.iter().enumerate() {
            if covered[i] && !proven.contains(&tid) && !maybe.contains(&tid) {
```

`use_tree` **already** walks `covers` over `tids` — the precompute replaces that walk rather than
adding one. It forfeits `any`'s short-circuit (at most `|tids|` extra lookups, once) to remove
`|tids| × |tokens|`.

**No ordering change. No set intersection. No new concept.** The loop still visits every tid in
`tids` order; only the source of the first conjunct moves.

## The contract decision, pinned

**The correctness proof is the branch-pair differential, and it must be shown to bite.**

`where_tree_branch_differential` (`5f0b2f1b1`) compares the tree branch against the reference branch
over 115 fixtures / 34,368 pairs / 9,576 derived facts. **A change to the tree branch is exactly what
it exists to police.** Landing this without demonstrating the differential can still fail on a broken
version of the new code would be shipping an optimisation under a gate nobody re-proved.

**The before/after is measured on C12's arms, not the grid.** C8 measured the grid's resolution at
~20%; this change cannot clear that bar, and the arms can resolve it. `J−I` is **~290 µs of a
~414 µs phase** at HEAD.

## Out of scope = REJECTED

- **The candidate-iteration rewrite (B).** Reasoned and rejected above; revisit with A's numbers.
- **Any other phase.** `alpha`, `hash-join` and `accumulate` are not this strike's.
- **Claiming a wall-clock win.** Fire is **0.18%** of wall on this axis. This is a fire-time and
  ratio change; saying otherwise would be the unfalsifiable-perf-claim defect C8 was opened for.
