# EXPECTATIONS — D5

> The disconfirming probe was banked on 2026-08-30 and is RED by design. This scorecard is about what
> must be true once it is not.

## ⛔ NO PINNED TEST COUNT

Floor ≥ its current value plus the new gate's cases, zero FAIL rows.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ a legal `match` compiles in `:then` | **refused** — `experiri-then-match.wat` raises | loads |
| 2 | ★ the two spellings agree | bare refused, wrapped accepted | **both compile** — STOP-1 if not |
| 3 | the `where` fence is unchanged | `experiri-when-match.wat` loads | still loads |
| 4 | a constructor nested in an arm BODY is still validated | walked today (by accident, via the same fallthrough) | **still walked** — mutation 2 |
| 5 | the diagnostic no longer names a phantom insert | `RhsArityMismatch` on `:probe::E::A`, absent from source | that error does not fire on this program |
| 6 | the repro becomes a gate | `rune:lint(red-by-design)`, a banked file nothing runs | a regression gate asserting both spellings |
| 7 | the rune is retired with its reason | present | gone, and the retirement recorded |
| 8 | `let` / `fn` / `cond` untouched | measured immune | **no branch added for them** |
| 9 | engine untouched | — | zero diff under `src/rete/kernel/fire/` |
| 10 | lints | 210/210 | green, plus the new cases |
| 11 | clippy | rc=0 | silent |

## The mutation proofs

1. **Revert the walker fix** → the gate REDs on the bare spelling.
2. **Skip the arm BODY too** → a constructor nested in a body goes undetected and its gate REDs.
   *(Row 1 alone cannot distinguish a correct fix from "stop walking match forms entirely" — this is
   the row that separates them.)*
3. **The wrapped spelling** compiles before and after.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

50–80 minutes. The walker change is small; the gate and the body-recursion proof are the work.

## What would make this strike a failure even if every test passes

**A fix that stops walking `match` forms altogether.** It would make row 1 green, row 5 green, and
silently retire four real error kinds inside every match arm body. **Mutation 2 is the only row that
catches it** — and this arc has already shipped exactly that shape once: `strike-nested-wall` found
`walk_nested_constructors` orphaned by a lowering, with `UnknownField`, `RhsMissingFields`,
`RhsArityMismatch` and `RhsPositionalConstructionRetired` all unreachable and every gate green.

**And a cure keyed on a head the walker never sees.** `strike-nested-wall`'s whole lesson was *read
the form as it exists AT THE WALL, not as it was written* — the record-constructor case was invisible
because `defrecord` lowers before freeze. Measure the head; do not assume the source spelling reaches
this code.
