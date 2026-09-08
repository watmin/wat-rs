# BRIEF v2 — thread ONE depth budget through `LowerCx` and close the quote door

⛔ **This supersedes BRIEF v1. Your STOP-2 was correct and it changed the cure.** Read
`DESIGN.md`'s **REDRAWN** section first — it carries the driving, the bisected abort, and the
derivation of the bound. Everything below rests on it.

## The work

`:wat::rete::lower` (`eval_lower`, `src/rete/expr_ir/eval.rs:1340`, dispatched at
`src/runtime.rs:5753`) hands `lower` a quoted `WatAST` that **never passed through expansion**.
Measured: 50,000 deep accepted on an 8 MB stack; **`rc=134`, stack overflow, at 1539 on a 2 MiB
stack** — the nextest test-thread size. Three things:

1. A shared depth budget in `LowerCx`, refusing past `EXPANSION_DEPTH_LIMIT`.
2. The header paragraph, now recording **all three doors**.
3. A probe that drives **the quote door** and asserts a refusal, not an abort.

## Read in order

1. `DESIGN.md` § REDRAWN — the measurements and the bound's derivation. **Do not re-pick the bound.**
2. `src/rete/export.rs:372-383` — **`deeper`'s contract, and copy its shape.** *"ONE budget is shared
   across `unpack_expr`, `unpack_prog`, `unpack_pat`, `unpack_cond_op` and `unpack_driver` rather
   than each counting its own… a tower of `:user` nodes alternates between the two and is walked
   past by any counter that only one of them increments."* **The same is true here** — `lower_expr`
   ⇄ `lower_list` ⇄ `lower_hof_callee`, and `lower_pat` self-recurses at `mod.rs:809`. Four towers,
   one budget.
3. `src/rete/expr_ir/mod.rs:255-262` — `LowerCx`. The budget is a fifth field.
4. `src/rete/expr_ir/eval.rs:1340-1374` — `eval_lower`, the open door.
5. `src/macros/expand.rs:16` — `EXPANSION_DEPTH_LIMIT`, `pub use`d at `src/macros/mod.rs:113`.
   **Bind it; do not restate 512.**
6. `tests/rete/probe_arc278_expr_ir.wat:19` — the existing `(:wat::rete::lower (:wat::core::quote …))`
   fixture, for the surface shape.

## Implementation sketch

A `depth: u32` field on `LowerCx` plus one method mirroring `deeper` — increments, compares,
returns a `LowerError` naming the depth and the bound. **Every mutually-recursive `lower_*` calls it
as its first statement and shadows its own depth**, so the budget is shared rather than per-function.

`LowerError` must carry it as a real variant — a refusal, not a panic, not an `Option::None`. The
file's whole invariant is *"total or it refuses"*; the refusal is the deliverable.

## STOP triggers

1. **If a per-function counter is easier and you are tempted** — don't, and if you believe the
   alternating tower cannot happen here, STOP and show me why. `export.rs` names this as the
   contract, not a detail, and it has probes for exactly it.
2. **If refusing at `EXPANSION_DEPTH_LIMIT` reddens ANY existing test** — STOP and report which. The
   derivation says the compile path already caps at 509 so nothing should change; a red means that
   is wrong and the bound needs re-deriving, not raising.
3. **If the quote door still aborts at 2 MiB after the guard** — STOP. The guard is in the wrong
   place: something recurses before `lower` is reached (the parser, or `eval_inner` on the quote).
4. No change to `EXPANSION_DEPTH_LIMIT`, `export.rs`, or any lowering function's semantics.

## Blast radius

`src/rete/expr_ir/mod.rs` (the field, the method, the call sites, the header), possibly a
`LowerError` variant. One new probe under `tests/rete/`. **Nothing else.**
