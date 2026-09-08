# SCORE — STOP-2. `:wat::rete::lower` takes AST that never went through expansion

The DESIGN listed seven production callers of `expr_ir::lower` / `lower_in_frame` /
`lower_named_rete_fn` and said they all take AST from expanded source. There is an eighth.

## STOP-2

`src/rete/expr_ir/eval.rs` `eval_lower` (`:wat::rete::lower <quoted-expr>`):

```
let v = eval_inner(&args[0], …)?;          // typically a quote
let ast = match v { Value::wat__WatAST(a) => … };
lower(&ast, sym)?;                         // no expansion, no depth
```

`expand_form` returns a `quote` / `forms` / `literal` list **untouched**
(`Boundary::AllData` — `expand.rs`, the quote-family check). Children are not
walked, `expansion_depth` is not incremented. The quoted payload is then handed
to `lower`, which has no depth counter of its own.

That is a caller whose AST does not come from expanded source. The BRIEF says
this outranks the strike and changes the cure back toward a guard in `LowerCx`.
Stopped. Did not add the header. Did not add the probe. Did not thread `depth`
through `LowerCx`. Did not touch `EXPANSION_DEPTH_LIMIT` or `export.rs`.

The seven listed callers (`compiled_rhs.rs:154,193`, `compiled_cond.rs:758`,
`matcher.rs:854`, `arm.rs:111,430,499`) still match the DESIGN: compile-path AST
from expanded source, plus field-reference rewrites of bounded constant depth.
`eval.rs:1457` also calls `lower` on `parse_all` output with no expand, but that
is a `#[cfg(test)]` probe with a one-line form.

## What I did not measure

Did not reproduce 509/510. Did not drive a 510-deep quoted form through
`:wat::rete::lower` to see whether parse fails first, or `lower` aborts. STOP-2
fired on the caller, not on an abort. The abort question is now the
orchestrator's: if quote-then-lower can present a 510-deep `WatAST`, `2W1` is
right about *this* door even though it was wrong about the compile path.

## Scorecard

| # | result |
|---|---|
| 1 lowering logic untouched | **HOLD by not starting.** No `expr_ir` edit. |
| 2 the probe | **not built.** STOP-2. |
| 3 floor | **not run.** No code change. Last green remains `.floor/2026-09-07T23-57-15Z/` 5480/19. |
| 4 clippy | **not run.** No code change. |

## What this did not do

No `LowerCx` depth. No `EXPANSION_DEPTH_LIMIT` change. No `export.rs`. No probe
that would go green over this door.
