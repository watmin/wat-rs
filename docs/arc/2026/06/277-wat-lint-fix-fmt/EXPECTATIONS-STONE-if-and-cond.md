# EXPECTATIONS — STONE: `if`'s measurement rides, and `cond`'s clauses align

⚠ **This stone MOVES the formatter's output on purpose.** No row demands byte-identity — that would
be a bar the stone cannot clear, which this arc has now written twice.
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

| # | what | expected — EXACT SHAPE, not "shorter" |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **`if`'s measurement RIDES** | `(:wat::core::if (:wat::i64::> x 0)` on ONE line. **Not the test on its own line; not all three children riding.** |
| 3 | ★★★ **and each branch takes its own line, one level in** | `(:wat::i64::+ x 1)` and `(:wat::i64::- x 1)` on separate lines at the `if`'s indent **+2** |
| 4 | ★★★ **`cond`'s body RIDES its test** | `((:wat::i64::= k 0) "zero")` on ONE line |
| 5 | ★★★ **and the bodies ALIGN across the clauses** | `"zero"`, `"one"` and `"many"` start at the **same column**, `:else` padded to the widest test. **Name the column in the SCORE.** |
| 6 | ★★★ **alignment obeys 120** | a fixture whose aligned cond exceeds 120 → **each clause by its own rules, unaligned**. **This is the only row that proves the fit test fires; rows 4-5 pass with no fit test at all.** |
| 7 | ★★★ **A NEW RULE IS A NEW FILE** | `git diff --stat` touches **no** existing rule file and **not** `wat/fmt.wat`. Two new files, plus the driver's `load-file!` lines. **The arc's acceptance criterion, proven 5×.** |
| 8 | ★★★ **width does NOT regress on the five real files** | `deporder 0/102` · `grep 0/98` · `spawn 9/174` · `fmt 2/147` · `io 0/104` — **over120 no higher and worst no higher than these**. Better is fine. |
| 9 | ★★★ **the 614 doc examples** | `OVER120=0` · `WORST<=120`. **6 of them contain an if or cond; the other 608 must not move.** |
| 10 | ★★ idempotent | every fixture and all five real files `IDEMPOTENT=true` |
| 11 | ★★★ **no comment is LOST** | `io 28` · `deporder 85` · `spawn 429` · `grep 157` · `fmt 45` — both sides |
| 12 | ★★ the ruled shapes still hold | `defn` · `let` · `match` · `defrecord` · `defenum` · kwargs · tables · atoms — unchanged |
| 13 | ★★ `cond`'s existing inconsistency is GONE | `(:else "many")` and `((:wat::i64::= k 0) "zero")` now take the **same** shape. Today the atoms rule rides one and explodes the other. |
| 14 | ★★ files still end on one newline | `trailing-empty-parts=1` |
| 15 | ★★ the walls stand | kind-conflict sabotage raises · `ClaimedUnder` **0** · `'col'` in rules **0** · `'120'` in rules **0** |
| 16 | ★★ wat-scripts load | `every_wat_scripts_file_loads` `1 passed` |
| 17 | floor (ORCHESTRATOR) | `5189+` run, **0 FAILED** |
| 18 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 40-70 min. Two rule files of the shape `match.wat` already has; row 6's
overflow fixture is the real work.

## Trap-doors named in advance

- **Row 6 is the row this stone dies on.** Rows 4 and 5 pass perfectly with a cond rule that always
  aligns — and then a wide cond produces the 240-character padding runs the table stone was built to
  remove. Build the overflowing fixture FIRST and confirm it aligns badly BEFORE the fit test exists.
- **Row 7 is the arc's whole thesis.** If `if` or `cond` needs `wat/fmt.wat` edited, the finding is
  that the rule language is short of something — **say which, and stop.** Do not quietly widen the
  emitter.
- **Row 8's baseline is a NON-REGRESSION, not a target.** `spawn 9/174` and `fmt 2/147` are the
  residues the table stone measured and named — 5 over-long comments, 2 `assertion-failed!` string
  literals, 2 unruled `defsurface` lines, and two 17-positional-arg `emit-node` calls. **Do not chase
  them; they are not this stone's.**
- **Row 5 needs a COLUMN, not a claim.** "Aligned" is satisfiable by output that merely looks tidy.
  Name the column the bodies start at, and show two clauses of different test width sharing it.
  `[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`
- **`cond` has NO subject.** Its form is `(cond (test body) … (:else body))` — clauses only, a
  required terminal `(:else body)`, and it is a MACRO (`wat/core.wat:1455`) over `if`. A rule written
  for `(cond <subject> …)` is written for `match`, which already has one.
- ⚠ **A rule file is not loaded until a driver `load-file!`s it** — the trap this arc has hit before.
  `run-all.wat` needs both lines or the rules are inert and every row silently measures the old
  behaviour.
