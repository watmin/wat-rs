# EXPECTATIONS 5 — replay batch 2, grok-rete #61 → #125 (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 65 replayed steps, in order | `git log --oneline <start>..HEAD \| grep -c 'REPLAY(grok-rete #'`; each subject's N and `-x` trailer against `commits.tsv` | 65, #61 → #125, each trailer naming its grok-rete hash |
| E2 | docs-only steps are plain cherry-picks | the 30 docs commits (#69 #70 #73 #75 #77 #79 #81 #84 #85 #86 #87 #89 #92 #97 #102 #104 #107 #109 #111 #113–#121 #123 #125): `git show --stat` | only `docs/`/`.md` files |
| E3 | every produced `.wat` passes `--check` | `./target/release/wat --check` over every `.wat` the batch added or modified | rc 0, or a deliberate `.bad` named in the log |
| E4 | C's own tests | a sample of ≥ 12 steps (every large shared step + a spread): the tests the step names, by name | all green |
| E5 | rete behaviour survived | the shared steps' named tests (#64 #68 #71 #72 #78 #80 #83 #88 #90 #91 #93 #94 #95 #96 #98 #99 #100 #101 #105 #108 #110 #122) | green; each resolution follows the ownership rule |
| E6 | the checkpoints | the SCORE's floors at #93 and #125 + the orchestrator's own floor at #125 | green; clippy 0 |
| E7 | the per-step walls ran | `REPLAY-LOG.md`: a census verdict AND a stone-3 gate verdict on each of the 33 qualifying steps (#61 #62 #64 #65 #66 #68 #71 #72 #74 #76 #78 #80 #82 #83 #88 #90 #91 #93 #94 #95 #96 #98 #99 #100 #101 #103 #105 #106 #108 #110 #112 #122 #124), a lint-subset verdict on each of the 29 `.rs` steps | every row present; no STOP-8, no STOP-10 |
| E8 | no hazard in range | `git diff --name-only <start>..HEAD` vs `wat-scripts/fixes/`, `wat/`, `main-deleted.txt` | none |
| E9 | no hand-edited corpus `.wat` | every step's diff for a `.wat` change not produced by `convert.sh` + `merge-file` | none, except wat inside `.rs` strings (logged) and a LATENT call head (logged) |
| E10 | the census at #125 | `census.sh --diff <baseline at start> .census/latest` | no STOP-8 |

**Runtime prediction:** ~2 h 15 m (batch 1 ran #22 → #60 in 66 min, ~1.6 min a step; batch 2 has 22
shared steps to batch 1's 10; ~40 s of census + lint and ~30 s of stone 3's gate on each of 33 steps;
two ~4 min checkpoint floors, each now carrying stone 3's gate at ~63 s under load).

**Trap doors:**
- the large shared steps #83 (14 files), #90 (14, all `.rs`), #95, #101, #108, #110 — the likely STOP-3;
- a LATENT call head (main's registry-only resolution) in any grok-rete `.wat` — the addendum's rule;
- `convert.sh` prints `UNREADABLE` for era context files it cannot lex — expected, not a STOP.
