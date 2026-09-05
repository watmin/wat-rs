# EXPECTATIONS — STONE: conflicting Breaks raise

| # | what | command | expected |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★ **THE WALL FIRES ON DISAGREEMENT** | throwaway rule asserting `"align"` for a node another rule breaks `"block"` | **raises**, naming the node id AND both kinds. Then delete the rule. |
| 3 | ★★ **AGREEMENT STAYS SILENT** | throwaway rule asserting the SAME kind for an already-broken node | **no raise**, output unchanged. Then delete the rule. |
| 4 | the wall is invisible to correct rule sets | every existing fixture | unchanged output, idempotent |
| 5 | `claims-set` untouched | `git diff wat/fmt.wat` | no change inside `claims-set` |
| 6 | no rule file changed | `git diff wat-scripts/fmt/rules/` | **EMPTY** |
| 7 | R11 still all-or-nothing | `git diff` | no always-break change |
| 8 | no `BlankBefore` | `grep -c BlankBefore wat/fmt.wat` | 0 |
| 9 | comments survive | `run.wat` on `wat/io.wat` | **COMMENTS=28**, count printed |
| 10 | the previous two walls hold | `grep -c ClaimedUnder` / `grep -c 'col'` over rules | 0 / all 0 |
| 11 | wat-scripts load | `nextest -E 'test(every_wat_scripts_file_loads)'` | 1 passed |
| 12 | floor (ORCHESTRATOR) | `scripts/floor.sh` | 5179+ run, **0 FAILED** |
| 13 | clippy (ORCHESTRATOR) | `--all-targets -D warnings` | 0 |

**Runtime prediction:** 15-30 min. One function.

## Trap-doors named in advance

- **Rows 2 AND 3 together are the stone.** Row 2 alone is achievable by raising on any duplicate —
  which fails row 3 and breaks extensibility. A strike reporting row 2 green without row 3 has not
  done the work.
- **Row 4 matters because the wall must be invisible when nothing is wrong.** If an existing fixture
  starts raising, STOP-4 applies: that is a real finding about the current rule set, not a reason to
  soften the wall.
- **Both sabotages must be DELETED after they fire.** A probe rule left in `rules/` is loaded by
  `collect-rules :fmt` and would change every later run.
- **The vacuous green.** Row 9 prints the comment COUNT — a preservation pass over zero comments is
  indistinguishable from success, and that was published once this session.
