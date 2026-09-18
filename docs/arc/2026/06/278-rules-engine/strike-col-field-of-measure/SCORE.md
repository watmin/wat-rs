# SCORE — count `col_field_of`, then hoist: occupancy is empty binds

**Decision: HOIST.** The DESIGN's vacuous-worry was wrong — the opposite of §1. Occupancy is `binds.len == 0`; that is when hash-join *indexes*. `col_field_of` ran once per occupancy element. Floor GREEN. No millisecond is claimed.

## Scorecard

| # | result |
|---|---|
| 1 ★ both counters, separable | **HOLD.** `col_field_of` entries vs `key_of_el` empty-binds vs `key_of_el` total vs `from_wm`. |
| 2 ★ existing axes | **HOLD.** Fanout cell (hash-join sites) and accum cell. No invented workload. |
| 3 ★ denominator | **HOLD.** Occupancy-right = `keys×fanout`; accum readings = `G×W`. Table below. |
| 4 ★ DECISION | **HOIST.** Fanout: 2000/2000 empty-binds, `col_field_of` 2002 → **3**, independent of occupancy. |
| 5 `from_wm` | **Eight reference copies + an i64.** Not a lookup. It ran per element because `GatherIntern` borrows `bind_pool` and `element_with_row_span` needs `&mut` — the literal sat in the loop to drop the borrow. Hoisted as a consequence of `col_field_of` (intern once, drop, then mut pool). Claude was right it is copies, not a cost of the same class. |
| 6 no wall-clock | **HOLD.** Counts only. |
| 7 floor | **HOLD (final).** `Summary [ 457.075s] 5456 tests run: 5456 passed (1 slow), 21 skipped`. `.floor/2026-09-06T03-57-36Z/`. First floor RED, captured, not re-run: citation `key_of_occupancy` (the fns are `_wm` / `_ctx`) at `.floor/2026-09-06T03-47-24Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 4 is the deliverable.**

## Before (instrument, no hoist)

| axis | denom | key_of_el | empty | empty/k | col_field_of | from_wm |
|---|---|---|---|---|---|---|
| fanout 10×10 (pairs=1000) | 100 | 100 | 100 | **1.00** | **102** | 101 |
| fanout 100×20 (pairs=40k) | 2000 | 2000 | 2000 | **1.00** | **2002** | 2001 |
| accum G=10 W=80 (read=800) | 800 | 0 | 0 | n/a | 14 | 14 |
| accum G=80 W=10 (read=800) | 800 | 0 | 0 | n/a | 84 | 84 |

Hash-join catch-up keys **occupancy**, which has empty binds (`DESIGN-STONE-join-index-span` writes the BindSpan *after* `key_of_el`). 100% of `key_of_el` takes the packed arm. `col_field_of` ≈ occupancy + 2 (`gather_join_keys` filters).

Accum unary gather already hoists `col_field_of` (`build_gather_index` / `append`). `key_of_el` is 0. The leftover `col_field_of` scales with G (cache/join-key filters), not with readings. Not this defect.

## After — `col_field_of` does not scale with occupancy

| axis | denom | key_of_el | empty | col_field_of | from_wm |
|---|---|---|---|---|---|
| fanout 10×10 | 100 | 100 | 100 | **3** | **2** |
| fanout 100×20 | 2000 | 2000 | 2000 | **3** | **2** |
| accum G=10 W=80 | 800 | 0 | 0 | 14 | 14 |
| accum G=80 W=10 | 800 | 0 | 0 | 84 | 84 |

Gate: fanout `col_field_of` at 10×10 **equals** 100×20. Same for `from_wm`. `key_of_el` still equals occupancy — we still key each element; we just don't look up the field each time.

3 = 2 `gather_join_keys` filters + 1 hoisted catch-up `col_fields_for`. Not a recorded occupancy number.

## Mutation (test arithmetic)

Add occupancy to the after count.

```
10×10:  after 3 + 100 occ  = 103
100×20: after 3 + 2000 occ = 2003
weaker (col_field_of > 0): PASS on 103/2003
```

Equality of the two sizes **FAILS** (103 ≠ 2003). Weaker `> 0` **PASSES**.

## Borrow shape

`col_field_of` is pure in `(alpha_id, join_key)`. `GatherIntern::from_wm` cannot live across `element_with_row_span(&mut bind_pool)`. Fields are computed in a dropped intern, then `join_key_from_columns` reads `i64_by_fact` only. Bound-span elements still take `key_of_el` with a short-lived intern; occupancy on these axes never does.

Counters kept — they are the gate.

## Still open (named, cut)

Accum's leftover `col_field_of` (14 vs 84, scales with G). `temperare` §2 and §5. FxHashMap. Census B, D–M. A4, D2p, F2.
