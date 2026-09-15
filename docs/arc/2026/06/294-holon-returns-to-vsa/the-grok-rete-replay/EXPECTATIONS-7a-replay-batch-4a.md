# EXPECTATIONS 7a — replay batch 4a, grok-rete #153 → #159 (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 7 replayed steps, in order | `REPLAY(grok-rete #` count + trailers vs `commits.tsv` | 7, #153 → #159 |
| E2 | docs-only steps are plain cherry-picks | the 3 docs steps: `git show --stat` | only `docs/`/`.md` |
| E3 | the step record is complete | `scripts/replay/verify-step-record.sh <start> HEAD` | exit 0 — its first PASSING case |
| E4 | the chain never converts itself | `convert.sh` handed a chain member's path | refused, loudly (the guard fails once) |
| E5 | the 5 ported tools work on main | each loads (`every_wat_scripts_file_loads`); each fixture replays (`every_recorded_migration_replays` shards); second run changes nothing | green; idempotent |
| E6 | each port reproduces its own step | the fixture's `before.pre`/`after.post` are a migrated file's converted C^/C | the ORACLE header-spec quote present; the pair traceable to its step |
| E7 | Q1 held | `to-faithful-clojure-{rete,net}` fixtures replay; `probe_arc278_7strat_native_differential` green | green |
| E8 | rete behaviour survived the sweeps | the shared steps' named tests (#153 #155 #157 #159) | green |
| E9 | the stdlib steps | two-phase; no `UNREGISTERABLE wat/` (STOP-9) | none |
| E10 | the checkpoint | the floor at #159 + the orchestrator's own | green; clippy 0 |
| E11 | no knowingly-red commit | no repair commit after #159 | none |

**Runtime prediction:** 1.5–2.5 h — four sweeps of 25–205 files, five ports with fixtures, one convert.sh
rule, one checkpoint floor.

**Trap doors:** the sweep size (convert.sh over ~200 paths; merge-file per file); a string-literal pattern
the chain does not reach (P1 step 2); a sweep file with a hand touch chosen as a fixture (E6).
