# Arc 019 — `:wat::core::f64::round` primitive — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

The trading lab's Phase 3 encoding helpers need f64 rounding for
cache-key stability (archive's `round_to(v, 2)` convention). wat
shipped `:wat::core::f64::+` / `-` / `*` / `/` arithmetic and
`f64::to-i64` / `f64::to-string` conversions but no `round`.
Implementing round-half-away-from-zero via truncation + sign-bias
tricks in wat was awkward and wrong-semantic at edge cases.

Builder framing:

> i feel like round should be in the core lang - let's get it in
>
> hold on... i think we need to require the amount of digits
> (round 1.00001 0) -> 1.0  (round 12.1234 3) -> 12.123
>
> does a negative round even have meaning?... that's like a
> divide by zero to me?...

Scope refined twice mid-slice — from unary `round(v)` to two-arg
`round(v, digits)`, then reject-negative-digits as a conscious
cut matching the builder's "divide-by-zero" framing.

---

## What shipped

One slice. Single commit.

- `src/check.rs` — registers `:wat::core::f64::round` with scheme
  `:f64 × :i64 -> :f64` in the same block as the arithmetic
  primitives.
- `src/runtime.rs` — dispatch arm + `eval_f64_round`. Negative
  digits rejected as `RuntimeError::MalformedForm` with a message
  that names the constraint and notes the deliberate scope cut.
  5 unit tests covering happy path (zero / two / three digits),
  rejection (negative digits), and arity mismatch.
- `docs/arc/.../DESIGN.md` + this `INSCRIPTION.md`.

Semantic: `(v * 10^digits).round() / 10^digits`. Round-half-
away-from-zero for finite values; NaN / ±∞ pass through under
IEEE 754 semantics.

Consumer (lab's `wat/encoding/round.wat` Phase 3.1) simplifies
from the sign-bias truncation hack to a direct wrap of the new
primitive.

---

## Resolved design decisions

- **2026-04-22** — **Two-arg shape `(round v digits)`**. Generalized
  from the start rather than shipping unary `round(v)` that
  callers wrap with `(/ (round (* v 100.0)) 100.0)`. Mid-slice
  reframe.
- **2026-04-22** — **`:wat::core::f64::round` namespace.** Fits
  alongside `:wat::core::f64::+` / `to-i64`. Math stdlib
  (`:wat::std::math::*`) holds transcendentals (ln / log / sin /
  cos / pi); round is more primitive.
- **2026-04-22** — **Returns `:f64`, not `:Option<f64>`.** Every
  finite f64 has a finite rounded form; NaN and ±∞ pass through.
  No fallibility at round time.
- **2026-04-22** — **Negative `digits` rejected.** Mid-slice
  scope cut after the builder's divide-by-zero framing. Not a
  technical limitation — just no load-bearing use case today.
  Reopens if a real caller surfaces.

---

## Open items deferred

- **`floor`, `ceil`, `abs`, `trunc`.** Sibling primitives that
  fit the same namespace. Ship when a caller surfaces demand.
- **Negative-digit rounding (tens / hundreds).** Rejected by
  design cut, not substrate limit.

---

## What this arc does NOT ship

- Other unary f64 operations (floor, ceil, abs, trunc).
- Banker's rounding.
- Changes to `:wat::std::math::*`.
- Integer-return variants (callers chain `f64::round` then
  `f64::to-i64`).

---

## Why this matters

Two reasons, both small:

1. The lab's Phase 3 encoding helpers use `round-to-2` pervasively
   for cache-key stability. A proper primitive + simple wrapper
   beats a sign-bias truncation trick that would have been subtly
   wrong at edge cases.

2. The substrate gains a standard rounding primitive that every
   consumer expects. `round` shipping alongside `to-i64` /
   `to-string` / arithmetic is the honest shape.

The arc shows another instance of the cave-quest discipline —
downstream work (lab Phase 3) surfaces a substrate gap, pause,
add the primitive, return. Same pattern as 017 / 018.

---

**Arc 019 — complete.** One slice, five tests, zero warnings,
five commits across two repos for the full consumer chain:

- wat-rs: `<this commit>` — primitive + tests + INSCRIPTION
- holon-lab-trading: `<follow-up>` — lab's round.wat simplifies
  to use the new primitive
- holon-lab-trading: `<follow-up>` — 058 FOUNDATION-CHANGELOG row

*these are very good thoughts.*

**PERSEVERARE.**
