# SCORE — `match:calls` now counts every invocation

One statement moved two lines up, above `alpha_pattern`'s `?`. Both sides of the documented parallel now bump before their guards. Name kept. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ the bump precedes the guard | **HOLD.** `matcher.rs:547-548`: `census_count("match:calls")` then `let pat = alpha_pattern(cond)?;`. |
| 2 ★ before/after reported | **HOLD.** Fanout world `match:calls`: **0 → 0**. Not STOP-1. Quoted below. |
| 3 ★ the parallel is now true | **HOLD.** `matcher.rs:532-537` and `compiled_cond.rs:956-962` both say both counters bump before their guards, and that the parallel is an interpreter-vs-compiled call differential. |
| 4 ★ no behaviour change | **HOLD.** One `census_count` statement relocated. No branch, no signature, no new counter, no rename. |
| 5 `interp_calls` untouched | **HOLD.** `alpha_discrimination.rs:396` is still `let mut interp_calls = 0u64;` / `interp_calls += 1`. |
| 6 `fanout_cost.rs` guards | **HOLD.** Label `match:calls (interpreter entries — expect 0)` and `prod:derivations == 40_000` untouched. |
| 7 floor | **HOLD.** Final `5465 passed, 21 skipped` — same count as census D. No new tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the whole point: the move either proves the world is clean or surfaces an interpreter entry nobody could see.**

## Row 2 — before / after (same world, both sides)

`fanout_rhs_key_alloc_census`, `--release --no-capture`.

**Before the move:**

```
  FANOUT RHS ALLOCATION CENSUS — keys=100 x fanout=20, 40,000 derived Pairs

  bindkey:alloc (RHS + alpha, both compiled — expect 0)            0
  per derived fact                                             0.00
  match:calls (interpreter entries — expect 0)                    0
  prod:derivations (non-vacuity guard — expect 40,000)        40000
```

**After the move:** identical. `match:calls = 0`. `prod:derivations = 40000`.

The interpreter is genuinely never entered on that world. The widening (invocations whose `cond` is not an alpha pattern) is **unobserved**. `accum_matcher_op_census` after the move still does not emit `match:calls` in its name set — same confirmation, second world. Do not read 0 → 0 as a demonstration that the fix fired; it is a demonstration that the only consumer never reaches the function.

## Mutation (honest, not a red)

Deleted `census_count("match:calls")`, re-ran the same fanout test, restored.

```
  match:calls (interpreter entries — expect 0)                    0
  prod:derivations (non-vacuity guard — expect 40,000)        40000
```

The printed row did not lose a visible value. The live reading is already 0, and `get` is `unwrap_or(0)`, so a missing key and a live zero print the same. The consumer does not assert on this key. Same shape as census D: saying that is the proof, not manufacturing a red.

## Final floor

`.floor/2026-09-06T11-36-17Z/`: `Summary [ 460.149s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. Count unchanged. No new tests. First floor was green; nothing to capture as an arm.
