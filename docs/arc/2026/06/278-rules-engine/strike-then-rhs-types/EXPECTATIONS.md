# EXPECTATIONS — D10

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ a knowably-wrong `:then` value is refused | **accepted**; `#tr/Bad {:n "not-an-i64"}` reaches the fact set | refused at rule-compile, both types named |
| 2 | ★ a literal is refused too | **accepted**; `#tl/Bad {:n "LITERAL-STRING"}` | refused |
| 3 | ★ not-knowable still compiles | n/a | compiles — **mutation 2** |
| 4 | the well-typed control still derives | `Good count: 1` | unchanged |
| 5 | the corpus survives | unmeasured | **counted, and every new failure reported** — STOP-1 |
| 6 | a RHS type error kind exists | **0** in `error.rs` | present, diagnostically complete |
| 7 | `:when` typing untouched | refuses `:T` comparisons | unchanged |
| 8 | no engine change | — | zero diff under `src/rete/kernel/` |
| 9 | floor / lints / clippy | 5345, 210/210, rc=0 | green |

## What would make this strike a failure even if every test passes

**Refusing what is merely not-knowable.** Rows 1, 2 and 4 all go green for a cure that refuses every
operand it cannot type — and a corpus of legal rules stops compiling. **Row 3 / mutation 2 is the
only thing standing between the cure and that**, and it must be a *constructed* not-knowable operand,
not an assertion that one would pass.

**And trusting the resolver's doc over its arms.** That function shipped a claim of exhaustiveness
while a source was missing; a computed operand fell to a `_` arm meaning "unbound `?var`" and skipped
the check outright. A cure built on the doc's four bullets, without reading the arms, can inherit the
same hole in the new position.
