# BRIEF — Strike 2a: `filter` goes native

Design: `DESIGN-STONE-seq-traversal-one-door.md`. **Read its Strike-2 section including the
"⛔ THE TWIN ROUTE IS DEAD" ruling** — an earlier version of this brief said to mint a
`filter-stream` twin. That is superseded. You are making `filter` **native**, like `map`.

Strike 1 (`61211301`) made `seqable->stream` native and linear. `filter` is still a wat `defclause`
with five per-container arms, each stepping its eager source by `(rest coll)` — O(n) per step,
O(n²) per walk.

## The work, in one paragraph

Replace `filter`'s five wat clauses with one native `eval_filter` that takes any seqable and returns
a lazy `Stream`, dispatching through the seq-container registry exactly as `map` does. No twin, no
per-container arms — one body, the shape Clojure has. Then delete the wat `defclause`.

## Read in order (the rooms)

1. **`src/collection/transform.rs`** — `eval_seqable_to_stream` (Strike 1, the bottom of the file).
   **This is your closest exemplar**: `StreamContainer::of_value` dispatch, `NativeThunk` lazy
   cells, the `List`-snapshot rule. Your `filter` is this plus a predicate.
2. **`src/collection/transform.rs`** — `eval_vec_map`. The other half of the shape: how a native
   seq verb evaluates a wat function per element (`apply_function`).
3. **`src/collection/infer.rs:637`** — `extract_lazyable_elem`, the checker-side Seqable set, and
   `infer_seqable_to_stream` above it. Your `infer_filter` mirrors it — same set, plus a predicate
   arg whose type must unify with the element type.
4. **`src/runtime.rs`** (search `:wat::core::seqable->stream`) and **`src/check.rs`** (search the
   same) — the two registration sites Strike 1 added. Yours go beside them.
5. **`wat/seq.wat:59-102`** — `filter`'s five clauses, which you delete. Note the fifth is a bare,
   unparameterised `PersistentVector` arm.

## Implementation sketch

Simplest correct shape: **compose, do not re-derive.** `filter` normalises through the existing
native `seqable->stream`, then lazily walks the resulting `Stream` applying the predicate. That
reuses Strike 1's per-container correctness (including the `List` snapshot) instead of duplicating
it, and keeps this strike to one new idea.

```
eval_filter(args=[pred, coll]):
    pred_fn  = eval(pred)
    stream   = <the same normalisation eval_seqable_to_stream performs on coll>
    return   a lazy Stream that, on each force, walks `stream` until the predicate
             holds, then yields Cons{that element, <the rest, filtered>}; Empty when exhausted
```

The predicate is a wat fn — call it with `apply_function`, and **propagate its errors** rather than
swallowing them; a raising predicate must surface, not silently drop the element.

Then delete `filter`'s `defclause` from `wat/seq.wat`, leaving a comment pointing at the native
implementation (Strike 1's deletion comment is the model).

**`filterv` stays as it is** — it is `(into [] (filter …))` and keeps working unchanged. Do not
touch it.

## Blast radius

`src/collection/transform.rs` (the new eval), `src/collection/infer.rs` (the new infer),
`src/runtime.rs` + `src/check.rs` (one registration each), `wat/seq.wat` (delete `filter`'s
clauses). **Do NOT touch** `remove`, `take-while`, `drop-while`, `interpose`, `reductions` — 2b.

## Name the concept while you are in there

At `extract_lazyable_elem`'s definition, add a doc comment naming it for what it is: **the Seqable
set — the type wat cannot currently spell**, with the three blockers (no surface nature admits a
builtin; no builtin satisfies a surface; wat has no ad-hoc unions per R7) and a pointer to
`docs/arc/2026/04/109-kill-std/NOTE-seqable-has-no-name-in-wat.md`. Do not rename anything else.

This is not decoration — it is what keeps the 109 note a marked delta instead of a good intention.

## The RED gate — write it FIRST, prove it RED, then fix

Add a test asserting the absence of the quadratic; run it before changing anything; paste the RED
output.

A wall, not a stopwatch: `(into [] (filter pred pv))` over a **4000-element PersistentVector** must
complete in **under one second**. It is ~12,000ms today.

**The source MUST be a `PersistentVector`, not a `Vector`.** Strike 1's rider drew this same wall
over a `Vector` and it passed *before* any fix existed — a `Vector`'s `rest` is a flat
clone-and-collect, cheap enough per element that O(n²) does not cross a one-second wall at n=4000,
while a `PersistentVector`'s trie rebuild misses it by 35×. If your gate does not go red, fix the
source, never the wall.

Model it on `seqable_to_stream_keep_stays_under_wall_at_n4000` in `src/collection/transform.rs`.

## Your gates

Foreground only. Never background a command and return.

1. `cargo build --release --all-targets` — exit 0, zero warnings (workspace lints are `deny`).
2. Your RED gate — **RED before** (paste it), **GREEN after**.
3. `cargo test --release --lib -- filter --nocapture` — green.
4. Load order, since you touched `wat/`: a two-line `:user::main` printing
   `(:wat::deporder::verify-stdlib)` must print `[]`.

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor.

## STOP triggers

Ship nothing, report, stop. None is permission to improvise.

- **STOP-1** — the gate does not fail before the fix.
- **STOP-2** — `filter`'s RESULTS change: different elements, different order, or laziness lost.
  `(take 1 (filter …))` over a large source must not realise the whole thing.
- **STOP-3** — the checker cannot type native `filter` over all four seqable heads *plus* the bare
  unparameterised `PersistentVector` that the fifth wat arm accepted today. Dropping that case
  would silently narrow what `filter` accepts — report the exact error instead.
- **STOP-4** — you find yourself editing another verb, `seqable->stream`, `filterv`, or `eval_rest`
  to make something pass.

## Report back

Each gate's result, the RED and GREEN output, the diff, and anything that surprised you. Do not
commit.
