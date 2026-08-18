# DESIGN STONE — 118.B6 · a NATIVE `foldl` over `Seqable<T>`, with the old one kept as a correctness oracle

**Builder, 2026-08-18:** *"that sounds like a proper good feature to have?... we can move foldl to
foldl' and then reclaim foldl for the native... foldl' becomes a correctness oracle... it must always
agree with foldl."*

Ruled G (below). Drawn after B2c/B2d closed both `Seqable<T>` doors and `reductions` collapsed.

## The gap — and the code already says it is owed

`foldl` is a native intrinsic (`src/collection/transform.rs:474`, dispatched `src/runtime.rs:6354`).
It walks Vector / PersistentVector / List **directly by iterator** and **refuses `Stream`** — gated
by `StreamContainer::mappable()`, whose own comment names the gap and names this arc:

```rust
// Arc 118 — Stream: HOFs (map/filter/etc.) are a later strike. ○ gap.
StreamContainer::Stream => false,
```

**Measured consequence** (`wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat`, 200k i64
sum, both block orderings, non-vacuity held):

```
native foldl      103 / 107 / 111 / 93 ms     0.52 us/elem
interpreted walk  503 / 574 / 541 / 503 ms    2.65 us/elem      ★ 5.1x
```

That 5.1× is why `reduce` could NOT collapse to one arm per arity when `reductions` did: its eager
arms delegate to native `foldl`, and routing them through the interpreted `reduce-walk` would tax
every eager reduce in the language. **A native `foldl` over `Seqable<T>` removes the trade
entirely** — `reduce` collapses AND stays fast, and `reduce-walk` dies.

## What it does

1. **Widen `foldl` to any seqable, natively.** `value_as_stream` (`src/stream/mod.rs:244`) plus
   `realize` already do native seqable walking — this is the same machinery `:wat::core::drop` uses
   (`eval_vec_drop`). No new traversal primitive is invented.
2. **Retain today's eager-only implementation under a second name**, unchanged.
3. **Bind them with a differential test**: the two must agree on every eager input, always.

★ **NO CALL-SITE MIGRATION.** The widened `foldl` accepts a strict SUPERSET of today's domain, so all
**551** existing `foldl` call sites keep their spelling and their answers. The "move" is of the
IMPLEMENTATION to a second name, not of the callers. This is a zero-churn change, and any brief that
proposes a 551-site codemod has misread the stone.

## ⛔ THE ONE CONTRACT DECISION — the second name, and it is NOT free

The repo already has this exact pattern, and its convention runs the OTHER way
(`src/runtime.rs:5563`):

> *"rete dual-impl: **unprimed is the wat ORACLE, primed the native kernel**; never collapsed"*

There, the pair is **wat oracle vs native kernel** — two different LANGUAGES, and the prime marks the
native one. Here BOTH implementations are native; the pair is **narrow-and-old vs wide-and-new**. So
`foldl'` would make the prime mean a third thing, and a reader arriving from `insert-all'` would read
it backwards.

Three spellings, and this is the builder's to rule:

- **(a) `:wat::core::foldl'`** — the builder's sketch. Shortest, but inverts the rete meaning of the
  prime and puts two conflicting conventions in one substrate.
- **(b) `:wat::core::foldl-eager`** — says what it IS. No convention collision. Longer, and it is a
  new naming shape for "retained reference implementation".
- **(c) invert to match rete** — make the ORACLE unprimed and the shipping verb primed. **Rejected
  here on sight**: `foldl` is the public name at 551 sites; the shipping verb must keep it.

⚠ Whatever is chosen, the choice and its relationship to the rete convention goes in a comment at
BOTH sites, or the next reader re-derives this. `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

## The four questions

**G — widen `foldl` natively; retain the eager impl under a second name; differential-test them.**
*Obvious? YES* — "foldl handles any seqable; the second name is the old eager path, kept so we can
prove they agree." *Simple? YES* — one widened fn, one retained binding, one test; the traversal
machinery exists. *Honest? YES*, provided the retained name's zero-non-test callers are stated as a
DISPOSITION and not left to look like rot (task #48: an oracle's only legitimate caller is its test).
*Good UX? YES* — strictly more accepting for every caller, and it unblocks `reduce`. ★ **RULED.**

**H — widen `foldl` and keep the eager fast path INSIDE it; no second name.**
*Obvious? YES. Simple? YES* — one fn, an internal branch, no new surface name.
*Honest? **NO*** — with one implementation **there is no oracle**; a function cannot check itself.
This is the option a reasonable engineer reaches for and it discards the whole point.
`NISI FRANGAS, NIHIL PROBAS`. **Named as rejected because it is what a rider will drift into.**

**I — status quo.** *Honest? NO* — the gap is flagged in the substrate as owed by this arc, `reduce`
stays uncollapsable, and `reduce-walk` remains an interpreted duplicate of a native capability.

## Rooms

| file:line | what |
|---|---|
| `src/collection/transform.rs:474` | `eval_vec_foldl` — the impl to RETAIN under the second name, and the one to widen |
| `src/stream/mod.rs:244` | `value_as_stream` — the native seqable walker to use; `eval_vec_drop` is the worked caller |
| `src/collection/seq_container.rs:214` | `mappable()` — ⛔ **DO NOT widen globally** (STOP-2) |
| `src/runtime.rs:6354` | the `:wat::core::foldl` dispatch arm; the second name needs its own |
| `src/check.rs:2383, 4382, 19326` | `infer_foldl` wiring + scheme registration; the second name needs its own scheme |
| `wat/seq.wat` `reduce` | the PAYOFF — out of scope here, see the cut below |

## ACCEPTANCE

| # | assertion |
|---|---|
| 1 | ★★ **the differential oracle**: `foldl` and the retained impl agree on MANY shapes — Vector / PersistentVector / List, lengths 0 / 1 / 2 / large, and a non-associative `f` so ORDER is proven, not just the sum |
| 2 | `foldl` over a `Stream` now works at all (it is a hard error today) |
| 3 | ★ **no eager regression**: widened `foldl` vs the retained impl on 200k, both block orderings, non-vacuity held. The existing bench gains a third arm |
| 4 | all **551** existing call sites unchanged — `git diff` touches no `.wat` outside the stone's own tests |
| 5 | floor ≥ 4740 passed / 0 failed / 19 skipped · clippy 0 |

## ⚠ STOP triggers

- **STOP-1 — the oracle DISAGREES on any input.** Report the input and both answers verbatim. Do not
  "fix" either side to make them match; a disagreement is the stone's whole reason to exist.
- **STOP-2 — `mappable()` would be widened globally.** It gates `Stream` out of `map`/`filter`/
  `foldr` too. Widening it ripples across the entire HOF family in one commit. This stone touches
  `foldl`'s path ONLY; if that cannot be done without the global gate, STOP and report.
- **STOP-3 — eager `foldl` regresses measurably** (row 3). That is a real finding, not a rounding
  error: report the numbers and stop.
- **STOP-4 — the floor goes red for any reason other than a line-number shift in a pinned golden**,
  or skipped moves off 19.

## Out of scope — affirmative cuts

- **Collapsing `reduce`** — the payoff, and a `wat/seq.wat`-only follow-up ONCE row 3 proves there is
  no eager regression. Doing both in one commit would make a perf regression and a surface change
  indistinguishable in the diff.
- **Deleting `reduce-walk`** — falls out of that same follow-up, not this stone.
- **`map` / `filter` / `foldr` over `Stream`** — the rest of the `mappable()` gap. Same class, named,
  and deliberately not swept in: one verb, one stone, one measurement.
- **B3** (delete both memos) — independent; its precondition has been met since B2b.
