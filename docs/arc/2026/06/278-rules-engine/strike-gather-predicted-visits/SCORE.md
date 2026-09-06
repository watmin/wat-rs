# SCORE — predicted gather visits, not a ratio

Equality is the proof. The ratio is a second reading and is exactly blind on `and-exists`. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ equality per path | **HOLD.** Visits `==` the keyed formula, not a bound. Simple: `G·W`. And-exists: `G·W·(1+W)`. Table below. |
| 2 ★ ≥3 `(G,W)` points | **HOLD.** Four points, `G×W = 800`: `(10,80) (20,40) (40,20) (80,10)`. |
| 3 ★ equality reddens where the ratio does not | **HOLD.** Simulated whole-memory: ratio **PASSES 1.00**, equality **FAILS** `72800 ≠ 64800` and `72800 ≠ 8800`. Quoted below. Arithmetic, not an unkeyed engine. |
| 4 ★ which assertion is the proof | **HOLD.** The equality. The ratio stays as a second reading. |
| 5 no fudge | **HOLD.** No constant added to either formula. |
| 6 no engine change | **HOLD.** `src/rete/kernel/tests/rank_and_instrument.rs` only. Zero lines in `src/rete/kernel/fire/`. |
| 7 floor | **HOLD.** `Summary [ 452.447s] 5450 tests run: 5450 passed, 21 skipped`. `.floor/2026-09-06T01-54-27Z/`. Two new tests vs the previous 5448. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is the deliverable.**

## Equality — visits == formula at four points

| path | (10,80) | (20,40) | (40,20) | (80,10) | pred |
|---|---|---|---|---|---|
| old-axis, distinct, all, group-by | 800 | 800 | 800 | 800 | `G·W` = 800 |
| and-exists | **64800** | **32800** | **16800** | **8800** | `G·W·(1+W)` |

Every cell is exact. No fudge. `min`/`max`/`User` share the Distinct/All/GroupBy `gather_bucket` materialise already driven; Count and exists-of-Leaf remain O(1) uncounted.

## Row 3 — mutation (test arithmetic, not an unkeyed engine)

Drive real and-exists observations, then add `G · elements` (`elements = 800`) in the test's own arithmetic. That is the whole-memory-per-token regression the ratio cannot see.

```
(G,W)=(10,80): keyed 64800 + 10·800 = 72800  pred 64800
(G,W)=(80,10): keyed 8800  + 80·800 = 72800  pred 8800
ratio of fakes: 1.00  (≤ 2.0 would PASS)
```

- **ratio PASSES:** `72800 / 72800 = 1.00` ≤ 2.0.
- **equality FAILS:** `72800 ≠ 64800` and `72800 ≠ 8800`.

The two fakes are identical because `800(1+G+W)` is symmetric in G and W on this axis. That is the DESIGN table, driven.

I did not build an unkeyed engine. STOP-3 did not fire: the equality against `G·W·(1+W)` reddens under the simulation; the ratio of the same two numbers does not.

## Ratio kept (second reading)

`keyed_gather_visits_do_not_scale_with_group_count` and the per-path bound remain. Old axis / distinct / all / group-by still 800/800 1.00x; and-exists 64800/8800 0.14x. Comment on the ratio test names the and-exists blindness and points at the equality.

## Still open (named, cut)

Folds not yet driven as their own wat surface (`min`/`max`/user-fn) — the walk is the same arm. Census B, D–M. `temperare`. A4, D2p, F2.
