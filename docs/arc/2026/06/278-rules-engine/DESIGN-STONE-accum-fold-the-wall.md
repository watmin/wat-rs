# DESIGN-STONE — the fold is the wall (rematch + slot)

> **Origin (2026-08-18).** Ranked by `NEXT-STRIKES-after-shadow.md`.
> `(b)` and keyed gather made `filter` 0.2% of accum fire. What
> remains is `accum:fold` at **42.6%** (68.49 ms of 160.70 ms at
> `[200 200]`). This stone is that row, not a new index.

## The measurement

`fold_cost_with_and_without_the_binding_lookup` (release, 3 runs,
one rule, 40,000 elements):

```
count (NO per-element lookup)   9.59 ms
sum   (ONE lookup per element) 18.50 ms
delta = Bindings::get on Arc<[(Value,Value)]>   8.91 ms  (223 ns/el)
```

Four nodes on the axis (count + sum + min + max) reconstruct the
census: 9.6 + 3×18.5 ≈ 65 ms against 68.49 ms measured.

`accum:fold` is **not** `accumulate_value` alone. The mark wraps
the per-token bucket walk: `census_gather_visit`, `fact_bindings_under`
rematch, `Vec<&Element>` push, then the fold. Count's 9.59 ms is
that walk. Sum's extra 8.91 ms is `acc_var_i64` → `bindings.get(?v)`.

Element bindings are already an array
(`DESIGN-STONE-element-bindings-array`). The get is a linear scan
of 1–2 pairs through `Value::String` equality, **per element, per
value-fold**. The rematch is a second compiled walk of a cond
populate already proved.

## Why rematch is usually ceremony

`token_element_compatible` agrees with join-key equality
(`DESIGN-STONE-keyed-gather`, "the predicates are equivalent").
Populate already ran `exec_compiled` (skips `SeedCmp`). The
Element in `wm.alpha` **is** the binding array the fold wants.

Rematch is load-bearing **only** when the `:from` cond has a
leftover `SeedCmp` (a `?var` the fact does not bind — join /
exists seed). `CompiledCond.seed_reads` / `Op::SeedCmp` name
that case. No leftover → rematch cannot reject a keyed bucket
member and cannot bind anything the Element does not already
hold.

## The algorithm

Per AccumulateNode, after the keyed probe:

1. `leftover = compiled.has_seed_cmp()` (walk `ops`, including
   `Or`/`Not` children). If leftover: **today's path** (rematch,
   then `acc_var_i64`). Do not guess.
2. Else the bucket **is** the gather, in insertion order.
3. `group_keys` empty (the accum axis; token already holds every
   `:from` bind except the operand):
   - `count` → `bucket.len()`. Do not allocate `Vec<&Element>`.
   - `sum` / `min` / `max` / `mean` → resolve `?var` to a **slot
     index** on the first element (`position` on `el.bindings`),
     then read `el.bindings[slot].1` as `i64`. Empty bucket:
     count/sum emit identity; min/max/mean drop.
4. `group_keys` non-empty: still skip rematch, still slot-fold
   each group. Do not change grouping.

`AccFold` keeps the `Value` key (oracle / empty / user folds).
The slot is **per node per round**, derived from a live Element,
not stored on the interned arm (binding order is a populate
fact, not an AccFold fact).

## ★ THE ONE CONTRACT DECISION

**A leftover `SeedCmp` keeps today's rematch. Absence of leftover
is a proof, not a hint.**

Wrong-direction failure: skip rematch on a leftover cond → a
`:from` with `(> ?threshold ?v)` (seed on the token) silently
counts the whole bucket. The differentials that carry leftover
`:from` (`where-accum-from-left`) are the net. A green fold
census with a red leftover differential is not the work.

Two supporting clauses:

- **Order.** Slot-fold walks bucket indices in the same order
  the rematch walk pushed. Folds that care (`min` of equals is
  first; user folds) stay oracle-identical.
- **Empty bucket.** `count`/`sum` still emit identity. Do not
  `continue` on an empty bucket.

## The gate — the existing instrument, re-pointed

Do not invent a wall-clock. Re-point what already prints:

1. `fold_cost_with_and_without_the_binding_lookup` — after this
   stone, **sum's fold sits near count's**. The 8.91 ms / 223
   ns/el delta must collapse (assert `sum <= count × 2` and
   `count` itself falls vs today's 9.59 ms — rematch is gone).
2. `accum_fire_phase_census` at `[200 200]` — `accum:fold` must
   fall **materially** from 68 ms. Floor: **< 25 ms** mean of 3.
3. Standing rete differentials, including leftover `:from`.

## Predicted win

`[200 200]` FIRE 160.70 ms → ~100–110 ms if fold lands near
10–20 ms. Grid `:ratio` 1.12 (unresolved) → ~1.6 `:us`.
Clara's 178 ms does not move. Weigh with `GRID_SKIP_ORACLE=1`
on `accum` only after the census is green.

## Blast radius

`src/rete/kernel.rs` (the accumulate token loop + `acc_var_i64`
slot helper). `src/rete/compiled_cond.rs` only if
`has_seed_cmp` is cleaner there than a local walk of `ops()`.
No `.wat`. Oracle unmoved.

## Out of scope = REJECTED

- **Persisting the gather.** Wrong cell. Own stone (#3).
- **Skipping rematch when leftover exists.** The contract.
- **Storing the slot on `AccFold` / the arm.** Binding order is
  populate's. Derive from a live Element.
- **The alpha phase.** 30% on the census, mostly instrument +
  push. Own investigation after this fold is gone.
- **User folds / `all` / `group-by` / `distinct`.** Keep
  rematch+`acc_var_i64` until a census names them. The grid
  axis is the four built-ins.

## Sequencing

1. `has_seed_cmp` (or `!seed_reads.is_empty()` plus an ops walk).
2. Fast path in the accumulate loop. Rematch stays the else.
3. Re-point `fold_cost_*` and add the `< 25 ms` assert on
   `accum_fire_phase_census` at `[200 200]`.
4. Weigh: those two, `binary_id(wat::rete)`, clippy.
5. Then, and only then, `run-axis.sh accum` under
   `GRID_SKIP_ORACLE=1`.
