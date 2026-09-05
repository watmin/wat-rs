# EXPECTATIONS — STONE: indent is STRUCTURAL

Fixed BEFORE the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★ **NO RULE READS A COLUMN** | `grep -c 'col' wat-scripts/fmt/rules/*.wat` | **0 in every file** — the wall, and it is greppable |
| 3 | no rule does indent arithmetic | `grep -c ':indent' wat-scripts/fmt/rules/*.wat` | **0** |
| 4 | ★ the four-rule composition | `run-all.wat` on `fixtures/all-four.wat` | `match` arms indent under the `match`, **not at column 67** |
| 5 | ★ **that fixture is IDEMPOTENT** | same run | `IDEMPOTENT=true` |
| 6 | R1 unchanged in effect | `run.wat` on `defn-multi.wat` | arg-spec own line, one arg per line, ret own line, body own line |
| 7 | R1 empty argspec | `defn-empty.wat` | `[]` on its own line |
| 8 | R3 `let` | `run-let.wat` on `let-two.wat` | head line bare; one binder per line; body after binders; `IDEMPOTENT=true` |
| 9 | R4 `match` | `run-r4.wat` on `half-broken.wat` | scrutinee on the head line, one arm per line, `IDEMPOTENT=true` |
| 10 | R11 default still reaches top-level | `run-r4.wat` on `unruled-top.wat` | the `do` breaks one child per line |
| 11 | comments still survive | `run.wat` on `wat/io.wat` | `COMMENTS=28`, same order, count printed |
| 12 | wat-scripts load | `nextest -E 'test(every_wat_scripts_file_loads)'` | 1 passed |
| 13 | floor (ORCHESTRATOR) | `scripts/floor.sh` | 5179+ run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | `cargo clippy --release --all-targets -- -D warnings` | 0 |

**Runtime prediction:** 40-70 min. The rule edits are mechanical; the emitter's `:align` bookkeeping
is the real work.

## Trap-doors named in advance

- **Row 2 is the point of the stone.** A green row 4 with a column still readable in a rule is a
  green built on sand — the defect would be absent, not unrepresentable.
- **`:align` needs the EMITTED column of the opening delimiter**, not the source's. If it is derived
  from `out`'s tail, a container opened mid-line and one opened after a break must both work.
- **Idempotence usually breaks on pass 2** where an indent is computed from already-indented text.
  With columns gone this SHOULD become structurally impossible — row 5 is the test of that claim.
- **Comments are a legitimate span consumer.** Rows 2/3 forbid columns for INDENT only;
  `extent-of` must keep working for comment placement (row 11).
- **The vacuous green.** Row 11 prints the comment COUNT because a preservation pass over zero
  comments is indistinguishable from success — published once this session.
- **`\;` is a char literal, not a comment.**
