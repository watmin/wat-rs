# DESIGN — two implementations of one filter semantics, chosen by a heuristic, never compared

## Why

Opened while answering the builder's question about a perf gain in the filter phase. The builder's
reframe is what found it:

> *"let's make the alg more correct … correctness above all"*

The perf question (a 10,000-iteration scan where ~1 iteration matters) is real and **secondary**.
Underneath it is a soundness question nothing in the tree asks.

## The two branches

`fire/mod.rs:2020` — `dispatch_where_tests` picks between two implementations:

```rust
let use_tree = tids.iter().any(|id| sink.where_tree.covers(*id));
```

**One covered tid flips the entire dispatch.**

| branch | behaviour |
|---|---|
| `else` (`:2060-2083`) | `exec_stashed_where` for **every** (tid, token). This is the DEFINITION of the filter. |
| `if use_tree` (`:2021-2058`) | **skips** on `covers && !proven && !maybe`; **pushes without evaluating** on `proven && is_pure_cmp`; evaluates otherwise |

The tree branch is an optimisation of the else branch. It must produce an identical fact set. **That
is asserted nowhere.**

## The two unchecked proof obligations

1. `covers(tid) && !proven && !maybe` ⟹ `exec_stashed_where(tid, binds) == false`
   — a wrong answer here **silently drops a derived fact.**
2. `proven && is_pure_cmp(tid)` ⟹ `exec_stashed_where(tid, binds) == true`
   — a wrong answer here **silently invents one.**

These are the where-tree's soundness claims, discharged by construction and by nothing else. **This is
D7's shape on a different mechanism**: a fast path and a reference path, one of them taken, no
differential. D7 cost this arc a day and was found by hand.

## The differential is already buildable — the affordance exists, unused

`WhereTree::empty()` (`where_tree.rs:117`) is `pub(crate)`. ⛔ **THIS LINE SAID "zero callers in `src/`" AND IT WAS FALSE** — `where_tree.rs:143` calls it as `build`'s own empty-input short-circuit, documented six lines above it. Asserted in bold three times across these artifacts and never grepped; repetition is not verification. It is a live constructor, and the lever works either way. An empty
tree makes `covers` false for every tid, so `use_tree` is false and the dispatch takes the reference
branch. **Same session, same facts, two dispatch strategies, comparable outputs.**

That is exactly C9's port-check shape — `native` vs `oracle` — applied one layer down.

## The contract decision, pinned

**A differential over the branch pair, comparing FACT SETS, on a corpus measured to exercise the
tree path.**

- Fire a session normally, then re-fire the identical staged session with `where_tree` replaced by
  `WhereTree::empty()`, and compare the **derived fact sets** — not counts. **D7 produced a
  right-sized wrong answer**; a cardinality check would have passed it.
- **The corpus must be measured, not assumed.** A fixture only exercises the tree branch if
  `filter:test-reuse > 0`. Fixtures where the tree never fires prove nothing and must not be counted
  as coverage — that is C9's corpus hole, which had a shape exactly like the bug it missed.
- **Non-vacuity is a gate requirement**: an empty derived set compares equal to an empty derived set.

## Out of scope = REJECTED

- **The scan optimisation.** `(proven ∪ maybe) ∪ uncovered` instead of all tids is a real
  O(tokens×tids) → O(tokens×k) win, and `covers(tid)` is token-independent so the uncovered subset
  hoists out of the loop. **It is a separate strike and it must land AFTER this one** — an
  optimisation to a path with no differential is a change nobody can prove safe.
- ~~**Deleting `WhereTree::empty()` as dead code.**~~ ⛔ STRUCK — it was never dead; `build` calls it.
- **The import door.** `export.rs:2451` builds a second tree. Whether the two doors agree is Class A's
  question and is not this strike's.
