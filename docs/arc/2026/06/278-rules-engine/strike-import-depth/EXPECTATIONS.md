# EXPECTATIONS — a format whose acceptance depends on the reader's stack has no format

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## ⛔ NO PINNED TEST COUNT — this row is deliberate

The previous strike's scorecard pinned the floor at "5,188 + **exactly three**". That did not
predict the work, it **bounded** it: the rider reported that writing a fourth arm *"would have
falsified your own scorecard row before you ran it"*, and two real arms went undriven because of my
arithmetic. So:

> **The floor must be ≥ 5,191 plus every arm you drive. Exceeding it is a PASS. Report the final
> number; do not tune the work to hit one.**

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | controls green before | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green — the corpus imports legitimately nested programs |
| 2 | the `:and` probe is RED before | its own `nextest -E` | **FAIL**, having been *accepted*: pre-fix, import takes a tower just over the bound without complaint |
| 3 | the `:user`-cycle probe is RED before | its own `nextest -E` | **FAIL**, same way — and if it goes green while the `:and` one is red, the budget is not threaded through `unpack_prog` |
| 4 | both GREEN after | same two | **2 passed**, each refused with `malformed` naming the bound |
| 5 | the refusal is a value, not a death | read the probe's assertion | it matches the `Err(_)` refusal, not merely "did not abort" |
| 6 | the bound was measured | read the constant's comment | it states the observed corpus maximum AND the multiplier. A bare round number fails this row |
| 7 | the budget is shared | `grep -n 'depth' src/rete/export.rs` | `unpack_expr`, `unpack_prog`, `unpack_pat` and the four cond/driver/rhs entries all carry it |
| 8 | no instrument left behind | `git diff` | the depth-measuring instrument from trap 4 is gone |
| 9 | blast radius | `git diff --stat` | `src/rete/export.rs` + `tests/rete/probe_arc278_export.rs`. Nothing else |
| 10 | the floor | `./scripts/floor.sh`, Summary from the captured log | **≥ 5,191 + every new arm**, zero FAIL rows, exit 0 |
| 11 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — one per arm, and the arms are named

Rows 2/3 → row 4 must prove the `:and` descent and the `:user`↔`:prog` cycle **separately**. They
are different code paths and a single probe cannot prove both — B1 demonstrated exactly that twice
(a panic that blew past its own assertion; an arm 2 that never ran because arm 1 failed first).

Per arm, state: **proven** (driven, red→green), **reachable but not driven**, or **not reachable,
and why**. An unreached arm named as unreached is a pass; an unreached arm not mentioned is a fail.

One further mutation, cheap: **set the bound to 1 and confirm the corpus goes red.** If it does
not, the wall is not on the path every import takes, and row 7 is passing on a technicality.

## Runtime prediction

50–70 minutes. Four or five release builds at ~2m40s (the threading, the measurement instrument,
its removal, at least one mutation, likely one fix-up), one floor at ~370s.

## What would make this strike a failure even if every test passes

**A bound that only `unpack_expr` counts.** The `:user` arm re-enters through `unpack_prog`, so an
expr-only counter is walked past by an alternating tower — and the `:and` probe would still go
green, because it never alternates. Row 3 exists precisely because row 2 cannot see this.

The second failure shape: **a bound chosen for roundness.** The whole finding is that acceptance
rested on an unstated environmental property. Replacing it with an unmeasured constant swaps one
unexplained criterion for another and calls it a wall.
