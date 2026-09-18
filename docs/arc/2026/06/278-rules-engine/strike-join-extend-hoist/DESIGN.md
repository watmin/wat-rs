# DESIGN — three SipHash lookups per emitted pair, on a key fixed for the whole node

> Drawn 2026-09-06 at HEAD `91f6b3609`. Source: vigilia 2026-09-05 `temperare` §1.
> **Every line verified on disk at THIS HEAD** (the file has moved under A1/D3/A3/A8).

## The site

`join_extend` (`fire/mod.rs:686`) is the innermost function of the engine — once per emitted
`(token, element)` pair. It performs **three `HashMap<i64, _>` lookups, all keyed on `alpha_id`**:

| line | lookup |
|---|---|
| `rematch_compiled(ctx.compiled_conds, alpha_id)` | `compiled_conds.get(&alpha_id)` |
| `span_from_row(…)` prologue | `bind_only.get(&alpha_id)` |
| same prologue | `cond_key_ids.get(&alpha_id)` |

`alpha_id` is a **parameter**, fixed for both enclosing loops at all **five** call sites —
`hash_join.rs:239`, `:439`, `:504`, `fire/mod.rs:771`, `:874` — each of the shape
`for tok in … { for el in bucket { join_extend(tok, el, alpha_id, ctx) } }`.

## Aggravating, and unstated

`AlphaMemory` is `FxHashMap` (`session.rs:154`). Its siblings on this path are **std `HashMap`** —
`CondKeyIds` (`:173`), `BindOnlyFields` (`:176`), `BetaMemory` (`:155`) — i.e. **SipHash-1-3 on an
`i64`, three times per pair**. `rustc_hash::FxHashMap` is already imported in that same file
(`session.rs:6`). Nothing in the tree states this as a decision.

## ★ Why nothing has ever seen it

`temperare`'s own L3: *"every counter measures occurrences of an operation the design names; none
measures **lookups performed**, so §1–§5 are invisible to the 265 gates by construction."*

That was true when it was written. **It is not any more** — the gather thread just built the missing
shape: `census::gather_bucket` counts per element yielded, and
`keyed_gather_visits_match_the_keyed_prediction` gates the **predicted count** rather than a ratio or
a wall-clock. The same instrument works here.

## The one contract decision, pinned

**Count the lookups, then hoist, then gate the PREDICTED count.**

- **Instrument:** a `#[cfg(test)]` counter incremented at the three sites, so "lookups performed" is
  a measurable quantity for the first time.
- **Hoist:** resolve the per-alpha triple **once per join node** and carry it in `FireCtx` — which
  `fire/mod.rs:57` says exists for exactly this (*"the split borrow that lets a join hold eleven
  session fields"*).
- **Gate:** lookups become **3 per join node**, not `3 × emitted pairs`. Assert the formula, not a
  time.

⛔ **NO WALL-CLOCK CLAIM.** This strike does not assert a speedup and must not report one. Six
samples or no number, and I am not asking for six. The deliverable is: the same rows, from fewer
lookups, proven by a count.

## Scope

**IN:** the counter, the hoist, the predicted-count gate, and a behaviour-identity check. Floor GREEN.

**OUT, affirmatively cut:** `temperare` §2–§5 (`root_join_delta`, `production_delta`, `key_of_el`,
`ensure_gather`) — the same instrument will serve them, one strike each; the FxHashMap-vs-SipHash
question, which is a **separate decision** needing its own evidence (say what you measured, do not
switch a hasher here); census B, D–M; A4, D2p, F2.

## Why the hasher stays out

Swapping a hasher changes iteration order. This engine has already been bitten by order — the explain
oracle's HAMT walk (F1) attributed derived facts nondeterministically. **A hasher change is a
correctness question wearing a performance hat**, and it does not ride along with a hoist.
