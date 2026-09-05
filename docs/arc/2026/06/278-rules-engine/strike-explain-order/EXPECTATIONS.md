# EXPECTATIONS — F1: explain attribution order

> Written BEFORE the strike. Graded by the orchestrator's own re-run, 8 process samples.

| # | what | expected |
|---|---|---|
| 1 ★ | the oracle is DETERMINISTIC | 8 process runs → the same rule 8/8 |
| 2 ★ | and it AGREES with native | native == oracle on all 8 |
| 3 ★ | control still discriminates | single-producer row stable on both (it already is — if it moves, the fixture broke) |
| 4 ★ | Gate A mutation-proved | re-introduce a raw keys-walk → **RED**; restore → green. Quote both. |
| 5 ★ | all five sites classified | each either calls the verb or carries a rune whose reason says what the fold builds and why order cannot reach it |
| 6 | one definition | `topological-node-ids` exists once; `fire.wat` calls it rather than keeping its own copy |
| 7 | floor | `./scripts/floor.sh` → **0 failed** |
| 8 | clippy | rc=0 |
| 9 | blast radius | `wat/rete/oracle/**` + one lint + one test pair. **Zero lines in `src/`.** |

★ load-bearing. **Row 7 is the deliverable.**

## Runtime prediction

40–70 min. Classifying the four other sites is the real work; the sort itself is a copy.

## Trap doors, named in advance

- **Gate B is probabilistically red at HEAD, not deterministically.** `experiri` saw 2/8 agreement,
  I saw 0/8. Do not present its red as the proof — Gate A is the proof. Say so in the SCORE.
- **A stale binary.** `wat/` is `include_str!`'d; a `wat/` edit needs a rebuild before it is live.
  A "no change" reading against a stale binary looks exactly like a refutation.
- **Sorting a site that something depends on being unsorted.** STOP-1.
- **A rune with a hand-wave reason.** STOP-2. `pass.wat:395` conjes into a Vector — that one is
  order-preserving by construction and the burden is on showing no consumer cares.
- **Re-run the floor at FINAL state**, after the last edit.

## What would make me reject the result

- Gate A unattempted or un-reddenable.
- `explain.wat` sorted and the other four left unexamined — that is the check rung presented as the
  shape rung.
- Two copies of the sort.
- A rune whose reason does not name what the fold builds.
- A red floor of any size.
