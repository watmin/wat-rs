# EXPECTATIONS 7b — replay batch 4b, grok-rete #160 → #211 (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 52 replayed steps, in order | `REPLAY(grok-rete #` count + trailers vs `commits.tsv` | 52, #160 → #211, contiguous |
| E2 | docs-only steps are plain cherry-picks | the 23 docs steps: `git show --stat` | only `docs/`/`.md` |
| E3 | every produced `.wat` passes `--check` | `./target/release/wat --check` on each | rc 0 |
| E4 | the step record is complete | `scripts/replay/verify-step-record.sh <start> HEAD` | exit 0 |
| E5 | rete behaviour survived | the shared steps' named tests (#164 #165 #166 #167 #168 #169 #171 #174 #175 #176 #177 #184 #193 #199 #207) | green |
| E6 | the checkpoints | floors at #185 and #211 + the orchestrator's own at #211 | green; clippy 0 |
| E7 | a spot re-run of the walls | the orchestrator re-runs lint subset + `kind(lib)` + doctests + stone 3's gate at 3 steps it picks | clean |
| E8 | no hazard in range | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | none |
| E9 | no knowingly-red commit | no repair commit after #211 | none |

**Runtime prediction:** ~1.5–2 h (29 code steps at ~2–3.5 min with the walls; 23 docs steps; two
checkpoint floors).

**Trap doors:** #176 and #184 (19 files each), #177 (16).
