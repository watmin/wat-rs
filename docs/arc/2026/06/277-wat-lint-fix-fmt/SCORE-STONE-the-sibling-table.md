# SCORE — STONE: the sibling table

No commit. Floor and clippy left to the orchestrator. Level-2 value alignment still open. Width does not drive layout.

## 3-row keyword group — 3 lines, not 15

`kw-table.wat`, `IDEMPOTENT=true`:

```
(:wat::grep::Node :id 1 :parent 0 :index 0 :kind "symbol")
(:wat::grep::Node :id 2 :parent 0 :index 1 :kind "keyword")
(:wat::grep::Node :id 3 :parent 9 :index 2 :kind "list")
```

`run-tables.wat`: `GROUPS2=1 GROUPS3=1 ROWS=3`.

## Positional group

```
(:wat::core::Tuple "short"       "xx")
(:wat::core::Tuple "much-longer" "y")
```

Same head, same arity. First column padded. Last column unpadded (no trailing space before `)`).

## Different key order is not a group

`key-order.wat` — two adjacent `format` calls, `:b :a` vs `:a :b`: `GROUPS2=0`. Each still pair-run explodes. Head-alone grouping would have tabled them.

Different heads (`+` / `*`) ungrouped.

## Group of 2

`pair-table.wat`: `GROUPS2=1 GROUPS3=0 ROWS=2`. At a threshold of 3 this group would not form.

## The drifted real table — exact

`rules-corpus-02:176-180` after format (no stray space on the `+` row):

```
(:m::Member :id 1 :prefix "String" :base "concat"    :style "slash")
(:m::Member :id 2 :prefix "string" :base "length"    :style "colons")
(:m::Member :id 3 :prefix "i64"    :base "to-string" :style "colons")
(:m::Member :id 4 :prefix "i64"    :base "+"         :style "colons")
(:m::Member :id 5 :prefix "Vector" :base "get"       :style "slash")
```

`rules-corpus-01` Node constructors stay one line; no padding was needed and none was added.

## Mechanism

- `FormSig {form, head, keys}` — keys is the trailing pair-run key sequence, or `"#N"` for positional arity. Built by a pre-order walk with the same ids as grep.
- `TableRow {form, group}` — group is the first member's id. `table-start` / `table-extend` in `rules/table.wat`.
- Emitter: group-scoped max of `ast->source` lengths per child index, this pass only. Last child is not padded.
- Pair-run Breaks on a table member's children are ignored (parent is in the table map). `kwargs-claim` also has `not TableRow`.

`grep -c 'col' rules/*` → **0**.

## Counts

| | threshold 2 | threshold 3 |
|---|---|---|
| 614 doc examples | **49** | **1** |
| corpus (507 files: `wat/`, scratch-pad, fixes) | **1527** | **400** |

614 examples: `N=614 CHANGED=342 OVER120=0 WORST=104` — over-120 still **0**.

## Walls / comments / load

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `io.wat` **COMMENTS=28**. Existing fixtures idempotent. `--check` clean on new fixtures before the floor. `every_wat_scripts_file_loads` **1 passed**.

No Rust. `Break` / `Claim` / `AlignPairs` / `BlankBefore` unchanged as records.

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** ⛔ **And row 6's report is a number the builder needs before this is applied
anywhere.**

| what | result |
|---|---|
| ★ row 2 — the 3-row keyword group | **3 aligned lines, not 15**, idempotent |
| ★★★ row 4 — a different KEY ORDER | **NOT grouped** — `format.wat`'s reversed call survives, each explodes |
| ★ row 3 / 5 — positional group · different heads | grouped · ungrouped |
| ★★★ row 7 — the drifted real table | **EXACT** — the stray space on the `+` row is gone |
| row 8 — a group needing no padding | recognised, and **no padding added** |
| row 10 — no rule names a column | `grep -c 'col' rules/*` **0** |
| row 16 — the 614 doc examples | over-120 still **0**, worst 104 |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

Rows 7 and 8 together are the pair that matters: **the drifted table comes out exact** (unchanged
would have been a failure) **and** the already-uniform one is left alone. A rule that only fires when
padding is needed passes 7 and fails 8; this one passes both.

## ⛔⛔ ROW 6 / 17 — THE THRESHOLD IS A REAL DECISION, AND MY PROXY WAS OFF BY 4×

```
                          threshold 2      threshold 3
614 doc examples                 49                1
corpus (507 files)            1,527              400
```

**My earlier estimate was ~354 candidate runs. The real count at threshold 2 is 1,527** — a proxy
(*3+ consecutive lines with 2+ internal spaces*) that missed every group needing no padding, which
row 8 proves is a real category. `[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`

★ **And the doc-example spread is 49 versus 1.** In the priority target, threshold 2 forms
**forty-nine** tables where threshold 3 forms **one**. That is not a tuning knob; it is two different
formatters.

> ⬜ **BUILDER'S CALL, and it is now on evidence rather than my preference.** The ruling was 2
> (*"different instances"*), and 2 is what shipped. I had recommended 3 and said I would ship the
> ruling and report the counts — **these are the counts.** 1,527 corpus groups and 49 doc-example
> groups is a large surface for a rule nobody has read the output of at scale.

⚠ **Nothing here says 2 is wrong.** A two-row table is a table. But the number should be seen before
the rule is pointed at 507 files, and it was invisible until this row was run.

## Not disputed

`FormSig {form, head, keys}` keys the group on head AND key sequence (`"#N"` for positional arity) —
which is what makes row 4 hold. Group-scoped padding is computed from **this pass's** `ast->source`
lengths; the last column is never padded, so no table row grows a trailing space. Pair-run Breaks are
suppressed on table members, and `kwargs-claim` carries `not TableRow`. `io.wat` **COMMENTS=28**.
Every new fixture `--check`s clean before the floor.

## ⬜ REMAINING

```
level-2 alignment INSIDE a value    (grep.wat:284-289's Location/line vs Location/col)  — open
the emitter's comment-indent defects — CORPUS-ONLY, and 277's corpus work is not now
fn / long-defn arg-spec one-per-line — corpus-visible, not doc-example-visible
R15 as a lint · wat fmt --check on the floor
```

★ **None of it blocks the builder's stated priority.** The doc examples format to 0 over 120 and are
idempotent; the reason `[[PARKED-the-migration-waits-on-wat-fmt]]` gave for parking arc 255 is
discharged. The next step on that path is wiring the `#wat.doc/Row` printer to emit through the
formatter.
