# EXPECTATIONS 6 — replay batch 3, grok-rete #126 → #152 (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 27 replayed steps, in order | `REPLAY(grok-rete #` count and each `-x` trailer against `commits.tsv` | 27, #126 → #152, contiguous |
| E2 | docs-only steps are plain cherry-picks | the 14 (#127 #130 #132 #133 #135 #136 #138 #141 #145 #146 #147 #148 #150 #152): `git show --stat` | only `docs/`/`.md` |
| E3 | every produced `.wat` passes `--check` | `./target/release/wat --check` on each | rc 0 |
| E4 | #126 re-expressed on main's homes | its diff: `src/edn/render.rs` + `src/string/mod.rs` carry grok's change; no `src/edn_shim.rs` / `src/string_ops.rs` resurrected | as stated; C's named tests (`probe_edn_write_unencodable_is_a_diagnostic` …) green |
| E5 | rete behaviour survived | the shared steps' named tests (#126 #131 #137 #142 #143 #149 #151) | green |
| E6 | the checkpoints | floors at #139 and #152 + the orchestrator's own at #152 | green; clippy 0 |
| E7 | the per-step walls ran | the log: census + stone-3 gate on the 11 qualifying steps (#126 #128 #134 #137 #139 #140 #142 #143 #144 #149 #151); lint subset + lib unit tests + doctests on the 10 `.rs` steps (#126 #131 #137 #139 #140 #142 #143 #144 #149 #151) | every row present; no STOP-8/10/11 |
| E8 | no hazard in range | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` (other than #126's two moved files) | none |
| E9 | no knowingly-red commit | no repair commit after #152; any composition repair folded into its step | none after #152 |

**Runtime prediction:** ~1 h (13 code steps, #126 the large one; 14 docs steps; two checkpoint floors).

**Trap doors:** #126 (31 files; two moved homes); #151 (14 files); #143 (10).
