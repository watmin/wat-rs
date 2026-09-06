# SCORE — gather key-set stability: distinct == 1; hoist + gate

**Decision: HOIST, with the stability gate.** Every driven `(node, alpha)` derived **one** key set. The hoist is safe by the shape of the data; the gate is the regression net. Floor GREEN. No millisecond is claimed.

## Scorecard

| # | result |
|---|---|
| 1 ★ distinct per (node, alpha) | **HOLD.** Table below. Calls and distinct, with denominators. |
| 2 ★ DECISION | **HOIST-with-gate.** `gather_join_keys` once beside `driver_of`; Leaf tokens reuse the `Arc`. |
| 3 ★ gate reddens | **HOLD.** Synthetic drives `assert_gather_key_stability` (the live verb). Quote below. Blinded `== 1` → `>= 1` now FAILS the synthetic. |
| 4 ★ differentials unmoved | **HOLD.** Floor includes oracle/differential. 5460 passed. |
| 5 if refuted | n/a — not the branch taken. |
| 6 no wall-clock | **HOLD.** Counts only. |
| 7 floor | **HOLD.** `Summary [ 458.987s] 5460 tests run: 5460 passed (2 slow), 21 skipped`. `.floor/2026-09-06T05-48-07Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is the price of the hoist.**

## Stability readings

| axis | denom | rows (node, alpha) | calls | max distinct |
|---|---|---|---|---|
| accum G=10 W=80 (count+sum+exists) | 10 groups / 80 readings | 3 / 2: 1 call; 5 / 2: 10 calls; 8 / 7: 1 call | 12 | **1** |
| exists-of-leaf G=10 W=8 | 10 groups | 3 / 2: 10 | 10 | **1** |
| not-of-leaf G=10 W=8 | 10 groups | 3 / 2: 10 | 10 | **1** |
| exists-of-and G=10 W=8 | 10 groups | 4 / 3: 90 | 90 | **1** |

Max distinct across driven nodes: **1**. Accumulators already called `ensure_gather` once per node (first token). Filter Leaf paid once per token before the hoist; after, `gather_join_keys` runs once per node. Combinator `:exists` of `:and` still derives inside `binding_extensions` (90 calls) — not the Leaf path this hoist covers; still distinct 1.

## Why the hoist is safe (shape, not the gate)

`gather_join_keys` reads only key **names**: `.map(|(k, _)| k)` discards values, then filters by `col_field_of(&intern, k)` — a function of `alpha_id` and the name. The set depends on the token's key names, `alpha_id`, and `elements[0]`, all fixed across one node's token loop. A3's field doc: `Or`/`Not` branch-local binds never reach the output slots, so the names are the node's compiled shape. The hoist is safe by that shape. The gate is the regression net, not the argument.

## Gate mutation (quoted red)

Two synthetic `census_ensure_gather` calls at node 7 / alpha 3 with `{?g}` then `{?g, ?v}`, driving **`assert_gather_key_stability`** (the same verb the live axes use):

```
assertion `left == right` failed: synthetic: node 7 alpha 3 derived 2 key sets — hoist would collide
two key sets at one node
  left: 2
 right: 1
```

`catch_unwind` Err.

REVIEW-1: blinding the live gate `== 1` → `>= 1`. Re-run:

```
PASS  gather_key_sets_are_measured_per_node_and_alpha
FAIL  gather_key_stability_gate_reddens_when_one_node_derives_two_key_sets
      stability gate did not redden under two key sets at one node
```

The proof now detects the gate being weakened. Gate restored to `== 1`.

## Hoist

`ensure_indexed` is the cache probe with already-derived keys. `filter.rs` / `filter_after_join.rs` compute `gather_join_keys` from `new_tokens[0]` beside `driver_of`, then pass `&Arc<[Value]>` into the Leaf arm. Combinator drivers still take the per-token `ensure_gather` path.

## Still open (named, cut)

Combinator-inner `ensure_gather` still per token. `GatherCache` key shape. Census B, D–M. A4, D2p, F2.
