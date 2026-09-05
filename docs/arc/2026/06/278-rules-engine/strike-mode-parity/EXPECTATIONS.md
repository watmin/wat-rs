# EXPECTATIONS — mode parity gate

> Written BEFORE the strike, so the result cannot move the goalposts.
> Graded by the orchestrator's own re-run, never the rider's report.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 ★ | the SOUNDNESS arm is RED at HEAD | `cargo nextest run --release -E 'test(mode_parity)' > L 2>&1` | the empty-file row FAILS: `--check` Accepted, run Rejected |
| 2 ★ | the LIVENESS arm is RED at HEAD | same run | the deep row FAILS: `--check` DiedBySignal, run Accepted |
| 3 ★ | the gate DISCRIMINATES | same run | the `good.wat` control row PASSES — both modes Accepted |
| 4 | non-vacuity | read the source | the gate asserts its fixture list is non-empty and names each case it ran |
| 5 | no exit-code table | read the source | classification is {Accepted, Rejected, DiedBySignal}; no `assert_eq!` on a raw code across modes; **no depth constant** |
| 6 | blast radius held | `git diff --stat` | `tests/cli/` only; zero lines in `src/` |
| 7 | the rest of the floor is unmoved | `./scripts/floor.sh > F 2>&1`, read the **Summary** line | `5420 + N run, N failed` where the N failures are exactly rows 1–2 and nothing else moved from 5420 passing |
| 8 | clippy silent | `cargo clippy --all-targets --release` | rc=0 |

★ = load-bearing. Rows 1 and 2 failing is **success**. A green gate is a failed strike.

## Runtime prediction

25–40 min. The deep fixture generator is the only fiddly part; the driver shape is a copy.

## Trap doors, named in advance

- **The deep fixture fails for the wrong reason.** The orchestrator's first attempt produced `2
  type-check errors` from a wrong `def` arity and a 4-arg `main`, and both modes agreed on it —
  which reads exactly like "the finding is false." STOP trigger 3 exists for this.
- **The depth is not portable.** The ward's ceiling was 400–600; the orchestrator's was 600–1000,
  same binary, different fixture shape. If ~1000 does not redden on the rider's machine, **raise
  the depth — do not weaken the assertion**, and record the depth used and why in the SCORE.
- **`--check` may abort the test harness** if anything loads the deep fixture in-process. It must be
  driven as a subprocess only. See the ⛔ in the BRIEF.
- **Row 7 is the one most likely to surprise.** Adding fixtures to `tests/cli/` could trip a walking
  gate nobody expects. `docs_wat_loads_or_declares_why_not` walks `docs/arc` only — verified this
  session — but `tests/` gates were not enumerated for this. If one reddens, that is a finding, not
  a nuisance.

## What would make me reject the result

- Rows 1–2 green (the gate cannot see the defect it was built for).
- A depth constant or a raw exit-code comparison in the assertion.
- Any `src/` change.
- A fixture whose failure mode was not verified to be signal death.
