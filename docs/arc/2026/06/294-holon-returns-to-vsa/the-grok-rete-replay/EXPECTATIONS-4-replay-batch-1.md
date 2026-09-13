# EXPECTATIONS 4 — replay batch 1, grok-rete #11 → #60 (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 50 replayed steps, in order | `git log --oneline <start>..HEAD \| grep -c 'REPLAY(grok-rete #'`; each subject's N and `-x` trailer against `bootstrap/era/replay-plan/commits.tsv` | 50, #11 → #60, each trailer naming its grok-rete hash |
| E2 | docs-only steps are plain cherry-picks | the 3 docs commits #26 `e9a5e0156`, #48 `ee1fe443b`, #56 `fcfe1b291`: `git show --stat` | only `docs/`/`.md` files, no conversion |
| E3 | every produced `.wat` passes `--check` | `./target/release/wat --check` over every `.wat` the batch added or modified | rc 0, or a deliberate `.bad` named in the log |
| E4 | C's own tests | a sample of ≥10 steps (every shared step + a spread of code steps): the tests the step names, run by name | all green |
| E5 | rete behaviour survived | the shared steps' named tests (#27 #29 #39 #43 #47 #50 #53 #58 #59 #60) | green; each resolution in the log follows the ownership rule |
| E6 | the checkpoints | the SCORE's floors at #35 and #60 + the orchestrator's own floor at #60 | green; clippy 0 |
| E7 | the log | `REPLAY-LOG.md` | one row per step; every `convert.sh` report line for a produced file carried |
| E8 | no hazard in range | `git diff --name-only <start>..HEAD` vs `wat-scripts/fixes/` and `bootstrap/era/replay-plan/main-deleted.txt` | none (STOP-6 never needed) |
| E9 | no hand-edited corpus `.wat` | read every step's diff for a `.wat` change not produced by `convert.sh` + `merge-file` | none, except wat inside `.rs` strings (logged) |

**Runtime prediction:** ~2 h for grok (pilot unit costs — docs ~20 s, grok-only code ~2 min, shared
~3.5 min once `convert.sh` is 10–14 s a commit — plus two ~5 min checkpoints). The log's wall column
replaces this estimate; the orchestrator re-estimates the remaining 591 from it.

**Trap doors:**
- #59 and #60 are large shared steps (11 and 13 files; 7 and 6 `.rs`) — the most likely STOP-3;
- a converted `.wat` whose nested child program the chain rewrites: `--check` does not start children
  (finding 8; stone 3 comes between batches) — log every `UNRESOLVED`/`UNREGISTERABLE` line so it can be
  followed up;
- `convert.sh` prints `UNREADABLE` for era context files it cannot lex — expected, not a STOP.
