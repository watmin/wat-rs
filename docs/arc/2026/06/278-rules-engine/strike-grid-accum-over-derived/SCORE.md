# SCORE — the `accum-over-derived` axis is green at depth 9; that is not lockstep

Native, oracle, and Clara agree on the facts at a depth past the probe's two. One Tally
survives. The expected count was `depth + 1 = 10` before the run, and the run agreed.
No `+1` moved. The finding remains L2-3's: the strata differ, the facts do not, and two
headers claim a lockstep that does not hold.

## Scorecard

| # | result |
|---|---|
| 1 ★ native == oracle on `:derived` | **HOLD.** Byte-identical vectors at depth 9. STOP-1 did not fire. Quoted `#grid/Result` below. |
| 2 ★ exactly one Tally | **HOLD.** One `enc(1,0,10)` element, not nine. `n = depth + 1 = 10` (all Step facts, including seeded Step(0)). |
| 3 ★ Clara agrees | **HOLD.** `check-grid-three-way.sh accum-over-derived`: `clara=10 native=10 oracle=10 ALL THREE MATCH`. |
| 4 ★ count is anti-vacuity | **HOLD.** `CORRECTNESS_SIZES` expected 10, derived from the header formula (`depth + 1`) before the run. `port_check` PASS 14.243s. |
| 5 ★ axis set reconciles | **HOLD.** Four registration points: `SIZES[accum-over-derived]="9"`, port_check row, axes_live `&[4]`, static `.clj`. No `gen-accum-over-derived.sh`. |
| 6 ★ floor | **HOLD.** `Summary [ 485.436s] 5473 tests run: 5473 passed (3 slow), 22 skipped`. `.floor/2026-09-07T09-30-58Z/`. Count unchanged: the axis lives inside walking tests. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 8 no cure | **HOLD.** No `src/`. No `wat/`. No `+1` moved. `stratify.rs`, `stratify.wat`, `probe_arc278_oracle_accumulate_supersedes` untouched. |
| 9 depth was a stated choice | **HOLD.** 9, written in the `.wat` header before the run. Oracle 145.9 ms, not minutes. STOP-3/4 did not fire. `compile-all` was `Compiled`. |

★ load-bearing. **A green here does not prove the engines are in lockstep.**

## Row 1 — quoted `#grid/Result`

```
#grid/Result {:axis "accum-over-derived" :size #wat.core/PersistentVector [9] :derived #wat.core/PersistentVector [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010] :native-ns 305824 :oracle-derived #wat.core/PersistentVector [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010] :oracle-ns 145867434}
```

Decoded: `enc(0,k,0)` for k in [1,9], plus `enc(1,0,10)`. Ten elements. Sort, not dedup.

`check-grid-three-way.sh accum-over-derived` (exit 0):

```
accum-over-derived   clara=10   native=10   oracle=10    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (3s)
```

## Clara twin, first pass

The first three-way was STOP-2-shaped: Clara `[1000000000000001]` (Tally n=1, no derived Steps)
against both wat engines at 10. Cause was the fixture, not the engines: `eval` of `defrule`
wrapped the condition as `[(Step (= prev level))]` (a vector of one list). Clara interned
the vars (`:rule true`) and they never fired. Cured to `[Step (= prev level)]` — type is the
vector's first element, matching `gen-deep-cascade.sh`. Re-run: Clara matches. Named here so
the first red is not a silent re-run.

## What this does not prove

L2-3 stands. Native puts Tally at stratum 1 and evaluates it once; the oracle puts it at
stratum 0 and supersedes `depth` intermediate tallies. The facts agree at depth 9 with a live
Clara referee. Two headers still claim a lockstep that does not hold. That cure is out of
scope.

## Landing

Two new files (`accum-over-derived.{wat,clj}`), three registration rows. Depth 9 correctness,
depth 4 liveness. Static `.clj`, no `gen-`. Do not commit unless asked.
