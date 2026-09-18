# SCORE — retract-multiplicity is a recorded `:MISMATCH`, not a red test

The axis stages `F(k)` twice, retracts `F(0)` once, and reports Out keys sorted not deduped. Clara keeps the bag. wat drops every equal `F`. Floor GREEN. No cure.

## Scorecard

| # | result |
|---|---|
| 1 ★ duplicate staged | **HOLD.** Seed is `F(i), F(i), G(i)` for `i in [0, items)`. Retract is `F(0)` once. Scratch on that seed: `facts-after-insert=3` `facts-after-retract=1` — both `F` gone, `G` remains. |
| 2 ★ neither side set-collapses | **HOLD.** Clara `all-codes` is `(sort (map :?k …))`. wat `derived-vector` is `sort`, not `distinct`. Clara line: `[0 1 1 2 2]`. |
| 3 ★ verdict verbatim | **HOLD.** Quoted below. `:accuracy :MISMATCH :oracle-accuracy :MISMATCH :port-accuracy :match`. |
| 4 ★ floor GREEN | **HOLD.** `Summary [ 459.017s] 5460 tests run: 5460 passed (2 slow), 21 skipped`. `.floor/2026-09-06T07-21-21Z/`. |
| 5 size tables | **HOLD.** `SIZES` and `CORRECTNESS_SIZES` both `[3]`, `want_n=2`. `SIZED_AXES` also gained the row — the live gate asserts exact set equality with disk; omitting it reds the floor. |
| 6 `.wat` gates | **HOLD.** `every_wat_scripts_file_loads` PASS. `every_rete_name_in_wat_scripts_code_resolves` PASS. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is the proof; row 4 is what makes it landable.**

## Row 3 — quoted `#grid/Verdict`

```
#grid/Verdict {:axis "retract-multiplicity" :size [3] :accuracy :MISMATCH :oracle-accuracy :MISMATCH :port-accuracy :match :runs 1 :ratio 11.7863 :min 11.7863 :max 11.7863 :wat-ns 30369 :wat-ns-min 30369 :wat-ns-max 30369 :clara-ns 357938 :winner :us :wat-wall-ms 397 :clara-wall-ms 3133 :wall-ratio 7.8917 :wall-winner :us :fire-share-pct 0.01}
```

`run-axis.sh` stderr on the same run:

```
run-axis: MISMATCH axis=retract-multiplicity size=[3] run=1
  wat   :derived [1 2]
  clara :derived [0 1 1 2 2]
run-axis: ORACLE MISMATCH axis=retract-multiplicity size=[3] run=1
  oracle :derived [1 2]
  clara  :derived [0 1 1 2 2]
```

`check-grid-three-way.sh retract-multiplicity` (exit 1, expected — the script fails the process on a pairing mismatch; that is not the floor):

```
[retract-multiplicity] ⛔ oracle != clara  =>  THE SPEC IS WRONG (size [3])
      oracle (2 elems): 1 2
      clara (5 elems): 0 1 1 2 2
      only in oracle:
      only in clara: 0
[retract-multiplicity] ⛔ native != clara  =>  THE FAST PATH IS WRONG (size [3])
      native (2 elems): 1 2
      clara (5 elems): 0 1 1 2 2
      only in native:
      only in clara: 0
grid-three-way: FAILURES above (0 of 1 axes agreed, 4s)
```

No `oracle != native` line. Port pairing holds. STOP-1 did not fire.

`report_pair` prints through `sort -u` (unique keys only); the pairing itself is a byte compare of the multiplicity-preserving vectors. Clara's `1 1 2 2` survives into the elem count (5 vs 2) and into the `#grid/Verdict` line.

## Retract `F(0)` only — vacuity

Retracting `F(k)` once for every `k` empties wat Out (`facts-after-retract` leaves only `G`). Three-way and the port check then report VACUOUS and the floor goes red — STOP-3. Size `[3]`, retract `F(0)` only: wat/oracle `[1 2]`, Clara `[0 1 1 2 2]`, both non-empty.

## wat derived is unique-by-value (not this axis collapsing)

Before retract, wat Out is `[0 1 2]` — one per key, not `[0 0 1 1 2 2]`. Production memory is unique-by-value (the derived-multiplicity divergence, DESIGN OUT as justified). After one retract of `F(0)`, wat is `[1 2]` (0 gone). Clara is `[0 1 1 2 2]` (0 remains once, remaining keys still a bag). The axis itself does not `set`/`distinct`. The retract witness is 0 present vs absent.

## Twin is static `.clj`, not `gen-retract-multiplicity.sh`

DESIGN IN named `gen-retract-multiplicity.sh`. A `gen-` twin is a perf axis (`run-all.sh:81-87`); `hunt_tooling_selftests.rs` reds one without a LADDER rung; a LADDER rung puts the axis on `check-grid-speed.sh`, which treats `:accuracy :MISMATCH` as a gate failure — the recorded violation would become a red. DESIGN OUT forbids the perf ladder. Same convention as `parametric-erasure`: static `retract-multiplicity.clj`. The `#grid/Verdict` line above was produced by a throwaway `gen-` that is **not** in the tree (`run-axis.sh` requires that pairing). Re-run is `check-grid-three-way.sh retract-multiplicity`.

`SIZED_AXES` in `wat_scripts_grid_axes_live.rs` is a third table the BRIEF did not name. Disk vs that array is exact set equality on the floor; the row is `[3]`, same size, liveness-only.

## Finding — CI three-way will fail until the cure

`check-grid-three-way.sh` exits 1 on this axis. CI `parity` invokes it with no args. That is not the nextest floor (row 4 holds). It is the script treating a recorded `:MISMATCH` as a process failure, which DESIGN said the grid does not do (`run-axis.sh` exits 0). Not patched here — the cure of retract is the next strike, and this verdict is its acceptance test. Named so it is not inherited as a surprise.

## Still open

The cure (retract drops one equal fact, Clara parity). After that, this axis's three-way goes green and CI three-way goes green with it. No engine change in this strike.
