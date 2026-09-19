# EXPECTATIONS — the `userfn-head` axis, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the axis runs | `echo '[<items>]' \| cargo run --release --bin wat -- wat-scripts/perf/grid/userfn-head.wat` | one `#grid/Result` with `:derived`, `:oracle-derived`, both `-ns` |
| native == oracle post-cure | that line | byte-identical vectors |
| Clara agrees | `check-grid-three-way.sh userfn-head` | `ALL THREE MATCH` |
| count is anti-vacuity | `cargo nextest run --release -E 'test(wat_scripts_grid_port_check)'` | `2 * items`, derived from the header formula BEFORE the run |
| ⭐ the axis REDDENS on the reverted cure | revert `rule-produces`, rebuild, port check | **MISMATCH naming the missing `Out` rows.** This row is the whole point |
| restored | re-run after restoring | green again |
| floor | `scripts/floor.sh` | 5475 + new rows, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

50–80 min. Two `wat/`-triggering rebuilds (revert, restore) dominate.

## Trap doors, named in advance

- **⛔ An axis that cannot fail.** The green-after-cure run is worth nothing by itself. The
  reverted-cure red is the deliverable, and the revert must be verified to have changed the file —
  an empty mutation is indistinguishable from a working cure.
- **A witness that cannot tell two failures apart.** `:derived` must carry Rate AND Out. With Out
  alone, "Out dropped" and "nothing derived" look identical, and a cure that broke both would pass.
- **The expected count read off a run.** `port_check`'s own contract forbids it. Derive `2 * items`
  from the shape in the header first.
- **`Bad` accidentally firing.** It is the live negation that raises the stratum; if the guard
  matches some `k` in `[0, items)`, the shape changes silently. Pick a `k` no key can take and say
  so in the header.
- **The `.clj` drifting from the `.wat`.** Carry the referee's two modelling statements AND their
  falsifier — *"if `mk-rate` ever computes, this file is no longer the same rule"* — into the axis
  twin, with `items` swept.
