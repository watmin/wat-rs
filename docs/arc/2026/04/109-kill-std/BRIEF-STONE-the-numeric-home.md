# BRIEF — STONE 1 of 2: `src/numeric/` — the tower gets a home, split by CONCERN

Move the numeric tower's implementations out of `runtime.rs` into `src/numeric/`, split by concern,
and re-point the four edges. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-numeric-home.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. The tree is clean and the floor is green
at 5114 — anything that breaks is yours.

## Read in order

1. The DESIGN above — especially **why the split is by CONCERN and never by TYPE**. The builder is
   adding all of Rust's numeric types (~16 against today's 5); a per-type layout grows one file per
   type and would defeat the entire point.
2. `src/collection/` — **the precedent, and the shape to copy.** `mod.rs` plus concern files
   (`eval.rs`, `infer.rs`, `map_container.rs`, `seq_container.rs`, `transform.rs`), declared
   `pub(crate) mod collection;` in `src/lib.rs:66`, reached from its edge as
   `crate::collection::eval::length_of(...)` (`src/intrinsic/collection.rs:111`).
3. `src/value/environment.rs:148` and `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` **actually live**. Read STOP-1 before you write a single `use`.
4. The four edges you will re-point: `src/intrinsic/{i64,f64,bigint,rational}.rs`.

## The work

### 1 — create `src/numeric/`, by concern

```
src/numeric/mod.rs        the module, its doc, and the tower's vocabulary
src/numeric/arith.rs      312 lines
src/numeric/convert.rs    247
src/numeric/compare.rs     34
src/numeric/ops.rs        type-specific operations that do NOT cross the tower
```

Declare it in `src/lib.rs` beside its siblings. `ops.rs` is the fourth concern and it is a real one:
`f64::round`/`unary`/`clamp` and `rational::numerator`/`denominator` are operations belonging to ONE
type rather than to the tower — they are not arithmetic, conversion or comparison, and folding them
into `arith.rs` would mix a tower-wide mechanism with a per-type surface.

### 2 — move the 24 implementations, verbatim

**arith** (`src/runtime.rs`): `eval_i64_arith` 9368-9415 · `eval_bigint_arith` 9624-9671 ·
`eval_rational_arith` 9732-9782 · `eval_f64_arith` 10009-10056 · `arith_i64_i64_inner` 11757-11794 ·
`arith_f64_f64_inner` 11797-11825 · `arith_bigint_bigint_inner` 11833-11861 ·
`arith_rational_rational_inner` 11870-11898

**convert**: `eval_i64_to_rational` 9801-9823 · `eval_bigint_to_rational` 9831-9853 ·
`eval_rational_to_f64` 9862-9890 · `eval_u8_cast` 9962-10005 · `eval_i64_to_string` 10159-10179 ·
`eval_i64_to_f64` 10184-10204 · `eval_i64_to_bigint` 10213-10233 · `eval_bigint_to_f64` 10241-10268 ·
`eval_f64_to_string` 10273-10293 · `eval_f64_to_i64` 10298-10323

**compare**: `eval_f64_compare` 11409-11443

**ops**: `eval_rational_numerator` 9908-9928 · `eval_rational_denominator` 9935-9955 ·
`eval_f64_round` 10337-10396 · `eval_f64_unary` 10405-10440 · `eval_f64_clamp` 10452-10503

⚠ **Bodies move VERBATIM.** No signature tidying, no "while I'm here" improvements. A behaviour
change hiding inside a relocation is the one thing that makes this stone's green meaningless.

### 3 — re-point 71 call sites

68 in the four edges, **3 inside `runtime.rs` itself** (measured). `crate::runtime::eval_i64_arith`
becomes `crate::numeric::arith::eval_i64_arith`, and so on. Leave a short retirement comment at the
`runtime.rs` cut in the shape arc 255's stones use.

## Blast radius

`src/numeric/` (new, five files) · `src/lib.rs` (one `mod`) · `src/runtime.rs` (778 lines out, three
call sites re-pointed) · `src/intrinsic/{i64,f64,bigint,rational}.rs` (68 call sites re-pointed). No
`.wat` corpus change. No registrations added or removed. **No verb's behaviour changes.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** This is the whole
stone's crate-liftability, and the compiler will not warn you. `src/runtime.rs:759-784` **re-exports
22 names from `crate::value`** — `Environment`, `SymbolTable`, `Value`, `EvalBreak`, `TrackedValue`,
`Function`, … So `use crate::runtime::SymbolTable` COMPILES and is a lie: the type lives in
`src/value/symbol_table.rs`. `src/check.rs:56` made exactly that mistake and it is a measured cause
of that home's cycle. **`src/numeric/` must import `Environment`/`SymbolTable`/`Value`/`EvalBreak`
from `crate::value::`**, `WatAST` from `crate::ast`, `Span` from `crate::span`. If you find a type
you genuinely cannot reach except through `crate::runtime::`, STOP and report which — that is a
finding about what still needs a home, not a licence to import through the facade.

**⛔ STOP-2 — `src/numeric/` must not reference `crate::intrinsic`.** The impl must not know about
its own edge. `grep -c "crate::intrinsic" src/numeric/*.rs` must be **0**. If the move seems to
require it, STOP — that is the cycle this architecture exists to prevent.

**STOP-3 — no per-TYPE files.** If you find yourself creating `src/numeric/i64.rs`, stop and
re-read the DESIGN. Sixteen types against per-type files is sixteen files; the concern split is the
deliverable, not the relocation.

**STOP-4 — `dispatch_rete_op` is NOT numeric.** It sits inside this range textually
(`runtime.rs:9528-9613`, between `eval_i64_arith` and `eval_bigint_arith`) and recurses into
`dispatch_keyword_head_value` — it belongs to the dispatch spine. `partire` flagged it by name for
exactly this reason. **Move by the function list in §2, never by line span.** A range is a claim
about every line inside it.

**STOP-5 — `src/value/numeric_order.rs` stays where it is.** It is the tower's ordering door and
moving it is defensible, but that is stone 2's call. Do not touch it.

**STOP-6 — verbatim means verbatim.** If a body cannot move without a change, STOP and report what
forced it.

## Report

Per-file diff summary; the five files you created and what went in each; **the `use` block of each
new file** (this is STOP-1's evidence and the orchestrator cannot reconstruct your reasoning from a
diffstat); confirmation that `grep -c "crate::intrinsic" src/numeric/*.rs` is 0 and that no import
reaches a `crate::value` type through `crate::runtime`; the before/after `wc -l src/runtime.rs`; and
what surprised you — a body that would not move verbatim, a type only reachable through the facade,
or a call site the count of 71 did not include.
