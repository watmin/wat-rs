# SCORE — `compiled:calls` is two names now

`compiled:exec` is an execution. `compiled:span-elided` is the skip. Same two sites, no new bump, `skip_span` untouched. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ key split | **HOLD.** `grep -rn '"compiled:calls"' src/ tests/` → no hits. |
| 2 ★ both mutations RED | **HOLD.** Live probe `c4_probe_bind_only_decides_skip_span_for_the_accum_axis`. Both REDs on `elided_built == exec_empty`. Quoted below. Restored. |
| 3 ★ assertions have content | **HOLD.** Passing probe: `elided_built=80200 exec_built=0` · `exec_empty=80200 elided_empty=0`. |
| 4 ★ no behaviour change | **HOLD.** Only census key strings and the gates that read them. `skip_span`'s condition is byte-identical. |
| 5 ★ four sentences true | **HOLD.** `compiled_cond.rs` now says EXECUTION counter. `accum_cost.rs:44` names exec vs span-elided. `:93` message names EXECUTED for `compiled:exec`. `:1365` block replaced: the two keys name the two paths. |
| 6 C10 | **HOLD.** `accum_cost.rs` C10 paragraph still present; notes census B splits names, not which arm runs. |
| 7 floor | **HOLD.** Final `5465 passed, 21 skipped` — same count as strike 1. No new tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the proof the split kept both sites gated; row 3 is the proof the new assertions can fail.**

## Row 3 — content

```
C4 PROBE: bind-only conds 3/3 · built pool=0 vals=1000 exec=0 elided=80200 · empty pool=120200 vals=1000 exec=80200 elided=0
```

`elided_built == exec_empty == 80200`. `exec_built == 0`. `elided_empty == 0`.

## Row 2 — live mutations

**Delete `census_count("compiled:exec")`:**

```
assertion `left == right` failed: elided_built=80200 exec_empty=0: same (fact, alpha) pairs by two paths. An inequality means one of the two bump sites (`fire/delta.rs` skip_span, `compiled_cond.rs` exec_compiled_with_key_ids) stopped counting
  left: 80200
 right: 0
```

**Delete `census_count("compiled:span-elided")`:**

```
assertion `left == right` failed: elided_built=0 exec_empty=80200: same (fact, alpha) pairs by two paths. An inequality means one of the two bump sites (`fire/delta.rs` skip_span, `compiled_cond.rs` exec_compiled_with_key_ids) stopped counting
  left: 0
 right: 80200
```

Same equality, opposite side.

## First floor (captured, not re-run)

`.floor/2026-09-06T10-07-23Z/`: `5464 passed, 1 failed`. Arm: `no_stale_path_in_doc` — `compiled_cond.rs` named `fire/delta.rs`, which does not exist. The skip_span comment now names `alpha_activate_fact`.

## Final floor

`.floor/2026-09-06T10-19-36Z/`: `Summary [ 460.486s] 5465 tests run: 5465 passed (2 slow), 21 skipped`.
