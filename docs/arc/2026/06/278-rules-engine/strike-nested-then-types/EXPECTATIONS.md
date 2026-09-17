# EXPECTATIONS — D11

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ a knowably-wrong NESTED value is refused | **accepted** — `#nh/Inner {:n "nested-string"}` reaches the fact set | refused, both types named |
| 2 | ★ not-knowable at depth still compiles | n/a | compiles — **mutation 2, constructed** |
| 3 | ★ D5 survives: `match` in `:then` compiles | `experiri-then-match.wat` loads | still loads — **mutation 4** |
| 4 | a well-typed nested control still derives | — | derives |
| 5 | the corpus survives | D10 measured 1664 files, 0 newly-failing | re-measured, every new failure reported |
| 6 | no new error kind | `RhsFieldTypeMismatch` exists | reused unchanged |
| 7 | radius | — | `validate/mod.rs` + a gate only |
| 8 | new fixtures have real `main`s | legacy idiom is blind (C18) | **run and print** when the wall is absent |
| 9 | floor / lints / clippy | 5351, 210/210, rc=0 | green |

## What would make this strike a failure even if every test passes

**Refusing not-knowable at depth.** Rows 1 and 4 go green for a cure that refuses everything it
cannot type, and the corpus breaks. Row 2 is the only guard, and D10 proved this is not theoretical:
making that variant a refusal took four pre-existing corpus tests down with it.

**Regressing D5.** This walker is the one that learned to skip `match` arm patterns. A `binds` thread
that disturbs the arm/pattern split re-refuses legal `match` in `:then` — and rows 1, 2 and 4 would
all still pass. **Row 3 is the only thing watching it.**

**And a `.wat.bad` fixture with a `nil` main.** Its `assert!(!ok)` cannot fail under its own mutation
(C18). A strike that closes a soundness hole with a probe that cannot detect its own reversal has
bought a green light, not a wall.
