# EXPECTATIONS — STONE: a rule owns ONE node's children

Fixed BEFORE the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★ **HORN A dies** | `run-all.wat` on `fixtures/claim-demo.wat` | binders keep name-with-value on one line; **`IDEMPOTENT=true`** |
| 3 | ★★ **HORN B dies** | `run-all.wat` on `fixtures/unruled-inside-defn.wat` | the `do` **BREAKS** one child per line, inside the `defn` |
| 4 | `ClaimedUnder` is gone | `grep -c ClaimedUnder wat/fmt.wat wat-scripts/fmt/rules/*.wat` | **0 everywhere** |
| 5 | ★★★ **THE WALL FIRES** | add a throwaway rule asserting a `Break` for a GRANDCHILD; run | **raises**, naming the offending node. Then delete the rule. |
| 6 | R1 unchanged in effect | `run.wat` on `defn-multi.wat` / `defn-empty.wat` | ruled shape, idempotent |
| 7 | R3 unchanged in effect | `run-let.wat` on `let-two.wat` | head bare, one binder per line, body after; idempotent |
| 8 | R4 unchanged in effect | `run-r4.wat` on `half-broken.wat` | scrutinee rides, one arm per line; idempotent |
| 9 | four-rule composition | `run-all.wat` on `all-four.wat` | ruled shape; **`IDEMPOTENT=true`** |
| 10 | the default still reaches top level | `run-all.wat` on `unruled-top.wat` | the `do` breaks |
| 11 | comments survive | `run.wat` on `wat/io.wat` | **`COMMENTS=28`**, count printed |
| 12 | no rule reads a column | `grep -c 'col' wat-scripts/fmt/rules/*.wat` | **0 in every file** — last stone's wall must hold |
| 13 | wat-scripts load | `nextest -E 'test(every_wat_scripts_file_loads)'` | 1 passed |
| 14 | floor (ORCHESTRATOR) | `scripts/floor.sh` | 5179+ run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | `--all-targets -D warnings` | 0 |

**Runtime prediction:** 40-70 min. The splits are mechanical; the wall is the real work.

## Trap-doors named in advance

- **Rows 2 and 3 must BOTH pass.** Either alone is achievable by picking a gate — that is the whole
  dilemma. Only the ownership ruling gets both, and a strike that reports one green and the other
  unmentioned has not done the stone.
- **Row 5 is a sabotage, not a review.** Every wall armed in this campaign was shown firing before it
  was trusted. A wall asserted-but-never-fired is a convention with better paperwork.
- **Row 12 guards the PREVIOUS stone.** Splitting a rule is exactly when a `?pc` sneaks back in to
  "just get the indent right".
- **The vacuous green.** Row 11 prints the comment COUNT because a preservation pass over zero
  comments is indistinguishable from success — published once this session.
- **Idempotence breaks on pass 2**, never pass 1. Rows 2, 7, 8, 9 all say idempotent for that reason.
- **`\;` is a char literal, not a comment.**
