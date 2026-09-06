# SCORE — hoist production_delta's entry

Lookups dropped from **1 per derived fact** to **1 per deriving production node**. The DESIGN's shape was right this time: the before count *is* one-per-derived-fact. Floor GREEN. No millisecond is claimed.

## Scorecard

| # | result |
|---|---|
| 1 ★ BEFORE measured | **HOLD.** Fanout cell: `entries = 40_000`, `prod:derivations = 40_000`. Table below. STOP-1 did not fire. |
| 2 ★ count drops to per-node | **HOLD.** After: `1` at four `(keys, fanout)` points and at the 40k cell. Both numbers quoted. |
| 3 ★ gate is a formula | **HOLD.** `1 × PROD_ENTRY_NODES` (one `defrule` on this axis). Independent of pairs. |
| 4 ★ behaviour identical, including order | **HOLD.** `derived_facts` / `next_delta` still push immediately. Production vec is the same sequence, `extend`ed once; vec len == pairs. Floor includes oracle/differential. |
| 5 explain index untouched | **HOLD.** `idx.entry(derived)` still keyed on the varying fact. |
| 6 no wall-clock claim | **HOLD.** Counts only. |
| 7 floor | **HOLD.** `Summary [ 458.342s] 5454 tests run: 5454 passed (2 slow), 21 skipped`. `.floor/2026-09-06T03-09-51Z/`. Two new tests vs the previous 5452. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the foundation.** The DESIGN did not need correcting.

## Before (instrument, no hoist)

One production node. `pairs = keys × fanout²`. `entry` ran inside `if seen_insert`, so on this axis (all unique Pairs) it equals `prod:derivations`.

| keys | fanout | pairs | entries | derivations |
|---|---|---|---|---|
| 2 | 8 | 128 | **128** | 128 |
| 4 | 4 | 64 | **64** | 64 |
| 8 | 2 | 32 | **32** | 32 |
| 16 | 1 | 16 | **16** | 16 |
| 100 | 20 | 40000 | **40000** | 40000 |

This is the count the last strike's DESIGN failed to earn. Here the live number matches the nest.

## After — `1 × deriving production nodes`

| keys | fanout | pairs | entries | derivations | pred |
|---|---|---|---|---|---|
| 2 | 8 | 128 | **1** | 128 | 1 |
| 4 | 4 | 64 | **1** | 64 | 1 |
| 8 | 2 | 32 | **1** | 32 | 1 |
| 16 | 1 | 16 | **1** | 16 | 1 |
| 100 | 20 | 40000 | **1** | 40000 | 1 |

Local `Vec<Value>` per node, `extend` into `wm.production` once. The `entry()` Occupied is not held across the inner loops — the explain arm still takes `&wm` whole (`encode_view`). STOP-3 did not fire.

## Mutation (test arithmetic)

Add the measured before (`pairs`) to the after count.

```
(K,F)=(2,8):  after 1 + 128 pairs = 129  pred 1
(K,F)=(16,1): after 1 +  16 pairs =  17  pred 1
weaker (entries > 0): PASS on 129/17
```

- **weaker PASSES:** `entries > 0` on 129 and 17.
- **equality FAILS:** `129 ≠ 1` and `17 ≠ 1`.

Not an unkeyed engine.

## Order

`wm.derived_facts.push` and `next_delta.push` stay in the innermost body, same sequence as before. The buffer is pushed in that same order, then `extend`ed — the production vec is the old `.push` sequence in one go. Occupancy check: one production key, vec len == pairs, at every driven point. Floor oracle/differential green.

## Still open (named, cut)

`production_pass` in `fire/mod.rs` (test-only full recompute) still `entry`s per fact — out of blast. `temperare` §2, §4, §5. The FxHashMap-vs-SipHash question. Census B, D–M. A4, D2p, F2.
