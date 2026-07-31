# EXPECTATIONS — Strike 2a: `filter` goes native

Written BEFORE the strike. Brief: `BRIEF-filter-through-the-door.md`.
Supersedes the twin-route version — see the design's "⛔ THE TWIN ROUTE IS DEAD" ruling.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | builds | `cargo build --release --all-targets` | exit 0, zero warnings |
| 2 | the gate is REAL | the new gate, **before** the fix | **FAILS** (~12s against a 1s wall) |
| 3 | **the load-bearing row** | the new gate, after | PASSES (~10–100ms) |
| 4 | laziness survives | `(take 1 (filter …))` over a large source | does not realise the whole source |
| 5 | results unchanged | same elements, same order, **all four seqable heads + the bare-PV case** | identical |
| 6 | `filterv` still works | it is `(into [] (filter …))`, untouched | green |
| 7 | load order | `(:wat::deporder::verify-stdlib)` | `[]` |
| 8 | **the payoff** | `probe-derive-chain-split.wat` at `[50 4000]` | `query-ns` collapses from **12,475ms** |
| 9 | the door is now one | `wat/seq.wat` has no `filter` clauses; no `filter-stream` was minted | 12 twins unchanged, none added |
| 10 | the floor | `cargo nextest run --release` (orchestrator) | only the new gate changes |

**Row 8 is why `filter` is the exemplar.** `query-by-type-string` calls it, so this strike closes
the A8 derive quadratic outright — what this whole day has been chasing — measurable with a probe
already on disk rather than inferred from the unit gate.

**Row 9 is the design ruling made checkable.** If a `filter-stream` appears, the strike built the
superseded thing.

Rows 8 and 10 are mine, not the rider's.

## Independent prediction

**Runtime: 30–45 minutes.** Rust plus checker plus a wat deletion — comparable to Strike 1, with
the advantage that Strike 1 is now a close exemplar for both halves. Predicted mode: one-shot green.

Moderate confidence. The predicate-application half (`apply_function` inside a lazy cell, with
error propagation) is the part with no exact precedent — `eval_seqable_to_stream` has no predicate,
`eval_vec_map` is eager. That seam is where I would expect trouble.

## Trap-doors named in advance

- **A gate that never goes red.** It already happened once on this arc: a `Vector`-sourced wall
  passes pre-fix because a flat clone-and-collect is cheap enough per element not to cross one
  second at n=4000, while a `PersistentVector`'s trie rebuild misses by 35×. Container constant,
  not just complexity class.
- **The bare unparameterised `PersistentVector` arm.** `filter`'s fifth wat clause accepts it. If
  native inference cannot, `filter`'s accepted domain silently narrows — STOP-3, a real checker
  finding, not something to quietly drop.
- **Swallowed predicate errors.** A raising predicate inside a lazy cell must surface. Dropping the
  element instead would be a hidden failure in the arc whose law forbids them, and it would pass
  every row above.
- **Eager rewrite.** Makes rows 3, 5, 8, 10 green while destroying the reason the function is lazy.
  Row 4 is the guard.
- **Clippy under the deny wall** — new `src/` Rust must be warning-clean; a warning is a build
  failure here, not a note.

## What this does NOT claim

Five verbs — `remove`, `take-while`, `drop-while`, `interpose`, `reductions` — stay quadratic until
2b. A still-slow `remove` is not a failed 2a.

Nor does it reach the top of the ladder. After this, "sequence verbs that accept any seqable live in
Rust" is a **convention**; a new wat-level stage could still hand-roll per container tomorrow and be
quadratic and green. The lint that would make that unrepresentable is tracked, not built, and the
type that would make it *unspellable* is `109-kill-std/NOTE-seqable-has-no-name-in-wat.md`.

Nor does it re-measure the Clara grid. Once 2a lands the A8 axis becomes worth re-running — behind
the memory guard, on a size ladder — because for the first time the whole node-share workload, not
just its fire, would be free of the quadratic.
