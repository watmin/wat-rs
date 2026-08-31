# EXPECTATIONS — the label follows the arithmetic

> Written **before** the strike. Scored against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate is RED before | `cargo nextest run --release --no-capture -E 'test(minimum)'` | **FAIL**, naming **seven** files: accum_cost, fanout_cost, rank_and_instrument, strat_cost, accum_alpha_cost, cascade_cost, harvest_cost |
| 2 | the gate sees all three spellings | the RED output | `harvest_cost` and `strat_cost` MUST appear — they use `let (a,b) = (a / r, b / r)` and a `/= r` reader is blind to them |
| 3 | non-vacuity | read the gate | explicit `assert!(!files.is_empty(), …)` with its reason, cited |
| 4 | the gate is GREEN after | as row 1 | **passed** |
| 5 | no `MINIMUM` header became `MEAN` | `git diff` on the 7 files | **zero** header labels changed from MINIMUM to MEAN. Any such change is the ★ decision failing |
| 6 | `calibrate_mark_ns` untouched | `git diff src/rete/kernel/tests/mod.rs` | its body does not move; only `stat()` / `net_of` / `total_mean` / the row loop do |
| 7 | the tables still print | run one axis with `--no-capture` | a table with real non-zero figures — **a conversion that seeds `INFINITY` and never updates prints `inf`**, which is the obvious way this goes wrong quietly |
| 8 | rete surface | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 9 | the floor | `./scripts/floor.sh`, Summary from the captured log | **5,182 / 5,182**, 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — two, and the second is unusual

- Restore one converted accumulator to `+= … / r` → **that file alone** reddens. Restore.
- **Mutate the GATE's reader** to look only for `/= r` → confirm it stops seeing `harvest_cost`
  and `strat_cost`. That is the orchestrator's own wrong regex from the scoping pass, and proving
  the gate is immune to it is the point of the gate.

⚠ If a mutation reddens nothing, that is a coverage finding, not a null result.

## Runtime prediction

60–90 minutes. 96 edits across 8 files is the largest mechanical sweep in this chain; budget four
or five release builds plus the ~370s floor. The edits are uniform, which is why one strike rather
than three — **a partial sweep leaves the label lying for the unconverted half, which is the defect
itself.** The file-scoped gate is what makes partial unshippable.

## Trap doors named in advance — with the step

- **`inf` in a printed table.** Seeding `f64::INFINITY` and failing to update prints `inf` rather
  than failing. **Step:** row 7 — run one axis with `--no-capture` and read the numbers, do not
  infer from a green test.
- **A divide that looks like RUNS and is not.** **Step:** check what the denominator binds to. If
  it is facts/pairs/iterations, leave it; `calibrate_mark_ns` is the worked example.
- **The numbers in the arc's record will move.** Expected and out of scope — DESIGN cuts it. Do not
  re-record figures inside this strike.
- **The count may not be 96.** It is a measurement, taken after a first count of 37 that was wrong.
  **Step:** report your per-file count against DESIGN's table and explain any difference. Agreement
  reached by copying the number is worse than a disagreement.

## What would make this a failure even if every test passes

A `MINIMUM` header changed to `MEAN`. That makes the gate green by moving the label to match the
code — which is `89e8c3ed0` performed in the opposite direction, and would mean this arc fixed the
symptom twice and the cause never.
