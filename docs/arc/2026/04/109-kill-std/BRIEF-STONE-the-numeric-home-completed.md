# BRIEF — STONE: complete the numeric home

Move 19 numeric-tower items out of `runtime.rs` into `src/numeric/`, so the home stops reaching back
for its own domain. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home-completed.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5114.

## Why this stone exists

`src/numeric/` was created four stones ago and the migration was **left half-finished** — the
orchestrator's function list was short by nineteen items. The home now imports its own domain back
from the megafile:

```
src/numeric/arith.rs:20   use crate::runtime::{collapse_bigrational, to_bigrational, I64ArithErr, …}
src/numeric/ops.rs:17     use crate::runtime::{bigint_component_to_value, …}
```

**That is the defect. Closing it is the deliverable** — not the line count, which is only ~168.

## Read in order

1. The DESIGN above.
2. **`src/numeric/`** — all five files. You are completing this home, not creating one; the concern
   split (`arith` / `convert` / `compare` / `ops`) already exists and the new items join it.
3. `src/intrinsic/{i64,f64,bigint,rational}.rs` — the EDGES. 33 of the 53 references live here.

## The work

### 1 — move 19 items into `src/numeric/`

**the `*_op` closures and the arithmetic error type → `arith.rs`**
`i64_add_op` 4287 · `i64_sub_op` 4297 · `i64_mul_op` 4307 · `i64_div_op` 4320 · `i64_quot_op` 4335 ·
`i64_rem_op` 4351 · `i64_mod_op` 4366 · `f64_add_op` 4582 · `f64_sub_op` 4589 · `f64_mul_op` 4596 ·
`f64_div_op` 4603 · `f64_max_op` 4610 · `f64_min_op` 4617 · `enum I64ArithErr` **5899** ·
`bigint_div` 4482 · `rational_div` 4534

**placement by CALLERS, not by my guess** — `to_bigrational` 4506 · `collapse_bigrational` 4521 ·
`bigint_component_to_value` 4555. Today `arith.rs` imports the first two and `ops.rs` the third;
**measure where their callers actually sit** and place accordingly. Report which callers decided it.

⚠ `I64ArithErr` is at **5899**, ~1,300 lines from the rest — it is not adjacent to anything you are
moving. It comes because its callers are `src/intrinsic/i64.rs` (the numeric edge), `src/numeric/arith.rs`
(this home), and `runtime.rs`, and because its own doc comment names it as the arithmetic verbs'
error type. **Take it by that evidence, not by proximity — it has none.**

### 2 — re-point 53 references across 6 files

`src/intrinsic/i64.rs` 19 · `src/numeric/arith.rs` 17 · `src/intrinsic/f64.rs` 7 ·
`src/intrinsic/rational.rs` 4 · `src/intrinsic/bigint.rs` 3 · `src/numeric/ops.rs` 3.

The compiler names them; fix what it names. Leave a short retirement comment at each cut in the shape
the previous stones used.

## Blast radius

`src/numeric/{arith,ops}.rs` · `src/runtime.rs` (19 items out) · `src/intrinsic/{i64,f64,bigint,rational}.rs`.
No `.wat` corpus change. No registrations. **No arithmetic verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `dispatch_rete_op` (4396) MUST NOT MOVE.** It sits **between** `i64_mod_op` (4366) and
`bigint_div` (4482) and is the dispatch spine's generic-op fallback — it recurses into
`dispatch_keyword_head_value`. **Both** independent `partire` casts flagged it by name as an intruder
in this exact range. `grep -c "fn dispatch_rete_op" src/runtime.rs` must still be **1**.

**⛔ STOP-2 — `eval_inner` and `eval_one_arg` STAY, and `src/numeric/` keeps importing them.**
`eval_inner` is used by six other homes. `eval_one_arg` is generic (`fn eval_one_arg<T>`) and its two
`runtime.rs` callers are `eval_bool_to_string` and `eval_keyword_from_string` — neither numeric.
**An impl home calling a shared evaluator primitive is the interface working.** If you find yourself
moving either to shorten an import list, STOP — that is the accidental seam.

**⛔ STOP-3 — THE ACCEPTANCE IS THE IMPORT LIST, NOT THE LINE COUNT.** When you finish:

```
src/numeric/arith.rs    must import from crate::runtime ONLY: eval_inner
src/numeric/ops.rs      must import from crate::runtime ONLY: eval_inner, eval_one_arg
src/numeric/convert.rs  unchanged: eval_inner, eval_one_arg
src/numeric/compare.rs  unchanged: eval_inner
```

Report all four verbatim. A green floor with these unchanged means nothing that mattered moved.

**STOP-4 — import from the canonical home, never through `runtime`'s facade.** `runtime.rs:759-784`
re-exports 22 `crate::value` names; `use crate::runtime::SymbolTable` compiles and is a lie.

**STOP-5 — verbatim.** No signature tidying. Visibility changes forced by the move are expected — on
the moving side and on functions that stay. Report each.

**STOP-6 — run the orphaned-doc-block scan** over the whole of `runtime.rs` after editing: any `///`
block left stranded above a retirement comment. Its result is a required report line.

## Report

Per-file diff summary; where each of the 19 landed; **all four `src/numeric/*.rs` `crate::runtime::`
import lines verbatim** (STOP-3's evidence); which callers decided the placement of `to_bigrational`
/ `collapse_bigrational` / `bigint_component_to_value`; confirmation `dispatch_rete_op` and
`eval_one_arg` are still in `runtime.rs`; before/after `wc -l src/runtime.rs`; the doc-block scan
result; and what surprised you.
