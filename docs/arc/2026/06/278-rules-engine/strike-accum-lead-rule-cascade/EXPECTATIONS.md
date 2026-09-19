# EXPECTATIONS — the fourth cell, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the axis runs | `echo '[<dials>]' \| cargo run --release --bin wat -- wat-scripts/perf/grid/accum-lead-rule-cascade.wat` | one `#grid/Result` |
| ⭐ answer independent of cascade depth | run at ≥2 depths | **`:derived` byte-identical across depths, on BOTH engines.** A spread is STOP-1 |
| native == oracle | that line | byte-identical |
| Clara agrees | `check-grid-three-way.sh accum-lead-rule-cascade` | `ALL THREE MATCH` |
| count is `anchors`, constant | `cargo nextest run --release -E 'test(wat_scripts_grid_port_check)'` | `anchors`, derived from the header before the run, NOT a function of depth |
| the axis can fail | a mutation or outcome 1 | a quoted red naming the extra rows |
| floor | `scripts/floor.sh` | 5475 + new rows, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

50–80 min. The `.clj` twin and four registration points dominate, as with the last two axes.

## Trap doors, named in advance

- **⛔ A cascade that is not inert.** If any rule or query can observe S1/S2/S3, the count becomes
  legitimately depth-dependent and the axis asserts nothing. This is the single design point the
  whole instrument rests on — `where-accum-lead-cascade`'s header says so in its own words.
- **The constant count "corrected" into a formula.** Every other axis's count tracks its dial. A
  future hand will read `anchors` as a bug. Say in the header that constancy IS the assertion.
- **A green read as "L2-1 is fine".** A green at depth 3 with `anchors=2` is thin. State the depths
  and anchor counts actually run, and do not close L2-1 on one cell of one size.
- **Modelling the wrong cell in Clara.** The distinguishing feature is RULE (not query) with a join
  after the leading accumulate. A `.clj` that drops the join or uses a query models a covered cell
  and would agree for the wrong reason.
- **`git checkout <commit> -- <path>` stages.** `git diff --stat <path>` then prints nothing and an
  empty mutation is indistinguishable from a working cure. Use `git diff HEAD`.
