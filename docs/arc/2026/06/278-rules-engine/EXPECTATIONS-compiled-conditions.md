# EXPECTATIONS — compiled conditions (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | bindings are IDENTICAL | the new differential test | compiled == interpreted, **including the array** — same pairs, same order |
| 2 | the failure path allocates NOTHING | the new counter test | key-allocations on failure **0**; the same measure against the interpreter is **non-zero** |
| 3 | no timing regression | `a0_depth_cost_split_at_equal_work` | both columns within noise or better — **a no-harm row, not a win row** |
| 4 | no timing regression, fact-heavy | `fanout_per_call_alpha_census` | `alpha:match` ≤ its 1.211 ms — again **no-harm** |
| 5 | floor | `cargo nextest run --release` | unchanged + the new tests, Summary line read directly |
| 6 | the deny wall | `cargo clippy --release --all-targets --workspace` | exit 0, zero warnings, **own exit code** |
| 7 | oracle unmoved | `git diff --stat` | zero lines under `wat/`; public binding representation unchanged |

## Independent prediction

- **Runtime:** 90–150 min. The mechanism is fully specified; the work is the compiler (resolving
  bind→field→slot chains through `classify_rete_clause`) and an executor that is bit-identical to a
  function it is replacing.
- **Rows 3 and 4 will barely move, and that is the expected outcome.** The target is 1.1% of a
  fact-heavy fire. A rider reporting "no measurable speedup" has **succeeded**, provided rows 1 and 2
  hold.

## Trap-doors named in advance

- **Row 2 is the load-bearing row.** Row 1 can pass with a perfectly correct executor that still
  allocates exactly as before — that is the whole stone missed while every other row goes green. The
  interpreter-comparison half is what stops row 2 reading as success when nothing changed.
- **Row 1 must compare the ARRAY, not a boolean.** Same verdict with different bindings produces
  wrong joins downstream, and the count differentials would not catch it.
- **Do not tune toward rows 3/4.** They are no-harm rows. This stone's justification is not a
  percentage, and a rider optimizing for a timing delta will be chasing noise on a 115 ms fire.
- **`alpha_match_inner` stays.** It is the differential's other half. Deleting it because it has no
  production caller is a separate ruling (`feedback_no_consumers_does_not_mean_dead`).
- **Clippy is a gate, not an afterthought.** The previous stone in this arc introduced a
  `type_complexity` error that every other row was blind to; it was caught only by running the wall
  out of habit. Row 6 exists because that scorecard did not have it.

## What would make me reject the strike outright

A "both matched" comparison standing in for row 1; a failure path that still allocates; a second
private condition parser; any edit under `wat/`; a change to the public bindings representation; or
a green claimed off a piped exit code.
