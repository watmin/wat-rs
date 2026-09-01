# DESIGN — STONE: complete the numeric home — the migration I shipped half-finished

> **Builder, 2026-09-01:** *"draw it."*
>
> This stone exists because a stone I already shipped was incomplete, and **its own rider told me so
> at the time.** `[[NOTE-partire-RECAST-on-the-current-runtime]]` item 1.

## ⛔ THE DEFECT IS MINE, AND IT WAS REPORTED BEFORE IT WAS FOUND

`src/numeric/` **reaches back into `runtime.rs` for its own domain**:

```
src/numeric/arith.rs:20   use crate::runtime::{collapse_bigrational, to_bigrational, I64ArithErr, …}
src/numeric/ops.rs:17     use crate::runtime::{bigint_component_to_value, …}
```

The numeric stone's brief named 24 functions. **My list was the defect** — nineteen numeric-tower
items were not on it. The rider flagged exactly this in its report:

> *"four sibling helpers … were not in the brief's 24-function list, so they were left in place
> rather than relocated"* … *"`I64ArithErr` is numeric-tower vocabulary that did NOT move …
> flagging as a candidate for stone 2 or a follow-up."*

I recorded it as an honest delta and did not act. A re-cast then found the same gap from the opposite
direction. `[[feedback_a_lesson_learned_and_then_dropped]]`

## What moves — 19 items, **enumerated**, ~168 lines

```
i64_add_op    4287   i64_sub_op   4297   i64_mul_op    4307   i64_div_op   4320
i64_quot_op   4335   i64_rem_op   4351   i64_mod_op    4366
bigint_div    4482   to_bigrational 4506  collapse_bigrational 4521  rational_div 4534
bigint_component_to_value 4555
f64_add_op    4582   f64_sub_op   4589   f64_mul_op    4596   f64_div_op   4603
f64_max_op    4610   f64_min_op   4617
enum I64ArithErr 5899
```

Destinations, by the concern split the home already has: the `*_op` families and `I64ArithErr` →
`arith.rs`; `to_bigrational` · `collapse_bigrational` · `bigint_component_to_value` → whichever of
`arith.rs`/`ops.rs` their callers sit in (**the rider measures**, as the previous stones did).

## ★★ `I64ArithErr` IS THE SEVENTH SPAN-TRAP INSTANCE — AND IT HID IN THE *LEAVE* LIST

The re-cast enumerated its SPLIT modules by name (that was the whole instruction) but gave its
**LEAVE** list as ranges — and `I64ArithErr` sits at **5899**, inside the eval-spine range
`5745–5954`. So it was left with the spine **by line position**.

Its callers say otherwise: `src/intrinsic/i64.rs` (the numeric **edge**), `src/numeric/arith.rs`
(the numeric **home**), `runtime.rs`. And its own doc comment: *"arithmetic verbs' `value_handler`
adapters (`src/intrinsic/{i64,f64,bigint,rational}.rs`) call these SAME fns."* **It is numeric
vocabulary.**

★ The lesson generalises: **the enumerate-by-name discipline must cover the LEAVE list too.** An
intruder hides just as well in "what stays" as in "what moves" — and it is harder to notice, because
nothing moves to expose it.

## ⛔ NOT EVERY `crate::runtime::` IMPORT IS A DEFECT — the distinction this stone pins

`src/numeric/` will still import two things after this stone, correctly:

```
eval_inner     the evaluator's inner entry point — also used by collection, declare, function,
               intrinsic/bytes, intrinsic/char, … A shared primitive. STAYS.
eval_one_arg   `fn eval_one_arg<T>` — a GENERIC single-arg unwrapper. Its two runtime.rs callers are
               eval_bool_to_string and eval_keyword_from_string, neither numeric. STAYS.
```

**An impl home calling a shared evaluator primitive is the interface working. An impl home calling
its OWN domain's code in another file is the defect.** Only the second kind moves.

## ★ THE PREDICTION — falsifiable

```
runtime.rs                 25,997 -> ~25,830   (-168)
src/numeric/arith.rs+ops.rs  gain the 19
crate::runtime:: in src/numeric/*.rs
    arith.rs   {collapse_bigrational, eval_inner, to_bigrational, I64ArithErr}  ->  {eval_inner}
    ops.rs     {bigint_component_to_value, eval_inner, eval_one_arg}            ->  {eval_inner, eval_one_arg}
    convert.rs {eval_inner, eval_one_arg}                                        UNCHANGED
    compare.rs {eval_inner}                                                      UNCHANGED
53 refs across 6 files re-point   (intrinsic/{i64 19, f64 7, rational 4, bigint 3}, numeric/{arith 17, ops 3})
dispatch_rete_op (4396)    UNTOUCHED — sits between i64_mod_op and bigint_div, is the dispatch spine
behaviour                  every arithmetic verb identical
```

⚠ **The load-bearing acceptance row is the import lists above** — this stone's whole point is that
the numeric home stops reaching back for its own domain. A green floor with those imports unchanged
would mean nothing moved that mattered.

## Out of scope = REJECTED (not deferred)

- **`eval_inner` / `eval_one_arg`.** Shared primitives, argued above. Moving them would break six
  other homes.
- **`dispatch_rete_op`** (4396) — the dispatch spine, sitting *between* `i64_mod_op` and `bigint_div`.
  Flagged by BOTH casts, by name. It is the campaign's most re-confirmed intruder.
- **The other six re-cast modules** — `record` · the kernel family · died-error/outcome ·
  `holon::outcome` · `option`/`result` · the purity classifier. One stone each.
- **numeric stone 2's promotion lattice.** Still the real prize for "adding `i8` is a row"; still
  after the home is whole. ⚠ **This stone is its precondition** — a lattice cannot be built while
  half the tower lives in another file.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the 19, into the home's existing concern files** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the 18, leaving `I64ArithErr` with the spine | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| move `eval_one_arg` too, to clear the import | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| leave it; do the lattice and fix this en route | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| a new `src/numeric/scalar_ops.rs` for the `*_op` families | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **leave-`I64ArithErr` Honest? NO** — it is the arithmetic error type, named as such by its own doc
  and used by the numeric edge and home. Leaving it is deferring to a line number over three callers.
- **move-`eval_one_arg` Honest? NO** — it is generic (`<T>`), and its runtime callers are
  `bool::to-string` and `keyword::from-string`. Moving it to satisfy an import-count would relocate a
  shared helper into one consumer's home — the accidental seam, exactly.
- **do-it-during-the-lattice Simple? NO / Honest? NO** — it bundles a relocation with an algorithm
  rewrite, so a red is un-attributable; and it leaves a known, reported defect standing while a
  larger change lands on top of it.
- **new-file Honest? NO** — `arith.rs` already holds `eval_i64_arith`/`arith_i64_i64_inner`; the
  `*_op` closures are the operations those very functions take. A third file would split one concern
  by size.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ the home stops reaching back | `grep -h "crate::runtime::" src/numeric/*.rs` | only `eval_inner` / `eval_one_arg` |
| the megafile sheds it | `wc -l src/runtime.rs` | ~25,830 |
| ★ `dispatch_rete_op` did not move | `grep -c "fn dispatch_rete_op" src/runtime.rs` | **1** |
| `eval_one_arg` did not move | `grep -c "fn eval_one_arg" src/runtime.rs` | **1** |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/numeric/*.rs` | 0 |
| the 53 refs re-point | `crate::runtime::` for the 19, outside `src/numeric/` | 0 |
| behaviour unchanged | every arithmetic verb | identical |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
