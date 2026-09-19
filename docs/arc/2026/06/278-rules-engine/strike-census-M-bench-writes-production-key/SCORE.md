# SCORE — the replica writes `bench:filter-reuse`, and cannot write a production key

Both bumps kept, moved off `filter:test-reuse`. A gate with an empty exemption list makes the old shape unwritable. Floor GREEN. First floor RED captured, not re-run.

## Scorecard

| # | result |
|---|---|
| 1 ★ the key moved, the call kept | **HOLD.** Both sites `census_count("bench:filter-reuse")`. Calls still there. |
| 2 ★ arm timings before/after | **HOLD.** Quoted below. Reconstruction `K+L = 0.349 ms` both ways. J scatter is within this test's existing noise. Not STOP-1. |
| 3 ★ the gate exists and is empty-exemption | **HOLD.** `tests/lint/kernel_tests_census_count_is_bench_scoped.rs`. `census_count` under `src/rete/kernel/tests/` may name only a `bench:` key. No rune, no exemption list. `census_counted` / `census_count_n` out of scope. |
| 4 ★ the gate is mutation-proved live | **HOLD.** Restored `filter:test-reuse` at arm J. Live gate RED naming file and key. Quoted below. Restored. |
| 5 the bumps survive | **HOLD.** Not deleted. |
| 6 no production code touched | **HOLD.** `fire/mod.rs` and the census window at `:274` unchanged. |
| 7 floor | **HOLD.** `5471 passed` — **+6** from the new lint (5 detector + 1 walking). Named below. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 4 is the point**: the rename fixes the instance, the gate removes the situation.

## Row 2 — J/K before and after

**HEAD, before the rename:**

```
  J  + the tid loop          ( 10000 x)     348.8 us   +   292.8   <- 86% of the branch
  K  + the d_beta pushes     (   200 x)     342.0 us   +    -6.8   <- the whole taken branch
  RECONSTRUCTION  K+L =  0.349 ms
```

**After `bench:filter-reuse`:**

```
  J  + the tid loop          ( 10000 x)     334.5 us   +   278.6   <- 81% of the branch
  K  + the d_beta pushes     (   200 x)     342.2 us   +     7.7   <- the whole taken branch
  RECONSTRUCTION  K+L =  0.349 ms
```

K is 342.0 → 342.2 µs. J moved 14 µs; this test's K-delta already changes sign run-to-run (HEAD −6.8, after +7.7). The reconstruction total is identical. Noise, not a fidelity loss.

## Row 4 — live mutation

Put `census_count("filter:test-reuse")` back at arm J (`node_share_cost.rs:479`). Drove `kernel_tests_census_count_names_only_bench_keys`:

```
a `census_count` under src/rete/kernel/tests named a key that is not `bench:`-prefixed:
  src/rete/kernel/tests/node_share_cost.rs:479: census_count("filter:test-reuse") — only a `bench:` key is writable from src/rete/kernel/tests
```

Names the file and the key. Restored.

(First drive of the same mutation RED'd on `bench_writes >= 2` because that assert sat above the violations check. Reordered: the rule fires first. Not a floor re-run.)

## Audit correction

Only `filter:test-reuse` is written, at two sites. The audit also named `filter:test-pass`; `grep` finds no such call in that file.

## First floor (captured, not re-run)

`.floor/2026-09-07T01-25-46Z/`: `Summary [ 460.747s] 5471 tests run: 5470 passed (3 slow), 1 failed, 21 skipped`.

Arm: `no_loose_string_assert::tests_carry_no_loose_string_assert` — detector used `contains` on the violation string at `:89` and `:93`. Cured with an exact `assert_eq!` on the whole line.

## New tests (+6)

- `kernel_tests_census_count_is_bench_scoped::detector::a_production_key_is_a_hit`
- `…::a_bench_key_is_not_a_hit`
- `…::census_count_n_is_not_a_hit`
- `…::census_counted_is_not_a_hit`
- `…::a_comment_is_not_a_hit`
- `…::kernel_tests_census_count_names_only_bench_keys`

## Final floor

`.floor/2026-09-07T01-37-02Z/`: `Summary [ 460.339s] 5471 tests run: 5471 passed (2 slow), 21 skipped`.
