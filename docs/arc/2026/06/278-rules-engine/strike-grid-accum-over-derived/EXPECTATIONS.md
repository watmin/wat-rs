# EXPECTATIONS — the `accum-over-derived` axis, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the axis runs at all | `echo '[<depth>]' \| cargo wat ./wat-scripts/perf/grid/accum-over-derived.wat` | one `#grid/Result` line carrying `:derived`, `:oracle-derived`, both `-ns` |
| native == oracle on `:derived` | the same line | **byte-identical vectors.** A difference here is STOP-1 and is the most valuable outcome this strike can produce |
| exactly ONE Tally survives | inspect `:derived` | one `enc(1,0,n)` element, not `depth`-many. `depth` tallies means supersession leaked at this depth |
| Clara agrees | `wat-scripts/perf/grid/check-grid-three-way.sh` | `:accuracy`, `:oracle-accuracy`, `:port-accuracy` all matching |
| the count is the anti-vacuity instrument | `cargo nextest run --release -E 'test(wat_scripts_grid_port_check)'` | `depth + 1`, derived from the header formula BEFORE the run |
| axis set reconciles | same gate | on-disk stems == table stems; a missing row REDs |
| floor | `scripts/floor.sh` | 5473 + whatever the new rows add, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

45–75 min. The `.wat` is mostly transcription from `retract-multiplicity` + `deep-cascade`; the
`.clj` twin and the four registration points are where the time goes. The JVM three-way adds
seconds, not minutes, at correctness sizes.

## Trap doors, named in advance

- **⛔ The expectation that hides the defect.** If the expected count is written as *"one row per
  tally"* or read off a first run, a leaked intermediate tally becomes the expectation and the axis
  passes over the exact thing it exists to catch. Derive `depth + 1` from the shape first, in the
  header, then run. This is the `port_check` contract in its own words.
- **A constant witness.** If `:derived` carried only the Tally, the count would be `1` at every
  size and no dial could falsify it — `[[a-count-cannot-see-a-value-defect]]`. Encoding the derived
  `Step` levels too is what makes the count move.
- **Depth chosen by what passes.** Pick the depth as a stated choice — meaningfully past the
  probe's two — and if the oracle is too slow there, report the number rather than shrinking until
  it is quiet (STOP-3).
- **Clara's fixture drifting from the wat one.** The two files must model the same rule set;
  `retract-multiplicity.clj`'s header is explicit that the wat header is read FIRST and mirrored.
  A `.clj` that dedups where the `.wat` does not reproduces exactly the blindness this axis removes.
- **A green here proves less than it looks.** If everything matches, the finding remains L2-3's:
  the strata differ, the facts do not, and two headers claim a lockstep that does not hold. Do not
  let a green axis be written up as "the engines are in lockstep."
