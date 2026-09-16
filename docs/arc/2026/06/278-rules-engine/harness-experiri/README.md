# experiri's harness — positions 3 and 4 of the declared surface

> Rescued from `/tmp` on 2026-08-30, at the builder's word, from the first real cast of
> `experiri`. It was about to evaporate.

## What this is

`src/rete/reachability.rs` sweeps `RETE_OPS` in **two** positions — inline-constraint and
where-fence — 79 rows x 2 = 158 cells. But `vocabulary.rs`'s own module doc declares **four**:
*"a `where` / `:then` / user accum fold may call"*.

`positions-3-4.rs.txt` is the ~237-line block that adds the other two: the `:then`
value-operand position (79 cells, sharded 4 ways like its siblings) and the user accumulator
acc-form head (1 cell — the acc-form's `(head ?v)` calling convention admits exactly one row,
`PersistentVector/length`, the only row whose declared params are one `PersistentVector<T>`;
the block computes that from `RETE_OPS` at run time rather than hard-coding it).

It appends to `reachability.rs` and references nothing outside that file's scope except
`crate::rete::vocabulary::`. Saved as `.rs.txt` so no tooling mistakes it for a live module.

**It carries its own calibration** — `experiri_then_calibration`, two pinned cells over four
drives with *mixed* outcomes. That is the property `complectens` found missing from 10 of the
15 file-walking gates in `tests/lint/` on the same day: a sweep that cannot demonstrate it
discriminates is a sweep that passes when it reaches nothing.

## What it proves

Two L1s, both driven, both reproduced by the `.wat` files beside this README:

1. **`PersistentVector/length` is UNREACHABLE as an accumulator head** and fires in all three
   other positions. `wat/rete/compile.wat:597` admits the acc-form on
   pure and deterministic and total and `primitive?` — and `primitive?` IS "has a `RETE_OPS`
   row", so every row passes by construction. `expr_ir/mod.rs:947`'s
   `lower_named_rete_fn` then resolves through `sym.get(head)`, the USER-function table, with
   no `rete_op_for` branch — while its sibling `lower_list` has one and says so in a comment.
   The raise reads `unknown rete-defn` about a minted row of the one table.
   Repro: `experiri-acc-head.wat` refuses; `experiri-acc-wrapped.wat` — same op behind a
   one-line user `defn`, same position — prints `"fired"`.
   **The class: any site that admits by `RETE_OPS` and dispatches by a different registry.**
   `holon_rete_ops_have_opexec` gates one such pair; this is the second, ungated.

2. **`match` is refused in `:then` and accepted in the `where` fence**, byte-identical
   expression. `validate/mod.rs:747`'s `walk_nested_constructors` cannot tell a match ARM from
   a CALL: `(:probe::E::A true)` has an enum-variant head, so the arity check fires the
   variant's 0 declared fields against the arm's length 1. It survives only by coincidence —
   `((:probe::E::A) true)` hides the keyword a level down, and a payload variant's arm matches
   1-against-1. Whether a legal `match` compiles in `:then` therefore depends on which of two
   equivalent spellings the author picked, and the diagnostic names an insert of
   `:probe::E::A` that appears nowhere in the source.
   Repro: `experiri-then-match.wat` refuses; `experiri-when-match.wat` loads.

This also closes a recorded deferral. `RETE-OPEN-WORK.md:1258` carried an unreproduced
`RhsArityMismatch` on `match` inside a `:then` with the instruction *"Drive it before
believing either"*, recorded 2026-08-29 and never executed. `exigere` found it buried under a
struck header in the same cast; `experiri` drove it.

## ⛔ WHY THIS IS NOT ON THE FLOOR YET, AND HOW TO LAND IT

Because it goes **RED against this HEAD** — that is the whole point of it. The floor is
zero-failure (5,165/5,165 at `.floor/2026-08-30T22-58-06Z`) and a red test may not be parked
there; `wat-rs/CLAUDE.md` is absolute on that.

**Land it WITH the fix, never before it and never after.** A gate written after a fix is one
that has only ever been green, which is the shape R59 names — `NISI FRANGAS, NIHIL PROBAS`.
Appended now, it is a mutation proof that costs nothing to obtain: it reddens on the defect,
and it must go green on the cure and on nothing else.

Procedure:

1. Append `positions-3-4.rs.txt` to `src/rete/reachability.rs`.
2. `cargo nextest run --release -E 'test(experiri)'` — CONFIRM RED, and confirm each arm
   fails for its own stated reason rather than for a harness error.
3. Fix the defects. For (1) that is a `rete_op_for` door in `lower_named_rete_fn` so the
   admission fence and the executor share one head-space; for (2) a head-aware
   `walk_nested_constructors` that does not descend into a form's pattern positions.
4. Re-run: GREEN, with the calibration still showing mixed outcomes.
5. Run the full floor before commit.

The `.wat` repros live here rather than in `wat-scripts/` deliberately: they are programs that
must FAIL, and `tests/lint/wat_scripts_fixes_load.rs` parses and type-checks every `.wat` under
that tree. Putting them there would redden a gate for being correct about being wrong.

`exp-matrix.txt` and `then-matrix.txt` are the raw per-row drive results from the cast.
