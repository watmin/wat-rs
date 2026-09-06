# EXPECTATIONS — STONE: Break.kind becomes an enum

Every row names an **exact value or an exact shape**, never a direction.
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

⚠ And the mirror of that, which cost this arc a row two stones ago: **no row here demands a number
the DESIGN's own measurements say is unreachable.** Formatting output must be BYTE-IDENTICAL — this
stone changes a representation, not a layout.

| # | what | command | expected — EXACT |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★★ **the enum exists and has exactly two variants** | read `wat/fmt.wat` | `:wat::fmt::BreakKind` · `:wat::enum::Pure` · `:Block []` · `:Align []` · **no `:Unknown`** |
| 3 | ★★★ **no `kind` STRING literal survives in the rules** | `grep -rc '"block"\|"align"' wat-scripts/fmt/rules/` | **`0`** across all 12 files |
| 4 | ★★★ **all 23 assert sites carry a VARIANT** | `grep -rc 'fmt::Break :id' wat-scripts/fmt/rules/` | **`23`**, every one `:kind (:wat::fmt::BreakKind::…)` |
| 5 | ★★★ **`pad-break` is an EXHAUSTIVE MATCH with NO else arm** | read `fmt.wat` | the `assertion-failed! "Break.kind must be block or align"` is **GONE**, replaced by a two-arm `match` — **not** an `if` with a raise |
| 6 | ★★★ **and the wall it replaced is now a COMPILE error** | delete the `:Align` arm, `wat --check` | `non-exhaustive: enum :wat::fmt::BreakKind missing arm(s) for variant(s): Align`. **Restore after.** This is the row that proves the ladder was climbed, not the check merely deleted. |
| 7 | ★★★ **the CONFLICTING-Breaks wall still FIRES** | the existing kind-conflict sabotage | still raises `fmt: conflicting Breaks for node N — … vs …`, now naming two VARIANTS. **A migration that silently drops this wall passes every other row.** |
| 8 | ★★★ **`breaks-map`'s value type is the enum** | read `fmt.wat` | `HashMap :- [i64 :wat::fmt::BreakKind]` — **not** `String` |
| 9 | ★★★ **THE OUTPUT IS BYTE-IDENTICAL** | the orchestrator captured the BEFORE state at draw time (one `println` per formatted line, so no EDN decode sits between the emitter and the digest) | these exact digests, unchanged: `deporder c69f0460e9f91672` · `fmt.wat 4fbb59de01302a92` · `grep 1dce84a9077037da` · `io ba89b65cbd61abc6` · `spawn 1c4bbb19bee10093`. **A representation change that moves one column is a defect.** |
| 10 | ★★ the width numbers are unchanged | `277-file-width-census.wat` | `deporder` **0/102** · `grep` **0/98** · `spawn` **9/174** · `fmt.wat` **2/147** · `io` **0/104** |
| 11 | ★★ the 614 doc examples unchanged | `run-examples.wat /tmp/fmt-614.wat` | `N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104` |
| 12 | ★★ file endings unchanged | `277-file-ends-with.wat` | `trailing-empty-parts=1` on every file |
| 13 | ★★ no comment lost | `277-comments-both-sides.wat` | `28/28` · `85/85` · `429/429` · `152/152` · `47/47` |
| 14 | ★★ idempotent | every fixture + the five real files | `IDEMPOTENT=true` |
| 15 | ★★★ **it is a RECORDED CODEMOD, not hand edits** | `ls wat-scripts/fixes/` | a new `.wat` codemod, committed, **idempotent — a second run reports 0 changes** |
| 16 | every fixture `--check` clean | `wat --check` | clean |
| 17 | walls / rules hygiene | `ClaimedUnder` · `'col'` in rules · `'120'` in rules | `0` · `0` · `0` |
| 18 | wat-scripts load | `every_wat_scripts_file_loads` | `1 passed` |
| 19 | floor (ORCHESTRATOR) | `scripts/floor.sh` | `5179+` run, **0 FAILED** |
| 20 | clippy (ORCHESTRATOR) | `-D warnings --all-targets` | `0` |

**Runtime prediction:** 45-75 min. The enum and `pad-break` are small; the codemod and its dry-run
are the work.

## Trap-doors named in advance

- **Row 9 is the row this stone is FOR.** Rows 2-8 can all pass while the output silently moves. This
  is a representation change; a single column of drift is a failure, not a rounding.
- **Row 7 is the row a migration silently loses.** The unknown-kind wall is RETIRED into
  exhaustiveness (row 6); the conflicting-Breaks wall at `fmt.wat:995` is a DIFFERENT invariant and
  must keep firing. Deleting both passes rows 2-6.
- **Row 6 is what separates "climbed the ladder" from "deleted the check."** Removing the raise and
  writing an `if`/`else` that silently picks `block` passes row 5's letter and fails its purpose.
- **Row 15 — a hand edit passes rows 2-14.** R21 is not a style preference: the codemod is the
  artifact, and a second run reporting 0 changes is what proves it idempotent.
- ⚠ **`wat/*.wat` is FROZEN into the release binary.** `cargo build --release` between an emitter
  edit and any measurement. Rule files under `wat-scripts/` are read from disk.
- **Expect a wide red mid-migration.** A `String` field and an enum field are different TYPES, so the
  checker is the cascade — the fail count is the progress meter, not a crisis
  (`docs/SUBSTRATE-AS-TEACHER.md`).
