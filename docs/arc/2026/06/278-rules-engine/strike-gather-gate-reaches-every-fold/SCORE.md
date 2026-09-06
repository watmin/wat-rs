# SCORE — the keyed-gather gate over every instrumented path

Holds across the surface. No ratio crossed 2.0. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ every path entered | **HOLD.** Distinct, All, GroupBy visits > 0. Mapping no-`SeedCmp` arm (`seeded_bindings_keyed`) entered via `:exists` of `:and` of two Leaves. |
| 2 ★ ratio per path | **HOLD.** Table below. |
| 3 ★ old axis unperturbed | **HOLD.** `800/800` still. |
| 4 ★ verdict | **HOLD.** Keyed-gather holds across the driven surface. No crossing. |
| 5 no engine change | **HOLD.** `src/rete/kernel/tests/rank_and_instrument.rs` only. |
| 6 floor | **HOLD.** `Summary [ 451.437s] 5448 tests run: 5448 passed, 21 skipped`. `.floor/2026-09-06T01-19-45Z/`. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. Row 4 is the deliverable.

## Readings — constant 800 elements, tokens 10 → 80

| path | G=10 W=80 | G=80 W=10 | ratio |
|---|---|---|---|
| old-axis (count+sum+exists-leaf) | 800 | 800 | **1.00x** |
| distinct | 800 | 800 | **1.00x** |
| all | 800 | 800 | **1.00x** |
| group-by | 800 | 800 | **1.00x** |
| and-exists (mapping no-SeedCmp) | 64800 | 8800 | **0.14x** |

Un-keyed whole-memory scan would be ~8× (visits ∝ token count). None of these are.

`and-exists` is larger because `:and` of two Leaves cartesian-extends inside the **keyed** bucket: `G × (W + W²)` → 10×(80+6400)=64800 vs 80×(10+100)=8800. Ratio follows W, not G. Still ≤ 2.0 against the token-scaling test.

## Which shape reaches the mapping arm

`any_seeded_keyed` (exists/not **Leaf**): no-`SeedCmp` is `!bucket.is_empty()` — O(1), not a walk.

`seeded_bindings_keyed` (Leaf under `binding_extensions`): no-`SeedCmp` maps the bucket. Reached by `:exists` of `:and` of two fact Leaves. A lone Leaf under `:exists` does not enter it. `:exists` of `:and` plus `:where` unbound `?v` (the Test is not seeded by the inner Leaf); two Leaves kept `:and` a combinator without that hole.

## User fold

`AccFold::User` shares Distinct/All/GroupBy's `gather_bucket` materialise. That walk is entered. A dedicated wat user-fn acc is the same arm, not a silent skip.

Count remains `bucket.len()` (O(1)). Exists-of-Leaf remains `is_empty()` (O(1)). Neither examines; neither is instrumented.

## Still open (named, cut)

Curing a crossing — there was none. Census B, D–M. `temperare`. A4, D2p, F2.
