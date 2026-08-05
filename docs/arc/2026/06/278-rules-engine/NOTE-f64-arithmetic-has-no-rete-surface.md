# NOTE — f64 arithmetic has no rete surface, and it is a `Fallback`-class stone

**Filed 2026-08-05.** Deferred out of `BRIEF-the-f64-surface-is-a-stub.md` by its own STOP-1, which
scopes that stone to `OpClass::Alias` rows only. This note is the declared home so the deferral is a
tracked item rather than a sentence inside a STOP.

## The state, measured

`RETE_OPS` splits per-type rows into two classes, and the split is by **totality**:

| class | rows | why |
|---|---|---|
| `OpClass::Alias` | `i64::{> < >= <= = not= to-f64}` | total — no domain hole, no fallback needed |
| `OpClass::Fallback` | `i64::{+ - * / mod rem quot}` | partial — carries `:undefined` |

i64 has both classes. **f64 has neither until this morning, and after the in-flight stone it will have
only the `Alias` half** — `f64::{> < >= <= = not=}`. There is no `:wat::rete::f64::{+ - * /}`, so a
rule can *compare* two floats but cannot *combine* them.

Core has the arithmetic: `:wat::core::f64::{+ - * /}` plus `abs clamp min max min-of max-of round
to-i64 to-string` — all registered, all classified pure ∧ det. **None is `total`**, and `purity.rs`'s
own comment (from `BRIEF-total-column-honest.md`) says exactly why, under the builder's
stricter-than-IEEE rule:

> *"`f64::*` … `eval_f64_arith` dispatches it to a bare `a * b` — raw IEEE 754 multiply, NO overflow
> guard. … NOT total two separate ways: (1) two large finite operands overflow to `±Inf`
> (e.g. `1e200 * 1e200`); (2) `0.0 * f64::INFINITY` … is `NaN` by IEEE 754."*

## Why it is its own stone, not a row-add

The `Alias` rows are a table entry. A `Fallback` row is a **mechanism**: the op must carry an
`:undefined` value for its domain hole, and the where-stone's ruling is that the fallback **differs by
type** —

> *"the fallback differs BY TYPE (`i64::+` overflows, `f64::+` reaches ±Inf), so one generic
> `:undefined` cannot serve both."*

So f64 arithmetic needs its own fallback semantics decided (what does a rule condition mean when its
arithmetic reaches `±Inf` or `NaN`?), not merely transcribed from the i64 rows. That is a design
question with a builder ruling in it, and it is why STOP-1 fenced it off rather than letting a rider
guess.

## Why it matters — do not read this as a nice-to-have

The target use case is rules over streaming anomaly scores (R25, the chaos engine; the lineage
Clara@Shield → the eBPF rule trees → this). Those scores are floats, and a real rule wants arithmetic
in the condition, not only comparison — a ratio, a delta, a normalised score against a floor. The
comparator rows make `(where (:wat::rete::f64::> score 0.8))` expressible. They do not make
`(where (:wat::rete::f64::> (:wat::rete::f64::/ hits total) 0.8))` expressible.

## What closing it needs

1. A ruling on f64's `:undefined` semantics for `±Inf` and `NaN` results — the builder's, not a
   rider's.
2. `OpClass::Fallback` rows for `f64::{+ - * /}` mirroring the i64 fallback shape.
3. A disposition for the non-arithmetic f64 family (`abs`, `clamp`, `min`/`max`, `round`,
   `to-i64`, `to-string`) — several are plausibly total (`abs`, `min`, `max` are in the pure∧det list
   already) and would be `Alias` rows; `to-i64` is partial on `NaN`/`±Inf` and is a `Fallback` or a
   refusal. **Do not guess these — audit each, same discipline as #52.**

## Related, on the disk

- `BRIEF-the-f64-surface-is-a-stub.md` — the in-flight stone this was cut from.
- `DESIGN-STONE-where-admits-only-rete-ops.md` — the per-type ruling, the `:undefined` design, and
  the "what total MEANS" section (builder-ruled, stricter than IEEE).
- `DESIGN-STONE-per-type-equality-restored.md` — the sibling reversal that restored per-type equality
  in core this session.
