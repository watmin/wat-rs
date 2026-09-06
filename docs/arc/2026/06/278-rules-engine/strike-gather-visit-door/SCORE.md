# SCORE — the gather-visit door

Every gather examination goes through `gather_bucket`. The keyed-gather ratio did not cross 2.0. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ three sites count | **HOLD.** Distinct/All/GroupBy/User (`acc.rs`), non-leftover `:from` (`accumulate.rs`), no-`SeedCmp` leftover rematch (`fire/mod.rs`) all walk `gather_bucket`. Existing counted sites converted so they cannot double-count. |
| 2 ★ ratio both ways | **HOLD.** See below. Did **not** cross 2.0. STOP-1 did not fire. |
| 3 ★ lint mutation-proved | **HOLD.** Raw `bucket.iter()` at `acc.rs:403` → RED `raw gather-bucket walk (not gather_bucket)`. Restore → 7/7 green. |
| 4 ★ non-vacuity | **HOLD.** Named subject list ≥ 3, each path `is_file()`, `gather_bucket(` used ≥ 3 times. |
| 5 O(1) not counted | **HOLD.** `bucket.len()` (Count) and `bucket.is_empty()` (exists/not no-SeedCmp) unchanged. They examine nothing. |
| 6 no engine change | **HOLD.** Census is `#[cfg(test)]`; helper's `inspect` calls a release no-op. Same facts, same rows. |
| 7 floor | **HOLD.** `Summary [ 452.954s] 5447 tests run: 5447 passed (1 slow), 21 skipped`. `.floor/2026-09-06T00-34-27Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 cost | **HOLD.** No `*_cost` assertion moved. STOP-3 did not fire. |

★ load-bearing. Row 2: no second strike from this axis.

## Ratio

`keyed_gather_visits_do_not_scale_with_group_count` — constant 800 elements, tokens 10 → 80:

| | small (G=10 W=80) | big (G=80 W=10) | ratio |
|---|---|---|---|
| **before** | 800 | 800 | **1.00x** |
| **after** | 800 | 800 | **1.00x** |

The three newly counted paths are not on this axis (Count/Sum/Min/Max leftover already counted; Distinct and no-SeedCmp rematch are not taken). Converting the seven old sites to the helper is net-zero. The gate still cannot see a whole-memory scan on an *uncounted* path — those paths now count, but this workload does not enter them. The lint is what closes the form.

## Mutation (row 3)

Reverted Distinct's walk to `bucket.iter().map`:

```
raw gather-bucket walk outside `gather_bucket`:
  src/rete/kernel/fire/acc.rs:403: raw gather-bucket walk (not `gather_bucket`)
```

Restored `gather_bucket`. Lint 7/7 green.

## Doors

| door | why it is safe |
|---|---|
| `gather_bucket` | the only gather-bucket walk; each yield is one `census_gather_visit` (no-op in release) |
| `rune:lint(gather-walk-not-examining)` | HashJoin probes in `fire/mod.rs` — not Acc/Neg/Exists examinations. Reason ≥ 40 chars. |

`bucket.len()` / `is_empty()` / `first()` are not walks. If either becomes a walk, it needs counting, not a rune.

## Still open (named, cut)

Census sections B, D–M. `temperare` cost rows. A4, D2p, F2. The keyed-gather gate still only measures the paths this accum fixture enters.
