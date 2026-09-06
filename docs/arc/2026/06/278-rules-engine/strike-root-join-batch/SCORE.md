# SCORE — batch root_join_delta; hoist its span lookups

Both costs were **1.00 per element**. Token writes batched. Span lookups hoisted to node scope. Floor GREEN. No millisecond is claimed.

## Scorecard

| # | result |
|---|---|
| 1 ★ both counters, denominators | **HOLD.** `span` vs `record_token` vs elements. Table below. |
| 2 ★ token writes batched | **HOLD.** Before: `record_token` = elements. After: `record_tokens` = 1 at every driven size; `record_token` = 0. |
| 3 ★ ORDER unchanged | **HOLD.** Argued below. Floor includes oracle/differential. |
| 4 ★ span branch by count | **HOIST.** Span/elements = **1.00** on fanout and accum. Lookups (`bind_only.get`, `cond_key_ids.get`) moved to `node_id` scope. Writes still per element via `span_from_resolved`. |
| 5 gate is a formula | **HOLD.** `record_tokens` independent of occupancy; equal at 10×10 and 100×20. |
| 6 no wall-clock | **HOLD.** Counts only. |
| 7 floor | **HOLD.** `Summary [ 457.782s] 5458 tests run: 5458 passed (2 slow), 21 skipped`. `.floor/2026-09-06T04-53-33Z/`. Two new tests vs the previous 5456. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is whether this ships.**

## Before (instrument, no batch)

| axis | denom | elements | span | span/e | record_token | record_tokens |
|---|---|---|---|---|---|---|
| fanout 10×10 | 100 | 100 | 100 | **1.00** | **100** | 0 |
| fanout 100×20 | 2000 | 2000 | 2000 | **1.00** | **2000** | 0 |
| accum G=10 W=80 | 10 | 10 | 10 | **1.00** | **10** | 0 |
| accum G=80 W=10 | 80 | 80 | 80 | **1.00** | **80** | 0 |

Occupancy is empty binds again (same population as `col_field_of`). Cost 1 is not rare. Cost 2 is unconditional, as DESIGN said.

## After

| axis | denom | elements | span | record_token | record_tokens |
|---|---|---|---|---|---|
| fanout 10×10 | 100 | 100 | 100 | **0** | **1** |
| fanout 100×20 | 2000 | 2000 | 2000 | **0** | **1** |
| accum G=10 W=80 | 10 | 10 | 10 | **0** | **1** |
| accum G=80 W=10 | 80 | 80 | 80 | **0** | **1** |

`record_tokens` = one `(alpha, RootJoin-child)` that emits, independent of occupancy. Span writes remain per element; the two `*node_id` gets do not.

## Order (row 3)

`production_delta` was safe because nothing read `wm.production` mid-nest (`encode_view` does not). **This nest also does not read what it writes.** `root_join_delta` only appends to `wm.beta` / `d_beta`. Hash-join reads those memories **after this pass returns**.

Buffer is per `(node_id, child_id)`, same two loops as before. `news.iter()` order is unchanged. `Token` is `Copy`. `record_tokens` `extend_from_slice`s the buffer. Sequence at each child is the old per-element `push` sequence.

`push_match` still runs per element **before** the Token is buffered. `Token.matches` is a `BindSpan` into `match_pool`; the slots are written in the same order as before. Buffering copies the span words, not the pool.

STOP-1/2 did not fire. Floor oracle/differential green.

## Mutation (test arithmetic)

```
10×10:  after 1 + 100 elems  = 101
100×20: after 1 + 2000 elems = 2001
weaker (record_tokens > 0): PASS on 101/2001
```

Equality of sizes **FAILS**. Weaker `> 0` **PASSES**.

## Still open (named, cut)

`temperare` §5 (`ensure_gather`). FxHashMap. Census B, D–M. A4, D2p, F2.
