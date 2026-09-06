# DESIGN — §5: the hoist is illegal unless key-set stability is PROVEN, and proving it closes a named gate gap

> Drawn 2026-09-06 at HEAD `92173d9ae`. Source: vigilia 2026-09-05 `temperare` §5, and
> **`experiri`'s explicit caution against acting on it**. Verified on disk at THIS HEAD.

## The cost

`ensure_gather` (`fire/mod.rs:2076`) derives its own cache key on **every call**:

```rust
let join_keys: Arc<[Value]> = gather_join_keys(sample, els, GatherIntern::from_wm(wm, alpha_id)).into();
let index = cache.entry((alpha_id, Arc::clone(&join_keys))).or_insert_with(|| …);
```

`gather_join_keys` filters the sample's binding keys through `col_field_of`, `collect`s a
`Vec<Value>`, and **sorts** it; `.into()` then allocates an `Arc<[Value]>` and memcpys. On a cache
**hit** — the designed common case — all of it is discarded, and only the `entry` probe's hash of
`(i64, Arc<[Value]>)` is consumed, which hashes every `Value` in the key.

It sits under `filter.rs`'s `for tok in new_tokens { … }`, where `driver` is already correctly
hoisted and the key derivation is not.

## ⛔ Why this is NOT a hoist strike

**`gather_join_keys` takes the TOKEN's bindings.** The derived key set is a function of the sample,
which is exactly why the cache is keyed on it. Hoisting it out of the token loop asserts that every
token at a node produces the same key set — and if that is ever false, two readers with different
key sets collide on one index.

**`gather_probe_cost.rs:18-22` names that failure mode and admits the gate is blind to it:**

> *"(b) a cache keyed on `alpha_id` ALONE — it would read 2 here … **This gate cannot catch that**;
> the DESIGN's contract clause and the differentials [are what protect it]."*

`experiri` closed its vigilia report on exactly this: *"Any hoist here must preserve that, and I have
not traced every `ensure_gather` caller's key-set stability. **Do not act on §5 without driving the
differentials that stone points at.**"*

`temperare` asserted stability — *"every token reaching one Negation/Exists node carries the same key
set"* — **as a claim, not a measurement.** Three `temperare` rows have now been driven and my own
reading of two was wrong in opposite directions. This one is not mine to assume either.

## The one contract decision, pinned

**Measure stability first. The hoist is legal only if it is proven, and the proof is the deliverable.**

1. **Instrument:** per `(node, alpha)`, count `ensure_gather` calls and the number of **distinct**
   key sets observed. Drive on the axes that exercise Negation / Exists / Accumulate leaves.
2. **Then:**
   - **distinct == 1 everywhere** → hoist the derivation beside `driver` in `filter.rs`, **and land
     the gate `gather_probe_cost.rs` says does not exist**: assert key-set stability per node, so a
     future divergence is caught rather than silently colliding.
   - **distinct > 1 anywhere** → **REFUTE §5.** The per-token derivation is load-bearing, the row
     closes as measured, and the finding is that a real workload varies key sets — which is worth
     more than the hoist.

⛔ **The gate is not optional in the first branch.** Landing the hoist without it would take a
protection the DESIGN's contract clause currently provides by construction and convert it into a
convention — the exact trade this arc has spent nine strikes undoing.

⛔ **NO WALL-CLOCK CLAIM.** Counts only.

## Scope

**IN:** the stability instrument, the readings, and either (hoist + stability gate) or (refutation
with numbers). Floor GREEN.

**OUT, affirmatively cut:** the `Arc` allocation itself if stability fails; `GatherCache`'s key
shape; census B, D–M; A4, D2p, F2.

## Why this is the right last `temperare` row

The other four were arithmetic — a lookup either happens per element or it does not. This one is a
**correctness precondition wearing a performance row**, and the honest outcome may be that we close
a gate gap and change no hot-path code at all.
