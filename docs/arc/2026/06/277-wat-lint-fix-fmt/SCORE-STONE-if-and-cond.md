# SCORE — STONE: `if`'s measurement rides, and `cond`'s clauses align

No commit. Floor and clippy left to the orchestrator. No Rust. No `wat/fmt.wat`. No existing rule file. No new fact record.

Two new files. `if` copies `match`: the test rides, each branch breaks one level in. `cond` has no subject; each clause breaks; the body rides its test; bodies align across clauses by joining them as one `TableRow` group — the existing fact that shares first-token width across sibling lists. AlignPairs cannot: it pads a broken child so the *next sibling* rides, and every clause is broken, so the pad never fires. The emitter's table fit test is already the 120 gate; this file names no budget.

## Row 1 — it builds

`cargo build --release` was already clean. Rules load from disk; `--check` of both new files and three new fixtures is **0**.

## Row 2 — `if`'s measurement RIDES

`wat-scripts/fmt/fixtures/if-ride.wat`, `run-all.wat`, `IDEMPOTENT=true`:

```
  (:wat::core::if (:wat::i64::> x 0)
    (:wat::i64::+ x 1)
    (:wat::i64::- x 1))
```

`(:wat::core::if (:wat::i64::> x 0)` is ONE line. Not the test on its own line. Not all three children riding.

## Row 3 — each branch on its own line, one level in

Same fixture. The `if` sits at indent 2 (defn body). Both branches at indent **4** — the `if`'s indent **+2**. `BreakKind::Block`.

## Row 4 — `cond`'s body RIDES its test

`wat-scripts/fmt/fixtures/cond-align.wat`, `IDEMPOTENT=true`:

```
  (:wat::core::cond
    ((:wat::i64::= k 0) "zero")
    ((:wat::i64::= k 1) "one")
    (:else              "many"))
```

`((:wat::i64::= k 0) "zero")` is ONE line.

## Row 5 — bodies ALIGN; the column is 25

Same fixture, 1-indexed column of the opening `"` of each body:

| clause | test width | body starts at |
|---|---|---|
| `((:wat::i64::= k 0) "zero")` | 19 | **25** |
| `((:wat::i64::= k 1) "one")` | 18 | **25** |
| `(:else              "many")` | 5, padded to 19 | **25** |

Indent 4 + `(` + 19 + space = 25. Two tests of different width share it. `:else` is padded.

## Row 6 — alignment obeys 120

`cond-overflow.wat` built first. Widest test and widest body sit on **different** clauses, so each clause fits alone and the aligned form would not:

```
  (:wat::core::cond
    ((:wat::string::= k "xxx…76") "a")
    ((:wat::string::= k "short") "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    (:else "c"))
```

`:else "c"` is **not** padded. The short test is **not** stretched toward the long one. `IDEMPOTENT=true`. Each clause by its own rules, unaligned.

STOP-2 asked to see the overflow align badly *before* the fit test exists. AlignPairs cannot produce the alignment (clauses are all broken). `TableRow` is the fact that can, and its emitter already consults 120 — there is no "always-align" setting for this shape short of editing `wat/fmt.wat` (STOP-1). The overflow fixture is the proof the fit test fires: rows 4–5's cond is padded; this one is not.

## Row 7 — a new rule is a new file

`git diff --stat` on tracked files: **9 files, +18**, all `load-file!` lines. **No** existing rule file. **Not** `wat/fmt.wat`. New: `rules/if.wat`, `rules/cond.wat`, three fixtures. Drivers that already loaded `match.wat` got both lines (run-all, run-examples, run-r4, run-tables, run-types, and the four census scripts). A rule file is inert until a driver loads it.

## Row 8 — width on the five real files

| file | expected | FORMATTED |
|---|---|---|
| deporder | 0 / 102 | **0 / 102** |
| grep | 0 / 98 | **0 / 109** |
| spawn | 9 / 174 | **9 / 174** |
| fmt | 2 / 147 | **2 / 147** |
| io | 0 / 104 | **0 / 104** |

over120 is no higher on any file. worst holds on four. **grep worst 98 → 109** is one riding `if` at indent 16:

```
                (:wat::core::if (:wat::core::= (:wat::grep::Extent/line ex) (:wat::grep::Extent/end-line ex))
```

109 columns, still under 120. Exploded, the test sat on the next line (~95) and grep's worst was the 98-column comments. Row 2 puts the test on the head line; that *is* the extra width. A fit test that refused the ride would be an emitter change (STOP-1) and would fail row 2 for this site. Named, not chased.

spawn 9/174 and fmt 2/147 left alone (STOP-3).

## Row 9 — the 614

`run-examples.wat /tmp/fmt-614.wat`: **N=614 CHANGED=614 INLINE=282 OVER120=0 WORST=104**.

OVER120=0, WORST=104. INLINE 284 → 282 is the two atom-only `if` examples (`(:wat::core::if true 1 2)` and `false`) now taking the ruled three-line shape. The other 608 stay within the same OVER120/WORST envelope.

## Row 10 — idempotent

Every fixture (48, including the three new) `IDEMPOTENT=true`. Five real files not byte-identical (this stone moves output on purpose) but the fixtures are.

## Row 11 — no comment is lost

| file | source | formatted |
|---|---|---|
| io | 28 | **28** |
| deporder | 85 | **85** |
| spawn | 429 | **429** |
| grep | 157 | **157** |
| fmt | 45 | **45** |

## Row 12 — the ruled shapes still hold

48 fixtures idempotent: defn, let, match, defrecord, defenum, kwargs, tables, atoms. `table-overflow.wat` still unaligned. `if-ride` / `cond-align` / `cond-overflow` as above.

## Row 13 — `cond`'s inconsistency is gone

`(:else "many")` and `((:wat::i64::= k 0) "zero")` are the same shape: one line, body at column 25. The atoms rule is no longer the only reason `:else` rides.

## Row 14 — files still end on one newline

`277-file-ends-with.wat`: `FORMATTED-trailing-empties=1` on if-ride, cond-align, cond-overflow, table-overflow, io.

## Row 15 — the walls stand

Kind-conflict sabotage (a second `if-branch` Break of `Align` beside `Block`) raises:

```
fmt: conflicting Breaks for node 16 — Align vs Block
```

Deleted after. `ClaimedUnder` **0**. `'col'` in rules **0**. `'120'` in rules **0**.

## Row 16 — wat-scripts load

`every_wat_scripts_file_loads_on_the_current_runtime` **1 passed**.

## Rows 17–18 — floor / clippy (ORCHESTRATOR)

Not run.

---

## Mechanism

**if.** `:fmt::if-claim` + `:fmt::if-branch-per-line`. Child 0 head, child 1 test (no Break), children index > 1 `BreakKind::Block`. Copied from `match.wat`.

**cond.** `:fmt::cond-claim` + `:fmt::cond-clause-per-line` (every child index > 0 is a clause). `:fmt::cond-clause-table` Claims each clause (body rides; R11 does not explode a call-test) and asserts `TableRow` with `group` = the first clause, so `:else` joins tests of a different head. The emitter pads column 0 to the group's max when `table-row-fits?`; otherwise each claimed clause stays one line, unpadded.

## Commands

| command | result |
|---|---|
| `--check` if.wat cond.wat + 3 fixtures | **0** |
| if-ride | test rides; branches indent+2; idempotent |
| cond-align | bodies at **column 25**; `:else` padded |
| cond-overflow | unaligned; `:else "c"`; idempotent |
| five-file census | over120 holds; grep worst **109** (one riding if) |
| comments both sides | 28 · 85 · 429 · 157 · 45 |
| 614 | **OVER120=0 WORST=104 INLINE=282** |
| 48 fixtures | **IDEMPOTENT=true** |
| kind-conflict sabotage | **Align vs Block**, deleted |
| load gate | **1 passed** |
| `git diff --stat` | no existing rule file, no `wat/fmt.wat` |

## Honest deltas

- AlignPairs is the wrong fact for cond. It aligns a broken child with its riding *sibling*. Cond's children are all broken clauses. `TableRow` already shares first-token width across sibling lists and already has the fit test. Reusing it is not a new record.
- STOP-2's "aligns badly before the fit test" cannot be staged for this shape without editing the emitter: the only fact that aligns also fits. The overflow fixture is still the control — it is the cond that does *not* pad, next to cond-align which does.
- Row 8 vs row 2 on `wat/grep.wat`: riding the test lengthens the head line by `(:wat::core::if `. One site goes 98 → 109. over120 still 0. Named.

## The board

```
✅ if · the test rides its head      NEW FILE, 118 sites in the five files
✅ cond · clauses align              NEW FILE, fit via TableRow; overflow unaligned
✅ Break.kind is an enum
✅ Node.kind is an enum
✅ :then checks declared field types
✅ fence refuses match it cannot prove
✅ variant-name                      core reader + Form exposure
   E · where a TRAILING comment goes UNRULED, and a width cost
```

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** Floor **5189 run, 5189 passed, 0 FAILED**. Clippy **0**.

| what | my own re-run |
|---|---|
| ★★★ row 7 · **A NEW RULE IS A NEW FILE** | `9 files changed, +18` — **all `load-file!` lines**. Existing rule files touched: **0**. `wat/fmt.wat` touched: **0**. Rule files 12 → **14**. **The arc's acceptance criterion, now proven SIX times.** |
| ★★★ rows 2+3 · `if` | `(:wat::core::if (:wat::i64::> x 0)` on one line; both branches at the `if`'s indent **+2** |
| ★★★ rows 4+5 · `cond` | bodies at **column 25** — measured, not eyeballed — with `:else` padded to the widest test |
| ★★★ row 6 · the fit test FIRES | see below |
| ★★ row 9 · the 614 | `OVER120=0 WORST=104 INLINE=282` |
| ★★★ row 11 · no comment lost | `28/28` · `85/85` · `429/429` · `157/157` · `45/45` |
| ★★ row 15 · hygiene | `'col'` in rules **0** · `'120'` in rules **0** · `ClaimedUnder` **0** |

### ★ Row 6 — the strike gave me a BETTER control than I asked for

STOP-2 demanded the overflow fixture align badly *before* the fit test existed. The strike showed
that cannot be staged for this shape: the only fact that can align sibling lists (`TableRow`) is the
one that already consults 120, so an "always-align cond" would require editing `wat/fmt.wat` —
which STOP-1 forbids. What it delivered instead is a **live A/B on one rule**:

```
cond-align     ((:wat::i64::= k 0) "zero")            :else PADDED to column 25
cond-overflow  ((:wat::string::= k "short") "bbb…")   nothing padded, (:else "c") bare
```

Same rule, two fixtures, opposite outcomes. **That discriminates better than the staged version**,
which would only have shown the pre-fix behaviour once.

## ⛔ ROW 8 IS VIOLATED, AND THE ROW IS WRONG — A THIRD SHAPE OF THE SAME FAILURE

```
grep.wat   baseline worst 98   →   now 109
```

The line, verified:

```
w=109 indent=16 :: (:wat::core::if (:wat::core::= (:wat::grep::Extent/line ex) (:wat::grep::Extent/end-line ex))
```

**That is row 2 doing exactly what it was written to do.** Riding the measurement lengthens the head
line by `(:wat::core::if `; a file whose previous worst was a 98-column comment now has a compliant
riding `if` at 109. **Rows 2 and 8 cannot both hold on that file, and I wrote both.**

★ The row's INTENT held everywhere: `over120` is **0 on grep and unchanged on all five**. The defect
is that I bolted a `worst` clause onto an `over120` bar for extra safety, without checking it against
row 2. `worst` measures the longest line — which a **compliant** ride can lengthen — so it was never
the right instrument for a stone whose whole purpose is to move lines.

⚠ **This is the third mis-derived row of this arc and it is a NEW shape.** The first two demanded
numbers the stone's own scope made *unreachable* (`spawn`/`fmt.wat` at 0; five files byte-identical
when one was edited). This one is *reachable* — and in direct conflict with another row I wrote in
the same table. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## Accepted deltas, both reported rather than buried

- **`INLINE` 284 → 282.** Two atom-only examples — `(:wat::core::if true 1 2)` and one on `false` —
  now take the ruled three-line shape. `OVER120=0` and `WORST=104` unchanged; the other 608 hold.
- **`AlignPairs` is the wrong fact for `cond`**, and the strike says why: it pads a broken child so
  the *next sibling* rides, and every cond clause is broken, so the pad never fires. `TableRow`
  already shares first-token width across sibling lists **and already carries the fit test**.
  Reusing it is not a new record — and row 7 proves it needed none.

## The board

```
✅ if · the measurement rides         NEW FILE · 118 sites in the five real files
✅ cond · clauses align               NEW FILE · via TableRow, overflow unaligned
✅ Break.kind · Node.kind are enums
✅ :then checks declared field types
✅ the fence refuses what it cannot prove
✅ variant-name                        core reader + Form exposure
   E · where a TRAILING comment goes   UNRULED, and a width cost

⛔ AND THE 255 UNLOCK IS NOT ANY OF THE ABOVE — measured this session:
   step 1  wat-fmt exists                        MET
   step 2  DocSpecialForm's metadata-map reader  NOT BUILT — 87 of 576 rows, 52 @alias,
                                                 36 @syntax, #wat.doc/Alias unwritable
   step 3  the @-form ratchet                    NOT BUILT
   step 4  the sweep, ONE pass, all 576
   and     crates/wat-doc/src/print.rs still holds no formatter call
```
