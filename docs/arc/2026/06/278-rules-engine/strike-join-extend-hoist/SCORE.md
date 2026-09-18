# SCORE — hoist join_extend's per-alpha triple

Lookups dropped from **1 per emitted pair** to **3 per HashJoin**. The DESIGN's "3 per pair" was the source shape, not the live count: `span_from_row` is skipped because the join-index already has a BindSpan. Floor GREEN. No millisecond is claimed.

## Scorecard

| # | result |
|---|---|
| 1 ★ lookups counted | **HOLD.** `JOIN_ALPHA_LOOKUPS` at the three gets. Before reading on the 2-condition join axis, table below. |
| 2 ★ count drops to per-node | **HOLD.** Before: `lookups == pairs`. After: `lookups == 3 × hash-joins` at four `(keys, fanout)` points. Both quoted. |
| 3 ★ gate is a formula | **HOLD.** `3 × JOIN_EXTEND_HASH_JOINS`. Independent of pairs. |
| 4 ★ behaviour identical | **HOLD.** Floor includes the differential and oracle suite. 5452 passed. No fact-level STOP-2. |
| 5 no hasher change | **HOLD.** Zero edits to a map's type. `session.rs` untouched. |
| 6 no wall-clock claim | **HOLD.** Counts only. |
| 7 floor | **HOLD.** `Summary [ 456.014s] 5452 tests run: 5452 passed (3 slow), 21 skipped`. `.floor/2026-09-06T02-31-55Z/`. Two new tests vs the previous 5450. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the deliverable, row 4 did not bend.**

## Before (instrument, no hoist)

One HashJoin, `pairs = keys × fanout²`. The live inner-loop lookup is `compiled_conds.get` only — `el.binds.len > 0` on the indexed path, so `span_from_row`'s two gets do not run.

| keys | fanout | pairs | lookups |
|---|---|---|---|
| 2 | 8 | 128 | **128** |
| 4 | 4 | 64 | **64** |
| 8 | 2 | 32 | **32** |
| 16 | 1 | 16 | **16** |

DESIGN said three gets per pair. That is what `join_extend`'s source does if occupancy is empty. On this axis the join-index already carries the BindSpan (`DESIGN-STONE-join-index-span`), so the before count is **1 × pairs**, not 3 × pairs. The hoist still resolves all three once per node: `join_extend` still has the occupancy arm.

## After — `3 × hash-joins`

| keys | fanout | pairs | lookups | pred `3 × 1` |
|---|---|---|---|---|
| 2 | 8 | 128 | **3** | 3 |
| 4 | 4 | 64 | **3** | 3 |
| 8 | 2 | 32 | **3** | 3 |
| 16 | 1 | 16 | **3** | 3 |

All five call sites: `alpha_id` is loop-invariant (STOP-1 did not fire). Resolve once per HashJoin visit (catch-up, or once shared by step-3 and step-4). `join_extend` takes `&JoinAlpha`, not `alpha_id` plus three maps.

## Mutation (test arithmetic)

Add the measured before (`pairs`) to the after count.

```
(K,F)=(2,8):  after 3 + 128 pairs = 131  pred 3
(K,F)=(16,1): after 3 +  16 pairs =  19  pred 3
weaker (lookups > 0): PASS on 131/19
```

- **weaker PASSES:** `lookups > 0` on 131 and 19.
- **equality FAILS:** `131 ≠ 3` and `19 ≠ 3`.

Not an unkeyed engine.

## Borrow shape

`JoinAlpha` owns the two side-table slices (cloned **once per node**). Holding `&wm.bind_only` across `hj_step3_term1(&mut wm, …)` is the borrow FireCtx exists to split; the lifted step functions take `&mut wm` whole. `compiled` stays a ref into `arm.compiled_conds`, which is not `wm`. FireCtx still holds the three maps — `GatherIntern` and `element_with_row_span` need them. The handle is an argument to `join_extend`, not a FireCtx field: `hash_join_pass` reuses one FireCtx across alphas.

No hasher was swapped. STOP-3/4 did not fire. `gather_probe_cost.rs` is a call-site of the new signature, not a new test.

## Still open (named, cut)

`temperare` §2–§5. The FxHashMap-vs-SipHash question. Census B, D–M. A4, D2p, F2.
