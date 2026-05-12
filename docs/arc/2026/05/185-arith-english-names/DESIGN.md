# Arc 185 — Arithmetic English-named per-Type ops; symbols dispatch

**Status:** stub opened 2026-05-13 per user direction.

## Motivation

> *"let's add a stub for arith names not symbols..."*
>
> Per-Type forms use English words:
> - `:wat::core::i64/plus`
> - `:wat::core::i64/difference`
> - `:wat::core::i64/times`
> - `:wat::core::i64/quotient`
> - `:wat::core::i64/remainder`
>
> Symbol forms dispatch:
> - `:wat::core::+`
> - `:wat::core::-`
> - `:wat::core::*`
> - `:wat::core::/`
> - `:wat::core::%`
>
> *"i have more thoughts - but this is enough for a stub"*

Per-Type arithmetic implementations get English names (`plus`,
`difference`, `times`, `quotient`, `remainder`). Symbol forms
(`+`, `-`, `*`, `/`, `%`) become bare dispatchers that pick the
right per-Type handler via arc 146 multimethod.

## Why English names

Symbols are dispatch entry points (concise + familiar). The
actual operations carry domain-honest English names that read
clearly in stack traces, in `(help :i64/plus)` reflection, and
when reading code that wants to be specific about which Type's
arithmetic is in play.

A user can write `(:wat::core::+ a b)` for the convenience form
OR `(:wat::core::i64/plus a b)` for the explicit form — both
work; the explicit form is unambiguous about the intent (vs `+`
which is dispatched).

Mirrors arc 184 (bare try/expect dispatching to Type/try) and
arc 148 (arithmetic handlers via arc 146 dispatch).

## Open design questions for the user

- **Division semantics**: `:wat::core::i64/quotient` is integer
  division; what about `:wat::core::f64/divide`? Names probably
  diverge per-type — i64 has `quotient`+`remainder`; f64 has
  `divide`+`fmod` (or similar). Surface in design.
- **Negation**: unary `-` (negate) vs binary `-` (difference).
  Different verbs? `:wat::core::i64/negate` for unary;
  `:wat::core::i64/difference` for binary. Or arity-suffix the
  symbol form `:wat::core::-'1` vs `:wat::core::-'2`.
- **Comparison naming**: `:wat::core::i64/less?` vs `lt?`?
  `equal?` vs `=?`? Open per-type taste.
- **Holon arithmetic**: bundle is "+" for holons (superposition).
  English-named as `:wat::holon::HolonAST/bundle`? Folds into
  the broader pattern.

## Sketch (placeholder; user has more thoughts)

TBD. User holds the broader design.

## Cross-references

- arc 146 (substrate dispatch mechanism — the enabler)
- arc 148 (arithmetic / comparison / holon-pair / time-arith
  handler migrations — this arc names the handlers; the two
  arcs may merge OR ship sequentially)
- arc 178 (primitive Type/fn shape — broader Type/verb concern;
  this arc is a sub-domain)
- arc 184 (bare try/expect dispatch — exact precedent for the
  symbol-as-dispatcher pattern)
- arc 174 (defclause — N-ary dispatch infrastructure)
