# EXPECTATIONS — random is threaded

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ BOTH axes classify | mirror `purity.rs:1802-1803` for the new verbs | `Axis::Pure` **ok** *and* `Axis::Deterministic` **ok**. `uuid::v4` fails the second; **passing it is the entire design** (STOP-2) |
| 2 | ★ same seed, same sequence | draw 100 from seed S twice | **identical**. Without this, chaos is unreplayable and inadmissible under the floor doctrine |
| 3 | ★ different seeds diverge | draw 100 from S and S+1 | different — it is actually random, not a constant |
| 4 | ★ `below` is in range and unbiased | draw 100k `below 6`, count buckets | every value in `[0,6)`, buckets within a few % of even. A biased schedule is a lying instrument (STOP-3) |
| 5 | ★ no ambient state | read the diff | the state is a parameter and a return value; **no statics, no thread-locals, no cells** (STOP-1) |
| 6 | registered in both places | `register_builtins` + `intrinsic_meta` | present in each — perf-3's red was exactly this omission |
| 7 | no new core type | `git diff` | state is `i64`; no `Rng` type added |
| 8 | no `.wat` corpus change | `git diff --stat wat/ wat-scripts/` | **empty** — nothing calls it yet (STOP-4) |
| 9 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5192 tests |

**Runtime prediction:** 60–120 minutes. The algorithm is four lines; the two registrations and the
determinism classification are the work.

## Trap doors, named in advance

- **★ Row 1 is the stone.** Rows 2–9 all pass on an ambient `rand::i64` that happens to be seeded
  somewhere. The claim that distinguishes this design is that a **threaded** draw satisfies
  `Axis::Deterministic`, which `uuid::v4` provably does not — making it usable in `sigma` fns, rete
  bodies, and everywhere else the analysis demands determinism. If that classification cannot be
  made, say so; the shape is wrong and the chaos work needs redesigning around it.
- **Modulo bias is the classic silent defect** and it would make every chaos measurement subtly
  untrue in a way no test of *ours* would catch. Row 4 is cheap and it is not optional.
- **Do not seed it here.** Seeding is the caller's, and the seed must be *reported* — that discipline
  belongs to the chaos brief. An intrinsic that seeds itself from the clock is unreplayable by
  construction.
- **Firing on nothing:** rows 5–9 pass on a correct-but-ambient implementation. Rows 1 and 2 are what
  require the threaded shape.
