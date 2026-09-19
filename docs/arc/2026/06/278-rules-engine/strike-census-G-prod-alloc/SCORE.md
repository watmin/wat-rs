# SCORE — two unread false counters are gone

A hardcoded `2` wearing an allocation name is not a measurement. Nothing read either one. Deleted. Floor GREEN is the proof. No mutation is available — inventing a reader would be the defect this strike removes.

## Scorecard

| # | result |
|---|---|
| 1 ★ both gone | **HOLD.** `grep -rn 'prod:vec-alloc\|prod:record-alloc' src/ tests/` → no hits, comments included. The replacement comment does not name them, so the grep stays honest. |
| 2 ★ floor GREEN | **HOLD.** `.floor/2026-09-06T23-02-13Z/`: `Summary [ 461.184s] 5465 tests run: 5465 passed (2 slow), 21 skipped`. |
| 3 ★ no NAMES list moved | **HOLD.** `accum_cost.rs` exact-equality list untouched. `accum_matcher_op_census` still green without an edit. |
| 4 ★ the reason is recorded | **HOLD.** `eval_insert.rs:188-191`: constant not measurement, zero readers, real-allocation count rejected (census F), path still covered by `prod:shape` / `prod:resolve` / `prod:construct` and `prod:derivations`. |
| 5 siblings kept | **HOLD.** `prod:class-alloc` still at `:157` — one real `String` per call, name accurate, unread but true. `prod:derivations` untouched (four readers). |
| 6 no replacement counter | **HOLD.** Nothing added under any name. |
| 7 floor count | **HOLD.** `5465 passed, 21 skipped` — same count. No new tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the proof, such as it is.** Deleting a counter that nothing reads cannot red anything by construction.

## Why `prod:class-alloc` stays

The line is falsehood, not unreadness. Census E kept a never-firing *guard* because it is true. These two were *instruments* reporting a number nobody measured: kwargs allocate three vecs, positional two, `Vec::with_capacity(0)` allocates none, and an `Err` between the bumps made them attempts vs successes. `prod:class-alloc` counts one real `String` allocation per call. Unread, but true. Not swept up.

## Mutation

None. Same as census D: saying that is the report. A green floor after the deletion is the demonstration that nothing was watching.

## Final floor

`.floor/2026-09-06T23-02-13Z/`: `Summary [ 461.184s] 5465 tests run: 5465 passed (2 slow), 21 skipped`. Count unchanged. No new tests.
