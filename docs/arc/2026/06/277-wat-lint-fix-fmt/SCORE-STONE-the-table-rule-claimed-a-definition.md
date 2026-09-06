# SCORE — STONE: the table rule claimed a definition, and the file lost its end

No commit. Floor and clippy left to the orchestrator. No Rust. No new record. No new rule file. `if` / `cond` / defect E untouched (STOP-5).

`table-start` no longer fires at parent 0. A group whose aligned width exceeds 120 is not padded — each member formats by its own rules. `emit` ends the file with one newline, not a blank.

## Rows 2–6 — census (`277-file-width-census.wat`)

| file | SOURCE | FORMATTED | expected |
|---|---|---|---|
| `wat/deporder.wat` | 3 / 306 | **over120=0 · worst=102** | 0 · ≤120 |
| `wat/grep.wat` | 1 / 121 | **over120=0 · worst=98** | 0 · ≤120 |
| `wat/spawn.wat` | 19 / 227 | over120=**9** · worst=**174** | 0 · ≤120 |
| `wat/fmt.wat` | 4 / 191 | over120=**2** · worst=**147** | 0 · ≤120 |
| `wat/io.wat` | 1 / 122 | **over120=0 · worst=104** | 0 · 104 |

deporder / grep / io hit the exact row. spawn 9/174 and fmt 2/147 are the DESIGN's **"no table at all"** column (spawn 9/174, fmt 2/147) — the fit test ate fmt's 1430-column nested group (16/1430 → 2/147) and grep's 141-column line (1/141 → 0/98). The residue is **not table padding**:

**spawn, 9 lines:** 5 are already-long `;;` comments (96, 97, 98, 289, 701); 2 are `assertion-failed!` string literals (643 w=172, 717 w=174); 2 are unruled `defsurface` / method-type lines (465, 495). Wrapping comments is defect E (STOP-5). `if`/`cond` are the next stone.

**fmt.wat, 2 lines:** both `emit-node` calls with 17 positional args (1088 w=147, 1369 w=141), inside a claimed `defn` so R11 does not explode them.

A five-head list is the rejected design. `grep -c 'string::not= ?h' wat-scripts/fmt/rules/table.wat` = **0**. The guard is `(:wat::rete::i64::not= ?p 0)`.

## Row 7 — last two bytes `29 0a`

`tail -c 2 | xxd -p` on formatted deporder, grep, spawn, fmt, io, comment-indent, table-overflow, and every other fixture: **`290a`**. `)` then ONE newline. Not `0a0a`, not a bare `29`.

## Row 8 — isolated example, one trailing newline

`(:wat::core::mapv …)` alone: R11 explodes the `fn`, last line `[1 2 3])`, **`tail -c 2` = `290a`**. Not a blank after the last form.

## Row 9 — 614 doc examples

`N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104`.

**INLINE returns to 284.** OVER120 still 0, worst still 104. CHANGED stays 614 because the source one-liners are still rewritten (layout), not because of a trailing blank.

## Row 10 — ★★★ a genuine table that does not fit FALLS BACK

Fixture `table-overflow.wat` built first: GROUPS2=1, both Unreadable rows **121** with `:reason` padded (`"one"` stretched to the long concat). `--check` clean.

After the fit test, `IDEMPOTENT=true`, no line over 120, each member by kwargs, unaligned:

```
  (:wat::core::do
    (:wat::grep::Unreadable
      :file   p
      :reason (:wat::string::concat p "one")
      :line   0
      :col    0)
    (:wat::grep::Unreadable
      :file   p
      :reason (:wat::string::concat p "two-much-longer-reason-string-XXXX")
      :line   1
      :col    1)
    nil))
```

No cross-row pad. This is the only row that proves the fit test fires.

## Row 11 — genuine tables that fit still align

| fixture | GROUPS2 | GROUPS3 | ROWS | shape |
|---|---|---|---|---|
| `kw-table.wat` | **1** | **1** | **3** | three `Node` one-liners |
| `pos-table.wat` | **1** | 0 | **2** | `"short"` pads to `"much-longer"` |
| `pair-table.wat` | **1** | 0 | **2** | aligned |

`pos-table` is the positional (`#N`) key-sequence — the same shape that was catching top-level `defn`s. The guard keyed on parent, not keys, so it survived.

## Row 12 — definition forms are no longer claimed

`deporder` GROUPS2=**0** ROWS=**0** (table-extend has nothing to extend). `collect-kwds` is the ruled `defn` shape, not a 352-column padded one-liner:

```
(:wat::core::defn :wat::deporder::collect-kwds
  [node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if
    …
```

Name on the head line, arg-spec its own line, ret-spec its own line. All 15 top-level `defn`s.

## Row 13 — defenum tag padding survives

`defenum-mixed` / `defenum-spawn-shape` unchanged: tags padded, `:Shutdown []` inserted, not duplicated. AlignPairs, not table.

## Row 14 — no comment LOST

| file | source | formatted |
|---|---|---|
| `wat/io.wat` | **28** | **28** |
| `wat/deporder.wat` | **85** | **85** |
| `wat/spawn.wat` | **429** | **429** |

grep 152/152, fmt.wat 47/47.

## Row 15 — idempotent

Every fixture `IDEMPOTENT=true`. Real files: deporder, grep, spawn, fmt, io all `IDEMPOTENT=true`.

## Row 16 — `--check` clean

Every fixture including `table-overflow.wat`.

## Row 16b — the guard is STRUCTURAL

`grep -c 'string::not= ?h' wat-scripts/fmt/rules/table.wat` → **0**. One `:where`: `i64::not= ?p 0`.

## Rows 17–18 — walls / load

Disagreeing-kind sabotage raises `fmt: conflicting Breaks for node 11 — align vs block`. Deleted after. `ClaimedUnder` **0**. `grep -c 'col'` / `'120'` over rules **0**. `every_wat_scripts_file_loads` **1 passed**.

## Mechanism

- `table-start`: `(:wat::rete::where (:wat::rete::i64::not= ?p 0))`. Data is never top-level. `table-extend` keys off an existing `TableRow` — verified: deporder GROUPS2=0.
- Fit test lives in the emitter (same place as `AllAtoms` + `indent + Width <= 120`). `table-row-fits?` computes `indent + '(' + Σ columns + spaces + ')'` against 120. Fail → skip-kids false, no `tblw` pad, child Breaks restored. Never truncate, never re-break. `table.wat` still names no width. `120` in `fmt.wat` now twice (atoms + table).
- `emit`: `ensure-blank` after the fold → `write-nl`. Between top-level forms, `ensure-blank` still inserts the ruled blank.

---

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| census deporder / grep / io | **0 / 102**, **0 / 98**, **0 / 104** |
| census spawn / fmt.wat | 9 / 174 · 2 / 147 — DESIGN "no table" tail |
| `tail -c 2 \| xxd -p` | **`290a`** on every formatted file |
| 614 | **INLINE=284** · OVER120=0 · WORST=104 |
| table-overflow before fit | GROUPS2=1, both rows **121** padded |
| table-overflow after | kwargs fallback, no line over 120 |
| kw / pos / pair tables | GROUPS2=1, shapes unchanged |
| deporder GROUPS2 | **0** |
| `grep -c 'string::not= ?h' table.wat` | **0** |
| io / deporder / spawn comments | **28 / 85 / 429** both sides |
| kind-conflict sabotage | **raises**, then deleted |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** Every row re-run independently. `wat-fmt` no longer makes a real file worse
than it found it.

| what | my own re-run |
|---|---|
| ★★★ rows 2 / 3 / 6 | `deporder` **0 / 102** · `grep` **0 / 98** · `io` **0 / 104** — the exact rows |
| ★★★ row 7 · the file's end | `trailing-empty-parts=1` on every file. **Control: a source ending `0a0a` reads 2**, so the probe discriminates |
| ★★★ row 8 · isolated example | ends `))` + ONE newline |
| ★★★ row 9 · the 614 | `N=614 CHANGED=614 **INLINE=284** OVER120=0 WORST=104` — INLINE *returned* to 284 |
| ★★★ row 10 · the fit test FIRES | see below — the row this stone could have died on |
| ★★★ row 11 · tables that fit still align | `pos-table` **GROUPS2=1 ROWS=2**, `"short"` still padded to `"much-longer"` |
| ★★ row 12 · definitions unclaimed | `deporder` **GROUPS2=0 ROWS=0**; all 15 top-level `defn`s in the ruled shape |
| ★★★ row 14 · no comment LOST | **28/28 · 85/85 · 429/429 · 152/152 · 47/47**, counted on BOTH sides by `read-string-with-comments` |
| ★★ row 16b · the guard is structural | `grep -c 'string::not= ?h'` → **0**; the guard is `(:wat::rete::i64::not= ?p 0)` |
| row 17 · walls | `ClaimedUnder` **0** · `'col'` in rules **0** · `'120'` in rules **0** |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy | **0** under `-D warnings --all-targets` |

### ★ Row 10 — the discriminator, because unaligned output alone proves nothing

An unaligned `table-overflow` is equally consistent with *the fit test fired* and with *the group
never formed*. Separated:

```
table-overflow    GROUPS2=1 ROWS=2   the group FORMED …  and the output is UNALIGNED
pos-table         GROUPS2=1 ROWS=2   fits → still aligns
wat/io.wat        GROUPS2=0 ROWS=0   the probe can see a no-group file
```

The rule groups the members and the **emitter** declines to pad. That is the fit test, firing.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

## ⛔ ROWS 4 AND 5 WERE NOT MET, AND THE DEFECT IS THE ROW

The scorecard demanded `over120=0` for `wat/spawn.wat` and `wat/fmt.wat`. The strike delivered
**9 / 174** and **2 / 147**.

**Those are exactly the "no table at all" column of this stone's own DESIGN**, written before the
scorecard was. I asked for a number my own measurement said the stone could not reach, and the
strike hit the reachable floor precisely.

The residue is named and none of it is table padding: 5 already-long `;;` comments (defect E, which
STOP-5 forbade the strike to touch), 2 `assertion-failed!` string literals, 2 unruled `defsurface`
lines, and two `emit-node` calls carrying 17 positional arguments.

★ **This is the session's failure pattern wearing its other face.** Four times this arc I wrote a row
the defect could satisfy; here I wrote a row *nothing* could satisfy. Both come from deriving the bar
from what I expected instead of from the measurement already on the page.
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## Two corrections to the strike's own numbers

- `120` appears **three** times in `wat/fmt.wat`, not twice: `626` (a comment), `657` (the table fit
  test), `902` (the atoms one).
- Row 14's claim was correct but the fmt drivers print the **source-side** count only, so "no comment
  is lost" cannot be read off them. Re-read through `read-string-with-comments` on both sides.

## The board this leaves

```
   if · the test rides its head line     RULED. New rule file. 1822 sites. NEXT.
   cond · clauses align                  RULED. New rule file. 51 sites.
   E · where a TRAILING comment goes     UNRULED — and now measurably a WIDTH cost too
   free-form String `kind`               the builder's catch — see below
   level-2 alignment inside a value      open since the kwargs stone
   arc 109 · bare variant ILLEGAL        the formatter is its migration
✅ the table rule claimed a definition    deporder 259 → 0, grep 66 → 0, fmt.wat 669 → 2
✅ the emitter survives a comment
✅ the arc-255 doc migration              0 over 120, INLINE back to 284, one trailing newline
```

## ⚠ AND THE BLOCKER COMMENT AT `wat/fmt.wat:9` IS FALSE — measured

> *"kind is a String, not a keyword: rete RHS may insert a string literal but refuses a keyword
> literal (`RhsUnresolvableOperand`)."*

True of a keyword; the conclusion does not follow.
`wat-scripts/scratch-pad/277-can-a-rete-rhs-carry-an-enum.wat`:

```
:then [(:user::Broken :id ?i :kind (:user::BreakKind::Block))]   →  RULES=1 BROKEN-FACTS=1
```

**Negative control**, the same slot with a bare `:Block` → `RhsUnresolvableOperand`. So the probe
discriminates: a keyword is refused because `validate.rs:1060` reads `:name` in a RHS as a FIELD
REFERENCE; an enum variant is a **call form**, and arc 278 Stone B removed `List` from the
never-resolves set — `validate.rs:1166`'s walk names *"a nested aggregate or enum-variant
constructor"* outright.

★ **The root cause is the diagnostic.** `RhsUnresolvableOperand`'s `accepted` list reads *"a ?var …
or an integer / float / boolean / string literal"* and **never mentions the call form**, though
`rhs_operand_can_never_resolve` accepts `IntLit | FloatLit | BoolLit | StringLit | List | ?var`. The
message taught the String. Fix the message and the wrong choice stops being the obvious one — the
class, not the case. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
