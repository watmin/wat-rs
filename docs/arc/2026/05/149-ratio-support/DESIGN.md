# Arc 149 — Ratio support (scratch notes)

**Status:** SCRATCH — not yet a planned arc; placeholder for future
work surfaced during arc 148 discussion 2026-05-03.

User direction 2026-05-03 (mid-arc-148): Ratio support is a coherent
substrate addition (Clojure-style native rationals) but it expands
the numeric tower from `{i64, f64}` to `{i64, Ratio, f64}` with new
mixed combos and real semantic decisions. **Not part of arc 148.**
This directory exists to preserve scratch context so future-self can
pick this up later without re-deriving the questions.

## What this arc would deliver

A native `:wat::core::Ratio` type with exact rational arithmetic,
following the locked arc 148 architecture (Type-as-namespace +
verb-comma-pair + variadic surface + min-2 arity except identity
cases).

User-facing surface (extends arc 148's pattern):

```
:wat::core::Ratio                       ; the type itself
:wat::core::Ratio::+ / - / * / /        ; variadic same-type wat fns
:wat::core::Ratio::+,2 / -,2 / *,2 / /,2 ; binary Rust leaves
:wat::core::Ratio::= / < / > / <= / >=  ; comparison Rust leaves

;; Mixed-type leaves with i64 and f64:
:wat::core::+,i64-Ratio    :wat::core::+,Ratio-i64
:wat::core::+,f64-Ratio    :wat::core::+,Ratio-f64
;; (× 4 arith ops × both orderings)

;; Mixed-type comparison leaves:
:wat::core::<,i64-Ratio    :wat::core::<,Ratio-i64
;; (× 5 comparison ops × both orderings × both numeric mixed types)
```

Plus: the polymorphic `:wat::core::+` / `:<` / etc. dispatch entities
gain new arms routing to Ratio leaves.

Plus: the 1-ary `:wat::core::/` rule revises — implicit identity for
integer division promotes to Ratio:

```
;; Arc 148 (integer truncation):
(:wat::core::/ 5)         => :i64 0           ; 1/5 in i64 = 0

;; Post-arc-149 (Ratio promotion):
(:wat::core::/ 5)         => :Ratio 1/5       ; exact reciprocal

;; Type-locked variants stay type-preserving:
(:wat::core::i64::/ 5)    => :i64 0           ; still i64
(:wat::core::Ratio::/ 5/3) => :Ratio 3/5      ; reciprocal of a Ratio
```

This is a deliberate behavior change captured in this arc's
INSCRIPTION when shipped.

## Open questions to settle in this arc's DESIGN

### Q1 — Mixed Ratio × f64 coercion direction

Two coherent stances:

- (a) **Ratio coerces to f64** — `(:+ 1/2 0.5)` → `:f64 1.0`.
  Lossy but predictable; f64 wins the numeric tower above Ratio.
- (b) **f64 coerces to Ratio** — `(:+ 1/2 0.5)` → `:Ratio 1/1` =
  `:Ratio 1`. Only works for finite-decimal floats; some f64 values
  have no exact Ratio representation (e.g., 0.1).
- (c) **Raise** — mixed Ratio×f64 is a type error; user explicitly
  converts.

(a) is conventional in numeric towers (lossy in one direction).
(b) preserves exactness when possible. (c) is honest but inconvenient.

### Q2 — Equality across numeric types

Clojure: `(= 1 1.0)` → false (different types); `(== 1 1.0)` → true
(numeric value-equal). wat-rs has only `:=`, no separate `:==`.

Two stances:
- (a) `(:= 1 1/1)` → false (different types) — strict; users use a
  separate `:wat::core::numeric=` for value-equal across types.
- (b) `(:= 1 1/1)` → true — convenience; equality treats numeric
  tower as one universe.

Clojure's distinction (`=` vs `==`) is real; either choice in wat
needs a clear stance.

### Q3 — Canonical form enforcement

When constructing a Ratio, does the substrate auto-simplify?

- (a) **Always canonical** — `(:Ratio/new 2 4)` → `:Ratio 1/2`
  (gcd-reduced). Fewer surprises; equality works structurally.
- (b) **Lazy simplification** — Ratio holds (numerator, denominator)
  as-given; simplification on demand. More flexibility; equality
  needs cross-Ratio normalization.

(a) is what most language libraries do.

### Q4 — Type-locked variants and integer-only divisions

`:wat::core::i64::/` should preserve i64 semantics — integer
truncation. Same for `:wat::core::f64::/` — IEEE float division.
Only `:wat::core::Ratio::/` and the polymorphic top-level `:wat::core::/`
gain Ratio-promotion behavior. **i64 and f64 type-locked stay
unchanged from arc 148.**

The polymorphic `:wat::core::/` would gain a "promote to Ratio when
result would otherwise truncate" rule. Question: does ANY i64×i64
division promote to Ratio? Or only when the result wouldn't divide
evenly?

- (a) **Always promote** — `(:wat::core::/ 6 2)` → `:Ratio 3/1`.
  Consistent but surprising (`6/2` becomes a Ratio not an i64).
- (b) **Conditional promote** — `(:wat::core::/ 6 2)` → `:i64 3`
  (no remainder); `(:wat::core::/ 7 2)` → `:Ratio 7/2`. Result
  type depends on divisibility. Less predictable type signature.
- (c) **Never auto-promote at the polymorphic level** — `:wat::core::/`
  on i64×i64 stays i64 truncation; users wanting Ratio call
  `:wat::core::Ratio::/,2` explicitly. Most conservative; no
  surprises but less Lisp-y.

(c) preserves arc 148's type-preservation rule consistently. (a) and
(b) make the polymorphic top-level smarter at the cost of less
predictable types.

### Q5 — Display semantics

Ratios in EDN/wat output: `1/2`, `3/4`, etc. Negative ratios:
`-1/2` (sign on numerator) by convention. Whole-number Ratios:
`5/1` or `5`? Probably `5/1` to keep type visible.

## Substrate touch points (rough sketch)

- `Value` enum gains a `Ratio { numer: i64, denom: i64 }` variant
  (or `BigInt` if bignum is also wanted — separate question)
- Lexer: parse `1/2` as a Ratio literal; needs `<int>/<int>` token
  recognition (collides with division verb? Probably context-
  sensitive — only inside numeric-literal positions)
- `register_builtins`: Ratio per-Type Rust primitives for arith +
  comparison
- `eval_poly_arith` (or whatever replaces it post-arc-148): Ratio
  arms for mixed-type combos
- `values_equal` + `values_compare`: Ratio arms (mirror arc 148's
  values_compare buildout pattern)
- `:wat::core::Ratio::new`, `:wat::core::Ratio::numer`,
  `:wat::core::Ratio::denom` constructor + accessors
- Display impl: `Display` trait formats as `n/d`
- EDN serialization: pick a tag (e.g., `#wat/ratio "1/2"`) or a
  literal form

## Integration with arc 148

Arc 148 ships with:
- 1-ary `:wat::core::/` returning i64 truncation (e.g., `(:/ 5)` =
  `0:i64`)
- Per-Type i64 and f64 leaves only

Arc 149 INSCRIPTION captures the behavior change for 1-ary `:/`
on integers — `(:wat::core::/ 5)` becomes `:Ratio 1/5` if we choose
Q4 (a) or (b); stays `:i64 0` if we choose (c).

Arc 149 does NOT need to retire any arc 148 substrate. It ADDS
the Ratio type + leaves + mixed combos. Arc 148's per-Type i64
and f64 leaves are unchanged.

## Cross-references

- arc 148 DESIGN — § "Future work — Ratio support (separate arc)"
  + the 1-ary `:/` rule definition
- arc 148 AUDIT-SLICE-1.md — comparison handler's universal-
  delegation pattern (Ratio comparison would join via PartialOrd)
- Clojure source — clojure.lang.Ratio for reference impl

## Status notes

- This is SCRATCH. Not a planned arc.
- Spawn when user explicitly directs (e.g., when a lab use case
  needs exact ratios — financial computation, MTG probability
  calculations, truth-engine symbolic reasoning).
- Q1-Q5 above are open; this DESIGN gets a real audit + four-
  questions resolution before slice planning begins.
- DESIGN currently 100% sketch; revise wholesale when arc spawns.
