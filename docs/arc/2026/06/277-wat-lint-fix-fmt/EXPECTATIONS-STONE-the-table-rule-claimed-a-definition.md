# EXPECTATIONS — STONE: the table rule claimed a definition, and the file lost its end

Every row names an **exact value**, not a direction. "Fewer over-120 lines" is a row the defect
satisfies; `over120=0` is not. `[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

| # | what | command | expected — EXACT |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★★ **`wat/deporder.wat`** | `277-file-width-census.wat` | `over120=0` · `worst<=120` |
| 3 | ★★★ **`wat/grep.wat`** | same | `over120=0` · `worst<=120` |
| 4 | ★★★ **`wat/spawn.wat`** | same | `over120=0` · `worst<=120` |
| 5 | ★★★ **`wat/fmt.wat` — the 1649-column file** | same | `over120=0` · `worst<=120` |
| 6 | ★★ **`wat/io.wat` must not regress** | same | `over120=0` · `worst=104` — unchanged |
| 7 | ★★★ **the last two bytes of every formatted file** | `tail -c 2 \| xxd -p` | `29 0a` — `)` then ONE newline. **Not `0a0a`, and not a bare `29`.** |
| 8 | ★★★ **an isolated one-line example** | format `(:wat::core::mapv …)` alone | ends `))\n` — **exactly one** trailing newline |
| 9 | ★★★ **the 614 doc examples** | `run-examples.wat` | `OVER120=0` · `WORST<=120` · **`INLINE` returns to 284** (row 8's consequence) |
| 10 | ★★★ **A GENUINE TABLE THAT DOES NOT FIT FALLS BACK** | new fixture: 2+ adjacent same-head **non-definition** forms whose aligned group exceeds 120 | each member formats **by its own rules**, unaligned. **This is the only row that proves the fit test FIRES; without it a no-op fit test passes rows 2-6.** |
| 11 | ★★★ **every genuine table that DOES fit still aligns** | `kw-table.wat` · `pair-table.wat` · **`pos-table.wat`** | all three unchanged. `kw-table` keeps `GROUPS2=1` and its three `Node` forms one line each. ⚠ **`pos-table` is the positional (`#N`) case — the SAME key-sequence shape that catches the `defn`s — so it is the fixture the guard is most likely to take with it.** |
| 12 | ★★ **definition forms are no longer claimed** | `deporder`'s 8 top-level `defn`s | each renders in the **ruled `defn` shape** — name on the head line, arg-spec one per line, ret-spec its own line. **Not merely "shorter".** |
| 13 | ★★ **`defenum` tag padding survives** | `defenum-mixed` · `defenum-spawn-shape` | tags still padded, `[]` still inserted, **not duplicated** — this is `AlignPairs`, not `table`; the guard must not reach it |
| 14 | ★★★ **no comment is LOST** | reader count, source and output | `io.wat` **28** · `deporder.wat` **85** · `spawn.wat` **429** — both sides |
| 15 | ★★ **idempotent** | every fixture + all four real files | `IDEMPOTENT=true` |
| 16 | ★★ every fixture still `--check` clean | `wat --check` | clean, including the new row-10 fixture |
| 16b | ★★★ **the guard is STRUCTURAL, not a head list** | `grep -c 'string::not= ?h' wat-scripts/fmt/rules/table.wat` | **`0`**. A list of definition heads is the REJECTED design — it left `spawn` at 75 because it missed `defservice`, and the corpus has twelve such heads at column 0. |
| 17 | walls stand | kind-conflict sabotage · `ClaimedUnder` · `grep -c 'col'` over rules | raises · `0` · `0` |
| 18 | wat-scripts load | `every_wat_scripts_file_loads` | `1 passed` |
| 19 | floor (ORCHESTRATOR) | `scripts/floor.sh` | `5179+` run, **0 FAILED** |
| 20 | clippy (ORCHESTRATOR) | `-D warnings --all-targets` | `0` |

**Runtime prediction:** 40-70 min. The guards are five `:where` lines of an idiom `defrecord.wat`
already uses; the fit test and the `emit` word are each small. Row 10's fixture is the real work.

## Trap-doors named in advance

- ⚠ **`wat/*.wat` IS FROZEN INTO THE RELEASE BINARY.** Editing `wat/fmt.wat` and re-running changes
  **nothing** without `cargo build --release`. Three sabotages in this arc were read as evidence
  before this was established, two of them self-consistent renames that could not have failed.
  **Rule files under `wat-scripts/` ARE read from disk** — a bogus `load-file!` path raises.
- **Row 10 is the row this stone dies on.** Rows 2-6 all pass with a fit test that never fires,
  because the definition-head guard alone already takes `deporder` to 0. Only a non-definition group
  that overflows can tell a working fit test from a no-op.
- **Row 9's `INLINE` is a RETURN to 284, not an increase.** It was 284 before rule 3 reached the last
  form and 0 after. A number between them means the trailing newline is still wrong.
- **Row 7 has three outcomes, not two.** `0a0a` is today's bug; a bare `29` with no newline is the
  overcorrection. Only `29 0a` passes.
- **Row 12 cannot be scored by width.** A claimed `defn` that merely got shorter still failed. Read
  the shape.
- **Row 16b is the design, not a style preference.** The head-list version was measured and is
  strictly worse: `spawn` 75 vs 9. And the registry cannot substitute — `defn`/`defrecord`/
  `defstruct`/`deftest` are wat-level macros with **0 registry rows**.
- **Row 13 is the guard's blast radius.** `defenum`'s padding comes from `AlignPairs`, and `kwargs`
  already excludes `defrecord`/`defstruct`/`defenum` heads. A guard written too wide takes the enum
  work with it.
