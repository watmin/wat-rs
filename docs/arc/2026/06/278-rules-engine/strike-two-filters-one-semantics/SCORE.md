# SCORE — the fast filter is SOUND, and my coverage criterion could not have proved it

> **Written after the orchestrator's own weighing.** The two ★ corrections were each one grep away.

## The result that mattered

**GREEN at HEAD.** The two branches of `dispatch_where_tests` derive **identical fact multisets**
across **115 tree-firing fixtures · 34,368 (tid, token) pairs · 9,576 derived facts**. Both soundness
obligations hold today:

| obligation | if wrong | status |
|---|---|---|
| `covers && !proven && !maybe` ⟹ test is false | **drops** a derived fact | ✅ holds, 13,982 skips exercised |
| `proven && is_pure_cmp` ⟹ test is true | **invents** one | ✅ holds, 2,441 reuses exercised |

**And now a gate would notice if either stopped holding** — which is the part D7 didn't have.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 ★ | the branch pair has a differential | ✅ `where_tree_branch_differential.rs`, one test, `PASS [10.3 s]` |
| 2 ★ | obligation 1 guarded | ✅ mutation 1 → **`DROPPED: 14 facts`, `INVENTED: []`** |
| 3 ★ | obligation 2 guarded | ✅ mutation 2 → **`INVENTED: 175 facts`, `DROPPED: []`** — disjoint from mutation 1 |
| 4 ★ | corpus measured to express the defect | ⚠ **my criterion was wrong** — see below. Corpus discovered by parsing, 26 exclusions pinned by exact set equality |
| 5 | sets, never counts | ✅ multisets, with symmetric difference printed |
| 6 | not vacuous | ✅ mutation 3 → *"0 FIRE"*, fails |
| 7 | engine unchanged | ✅ **`fire/mod.rs` md5 identical to HEAD**; zero `src/` change, not even visibility |
| 8 | floor / lints / clippy | ✅ **`5408 tests run: 5408 passed, 21 skipped`** (440.0 s), 0 FAIL, lints **258**, clippy rc=0 |

## ⛔⛔ THE EIGHTH FALSE CLAIM — asserted in bold three times, one grep away

The DESIGN, the BRIEF **and the rider's own prompt** all state:

> *"`WhereTree::empty()` … `pub(crate)` with **zero callers in `src/`**"*

**False.** `where_tree.rs:143` calls `Self::empty()` — it is `build`'s own empty-input
short-circuit, **documented six lines above it**:

> *"An empty input short-circuits to `empty()`: a tree over no `where`s would answer every query with
> an empty candidate set, which is correct but pays a walk to say so."*

The follow-on in the DESIGN — *"Deleting `WhereTree::empty()` as dead code. `purgare` would flag
it"* — is false for the same reason. Nothing in the strike depended on it, but it is exactly the
shape the brief warns riders about: **a bold, thrice-repeated, one-grep-checkable claim that nobody
grepped.** Repetition is not verification.

## ⛔⛔ AND MY COVERAGE CRITERION MEASURED ONE OBLIGATION OF TWO

Stated three times — DESIGN, BRIEF, EXPECTATIONS row 4 — as *the* test for whether a fixture
exercises the branch pair:

> *"a fixture exercises this branch pair only if `filter:test-reuse > 0`"*

**`reuse` is obligation 2's counter. Obligation 1's arm (`:2036`) is a bare `continue` and emits no
census counter at all**, so it is invisible to that criterion. Measured: **5 fixtures reach obligation
1 with `reuse == 0`** — 528 skipped pairs — and `reuse > 0` selects only **34 of the 115** firing
fixtures.

**My criterion's selection principle is orthogonal to the arm that drops facts.** Taken literally it
would have gated over a third of the corpus and dropped some of the only obligation-1-bearing
fixtures — the C9 corpus hole, rebuilt by the brief that cited C9 as the reason to avoid it.

The rider's criterion, argued rather than assumed: `wheres > 0 && reference.evals > 0`, sound because
a dispatched tid always has a compiled `where` and `covers` is `ids.contains` over exactly those
keys, so **any dispatch implies `use_tree` was true**.

It also recovered the invisible skip count without touching the engine —
`skips = ref_evals − tree_evals − tree_reuse` — and **anchored it on a number it did not produce**:
node-share `[50 200]` yields **9,800**, matching `node_share_cost.rs`'s independently-recorded figure
from C12.

## ★ A finding neither artifact anticipated: the tree's ROUTE is nondeterministic, its ANSWER is not

`WhereTree::build` collects its dims from a `std::collections::HashSet` (`where_tree.rs:29` imports
**std**, not Fx), so the discrimination tree's level order is **randomised per process**.

Measured over five runs: `where-join-order row 6` flips between `evals 14 / reuse 29` and
`evals 29 / reuse 14`; corpus reuse totals came out **2426 / 2441 / 2456**. **Derived multisets, the
13,982 skip count and the 34,368 pair count never moved.**

**Any gate pinning a reuse or evals number here would be a flake by construction.** Every assertion in
the new file is a set equality or a `> 0` reach, and the header says why so a future hand does not pin
one. This is C20's family — `HashMap`-ordered traversal — with a different consequence: not a wrong
answer, a different route to the right one.

## Mutations — four, each on a named arm, two of them disjoint

| # | mutation | arm | result |
|---|---|---|---|
| 1 | drop `&& !maybe.contains(&tid)` | obligation 1 | RED, `where-join-order row 5`: **14 dropped**, 0 invented |
| 2 | drop `&& is_pure_cmp(tid)` | obligation 2 | RED, `where-boolean row 1`: **175 invented**, 0 dropped |
| 3 | corpus → non-firing only | vacuity | RED: *"NO FIXTURE FIRES THE TREE BRANCH… 0 FIRE"* |
| 4 *(rider's own)* | reference run fires the **tree** arm | self-comparison | RED: *"the empty-tree arm did not take"* |

**Mutation 4 was not in my brief.** It is the `X == X` guard — proof the reference run actually took
the reference branch. Given that this arc found a landed gate comparing a session against itself two
days ago, its absence from my mutation list was a real omission.

## Honest deltas

- **"A new gate under `tests/rete/`" is unreachable.** That is an integration-test crate and cannot
  see `pub(crate)`. The gate lives in the in-crate `src/rete/kernel/tests/` module. My BRIEF
  anticipated a *visibility widening*; the right answer was **none at all, in a different directory**.
- **Zero `src/` change** — better than the blast radius allowed for. `fire_fixpoint_delta_armed`
  already takes an optional pre-built arm (stratify uses it for slice arms), so the same staged
  session fires twice with no new seam.
- **Observation, not a disposition:** across the rider's two floor runs a `SLOW [> 15 s]` marker
  appeared on `reachability_shard_0_of_6` in one and not the other; both runs were 5408/0. My own run
  showed no SLOW row. Recorded, not characterised.
