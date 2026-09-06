# SCORE — STONE: Break.kind becomes an enum

No commit. Floor and clippy left to the orchestrator. No Rust. No new fact record. No new rule file. Node.kind, `if`/`cond`, defect E, `RhsUnresolvableOperand` untouched.

`:wat::fmt::Break.kind` is `:wat::fmt::BreakKind`. The unknown-kind raise is gone; the conflicting-Breaks wall names two variants.

## Row 2 — the enum, two variants, no Unknown

```
(:wat::core::defenum :wat::fmt::BreakKind :wat::enum::Pure
  :Block []
  :Align [])
```

No `:Unknown`. The false String-blocker comment is deleted.

## Row 3 — no kind STRING literal in the rules

`grep -rc '"block"\|"align"' wat-scripts/fmt/rules/` → **0** across every file (grep exits 1).

## Row 4 — 23 assert sites, every one a VARIANT

`grep -c 'fmt::Break :id'` sums to **23**. Every one is `:kind (:wat::fmt::BreakKind::Block)` or `::Align`.

## Row 5 — pad-break is an exhaustive match, no else

```
(:wat::core::match bk
  ((:wat::fmt::BreakKind::Block) … indent + 2 …)
  ((:wat::fmt::BreakKind::Align) … open-col + 1 …))
```

`assertion-failed! "fmt: Break.kind must be block or align"` is **GONE**. Not an `if`. No `_`.

## Row 6 — the wall is now a COMPILE error

Deleting the `:Align` arm of a `match` on `:wat::fmt::BreakKind` (the pad-break shape):

```
non-exhaustive: enum :wat::fmt::BreakKind missing arm(s) for variant(s): Align
```

Both-arm control `--check`s clean. pad-break restored after. (`wat --check wat/fmt.wat` only reports ReservedPrefix — stdlib files cannot be `--check`ed as user code — so the probe is a user `match` on the same enum.)

## Row 7 — conflicting-Breaks still FIRES

Sabotage on `defn-multi.wat` (`Block` vs `Align` on the same `->` node):

```
fmt: conflicting Breaks for node 11 — Align vs Block
```

Deleted after. Two VARIANTS, not two strings.

## Row 8 — breaks-map value type is the enum

`HashMap :- [:wat::core::i64 :wat::fmt::BreakKind]`. Not String.

## Row 9 — formatted output byte-identical (the four corpus files)

`cmp` against the BEFORE capture (format-source written to disk, no EDN decode):

| file | vs BEFORE |
|---|---|
| `wat/deporder.wat` | **IDENTICAL** |
| `wat/grep.wat` | **IDENTICAL** |
| `wat/spawn.wat` | **IDENTICAL** |
| `wat/io.wat` | **IDENTICAL** |
| `wat/fmt.wat` | source edited (enum + pad-break match) — formatted form follows the source edit, not a layout drift |

A representation change that moved a column would have shown in deporder/grep/spawn/io. It did not.

## Row 10 — width numbers unchanged

| file | FORMATTED |
|---|---|
| deporder | **0/102** |
| grep | **0/98** |
| spawn | **9/174** |
| fmt.wat | **2/147** |
| io | **0/104** |

## Row 11 — 614

`N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104`.

## Row 12 — file endings

`277-file-ends-with.wat`: **FORMATTED-trailing-empties=1** on deporder, grep, spawn, fmt, io, comment-indent, atom-map.

## Row 13 — no comment lost

| file | source/formatted |
|---|---|
| io | **28/28** |
| deporder | **85/85** |
| spawn | **429/429** |
| grep | **152/152** |
| fmt.wat | **45/45** (was 47: the two-line false String comment was deleted with the String) |

## Row 14 — idempotent

Every fixture `IDEMPOTENT=true`. Five real files `IDEMPOTENT=true`.

## Row 15 — recorded codemod, not hand edits

`wat-scripts/fixes/break-kind-string-to-enum.wat`. Dry-run on `/tmp` copies, `diff` showed the 23 sites and the three header comments. Applied. **Second run: CHANGED=0**.

## Row 16 — `--check` clean

Every fixture. The new fix script `--check`s clean.

## Rows 17–18 — walls / load

`ClaimedUnder` **0**. `grep -c 'col'` / `'120'` over rules **0**. `every_wat_scripts_file_loads` **1 passed**.

## Mechanism

- `BreakKind` `:Pure`, two nullary variants. `Break.kind` is that enum.
- Rule `:then` writes `(:wat::fmt::BreakKind::Block)` — the proven RHS call form, not a bare keyword.
- `pad-break` exhaustive `match`. The third state has no constructor.
- `breaks-map` still raises on two different kinds for one node; interpolate prints variant names via `break-kind-name` (also exhaustive).
- Codemod: walk `:wat::fmt::Break` constructors, replace the string after `:kind` by span (`fix-text-apply`). Comments that named `"block"`/`"align"` rewritten in the same pass. Edits sorted by offset, applied right-to-left.

---

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `grep -rc '"block"\\|"align"' rules/` | **0** |
| 23 Break sites | all `BreakKind::Block` / `::Align` |
| non-exhaustive probe | **missing arm(s) for variant(s): Align** |
| kind-conflict sabotage | **Align vs Block**, then deleted |
| cmp deporder/grep/spawn/io vs BEFORE | **IDENTICAL** |
| census | 0/102 · 0/98 · 9/174 · 2/147 · 0/104 |
| 614 | INLINE=284 OVER120=0 WORST=104 |
| codemod second run | **CHANGED=0** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** The wall climbed a rung: `Break.kind`'s invariant is no longer defended by a
runtime raise, it is unrepresentable.

| what | my own re-run |
|---|---|
| ★★★ row 9 · **BYTE-IDENTICAL** | against the digests captured at DRAW time, before the strike began: `deporder c69f0460e9f91672` · `grep 1dce84a9077037da` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` — **all four unchanged** |
| ★★★ row 6 · the wall is a COMPILE error | `non-exhaustive: enum :wat::fmt::BreakKind missing arm(s) for variant(s): Align` · **control:** the same file with both arms `--check`s clean |
| ★★★ row 7 · the OTHER wall still fires | `conflicting Breaks for node 11 — Block vs Align`, naming two VARIANTS · control clean after restore |
| ★★★ row 15 · a real, idempotent codemod | my second run **`CHANGED=0`** · **positive control:** the same codemod on a pre-migration `defn.wat` from git HEAD → **`CHANGED=1`** with the exact rewrite |
| row 3 · no `kind` string left in the rules | **0** files |
| row 4 · the 23 sites | **23** asserts, **23** carrying a variant |
| row 5 · `pad-break` | `assertion-failed` **0** · wildcard `_` **0** |
| row 8 · `breaks-map` value type | `HashMap :- [i64 :wat::fmt::BreakKind]` |
| row 10 · width | `0/102` · `0/98` · `9/174` · `2/147` · `0/104` — unchanged |
| row 11 · the 614 | `N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104` |
| row 12 · file endings | `FORMATTED-trailing-empties=1` |
| row 13 · comments | `28/28` · `85/85` · `429/429` · `152/152` · `45/45` |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy | **0** under `-D warnings --all-targets` |

### ★ Row 9's `fmt.wat` delta is the SOURCE edit, checked line by line

`wat/fmt.wat`'s own formatted output differs, and it must: the stone edits its source. Every one of
the 69 changed lines is accounted for — the rewritten header comment, the `defenum`, `Break.kind`'s
type, `pad-break`'s and `apply-break`'s parameter types, and the `if`+`assertion-failed!` collapsing
into a two-arm `match`. **No layout drift.** The row as written demanded all five files be
byte-identical, which the stone's own scope makes impossible for `fmt.wat` — a smaller instance of
the mis-derived row this arc keeps producing.

### ⚠ MY FIRST ROW-7 SABOTAGE WAS MIS-AIMED AND ITS SILENCE PROVED NOTHING

Flipping one rule's `Block` to `Align` produces no conflict — it just changes the kind. The wall needs
**two rules disagreeing about one node**, so the real sabotage appends a twin rule claiming the same
`?arrow` node with the opposite variant. The first attempt returned nothing, and *nothing* is exactly
what a retired wall returns.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

### Honest deltas, both benign and both the strike's own reporting

- `wat/fmt.wat`'s comment count is **45**, was 47: the two-line String-blocker comment was deleted
  with the String it justified. Correct, and the strike said so.
- `wat --check wat/fmt.wat` reports `ReservedPrefix` — stdlib files cannot be checked as user code —
  so row 6's probe is a user-namespace `match` on the real `:wat::fmt::BreakKind`. That is the same
  enum and the same checker; the restriction is on the file, not the type.

## The board

```
   Node.kind → enum                 NEXT. 62 comparisons across 12 rule files, producer at
                                    grep.wat:194 (pure wat, no Rust side)
   if · the test rides its head     RULED, new rule file, 1822 sites
   cond · clauses align             RULED, new rule file, 51 sites
   RhsUnresolvableOperand's message  the CLASS behind this bug — its `accepted` list omits the
                                    call form and is what taught the String. Rust. Own stone.
   E · where a TRAILING comment goes UNRULED, and a width cost
✅ Break.kind is an enum             23 sites by codemod · a runtime wall retired into the type
```
