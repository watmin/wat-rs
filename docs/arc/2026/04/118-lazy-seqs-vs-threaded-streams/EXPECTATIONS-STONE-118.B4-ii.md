# EXPECTATIONS — STONE 118.B4-ii · the `(first (drop X n))` codemod

Written before the strike. The scorecard cannot move.

| # | what | command | expected |
|---|---|---|---|
| 1 | census before | the census, 13 paths | **44** hits, 0 malformed |
| 2 | ★ census after | the census, 13 paths | **0** |
| 3 | ★ idempotent | second codemod run + `git diff` | **no diff** |
| 4 | dry-run diff is structural only | `/tmp` copy + `diff` | only `first`/`drop`/paren tokens move; operand text byte-identical |
| 5 | stdlib rebaked | `cargo build --release` after apply | clean |
| 6 | floor | `scripts/floor.sh` | **≥4756 run, 0 FAIL, 19 skipped** |
| 7 | clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
| 8 | ignores unchanged | the ignore census | 13 |
| 9 | loader gate holds | floor (`every_wat_scripts_file_loads`) | the 8 `wat-scripts/` files still load |
| 10 | blast radius held | `git status --short` | 13 census files + `wat/fix.wat` + 1 new fix script |

**Rows 2 and 3 are load-bearing together.** Row 2 alone would pass on a codemod that mangles the
corpus into something that no longer matches the census pattern. Row 3 alone would pass on a codemod
that does nothing. Only both, plus row 6, say the migration is real and complete.

## Independent prediction

**40–60 minutes.** The generic helper is the work; the wrapper and the run are mechanical. Two
rebuilds (~35s each) and two floors are most of the wall-clock.

## Trap-doors named in advance

- **Forgetting step 5** (rebuild after apply) makes every subsequent measurement a lie — the binary
  would still be running the pre-codemod stdlib. Any "the census says 0 but the floor is green"
  result with only one build in the log is this trap.
- **`wat/fix.wat` rewriting itself** is expected and documented in its own header. A rider that
  excludes it to feel safe has left 5 sites for B4-iii to trip over.
- **The nil→raise edge is the whole point of the migration and also its only real risk.** STOP-1
  exists because a silent behaviour change here would be indistinguishable from a correct one until
  something far away broke.
- **`wat-scripts/probes/arc-170/*` are old probes** (17 of the 44 sites). They are loader-gated, so
  they must still load, but their *content* is historical. Migrate them; do not "improve" them.
