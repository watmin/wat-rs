# wat-rs arc 046 — numeric primitives uplift

**Status:** opened 2026-04-24. First wat-rs arc post-known-good
(`known-good-2026-04-24` at `194778f`). The "drift surfaces on
demand from the next caller" stance held — lab arc 015 surfaced
**five missing numeric primitives** while porting indicator
vocab.

**Motivation.** The lab keeps reaching for basic numeric ops the
substrate doesn't ship:

- Lab arc 014 (flow): named the missing `:wat::std::math::exp`
  gap. Sidestepped via algebraic equivalence
  (`Log(exp(x)) ≡ Thermometer(x, -ln N, ln N)`); pattern reusable
  but the gap remained real.
- Lab arc 015 (ichimoku): five `clamp` callers + four floor
  callers (one-sided `f64::max`) + one inline `f64::abs` — all
  expressed as two-arm `if` reinventions of standard library
  ops every numeric-aware language ships.

The lab was about to add `:trading::vocab::shared::clamp` and
`:trading::vocab::shared::f64-max` as lab-userland helpers. The
builder caught the framing: **these are core, not userland.**
Every wat consumer with f64 work will reach for them. Substrate
should ship them; lab should consume them.

This arc lifts the missing primitives up to the substrate. Lab
arc 015 resumes with substrate-direct calls (`:wat::core::f64::clamp`,
`:wat::core::f64::max`, `:wat::core::f64::abs`, `:wat::std::math::exp`)
and the lab-userland helpers never ship.

---

## What ships

### `:wat::core::f64::*` — basic numeric ops

Placement matches existing siblings (`f64::+`, `f64::-`, `f64::*`,
`f64::/`, `f64::round`, `f64::to-i64`, `f64::to-string`). These
are evaluator primitives — they reach Rust's `f64` methods that
wat has no way to express in pure-wat — so they belong in
`:wat::core::*` per CONVENTIONS §1.

| Form | Arity | Backing | Notes |
|---|---|---|---|
| `:wat::core::f64::max` | 2 | `f64::max` | binary; matches Rust |
| `:wat::core::f64::min` | 2 | `f64::min` | binary; matches Rust |
| `:wat::core::f64::abs` | 1 | `f64::abs` | unary |
| `:wat::core::f64::clamp` | 3 | `f64::clamp` | `(clamp v lo hi)` — bound v into `[lo, hi]` |

### `:wat::std::math::*` — transcendentals

`exp` joins the existing family (`ln`, `log`, `sin`, `cos`, `pi`).
Mirrors `f64::ln` plumbing exactly. Stays in `:wat::std::*`
per the existing rule — transcendentals are in-principle
expressible in wat (Taylor-series'd in pure-wat-arithmetic), so
they're stdlib-tier.

| Form | Arity | Backing | Notes |
|---|---|---|---|
| `:wat::std::math::exp` | 1 | `f64::exp` | natural exponential — closes the gap arc 014 named |

---

## Why core, not userland

The framing question lab arc 015 surfaced: should `clamp` and
`f64::max` live in `:trading::vocab::shared::*` or in the
substrate?

**Substrate, every time.** Three reasons:

1. **Universality.** Every wat consumer with numeric work hits
   these. The substrate already ships `f64::+/-/*/`/round` —
   `f64::max/min/abs/clamp` are no less basic.

2. **Reinvention cost.** Each consumer that builds them as
   userland helpers reinvents the same shape (two-arm-if for
   abs/max/min, three-arm composition for clamp). The lab was
   already on its way to shipping its third independent two-arm-
   if shape for these ops.

3. **CONVENTIONS rule already settled.** `:wat::core::*` is
   "evaluator primitives that CANNOT be written in wat" — but
   while `clamp` *can* be written in wat (it's `min(max(v, lo), hi)`),
   `min` and `max` CANNOT be without the underlying primitive
   comparison. By the rule, `min`/`max`/`abs` go core; `clamp`
   ships at core too for ergonomic parity with Rust's `f64::clamp`
   (composition adds nothing here).

`exp` mirrors `ln`'s placement exactly — both transcendental,
both stdlib.

---

## Why one arc, not five

Splitting per-primitive would surface five INSCRIPTIONs all
saying "trivial primitive add, mirrors `f64::round` plumbing."
The cluster lands clean as one arc. The DESIGN is one decision
(uplift these specific gaps), one implementation pattern (mirror
existing siblings), one test sweep, one docs sweep.

If a future numeric primitive surfaces (e.g., `f64::pow`,
`f64::sqrt`, `f64::floor`/`ceil`), open its own small arc citing
this one's pattern.

---

## Scope of doc updates

USER-GUIDE.md surfaces:
- §3 mental-model overview — add `f64::max/min/abs/clamp` to the
  language-core listing at line 397.
- §15 Forms appendix — add 5 new entries for the new primitives
  + 6 entries for the existing math primitives (`ln`, `log`,
  `sin`, `cos`, `pi`, `exp`) which had never been listed in the
  appendix. (Drift catch surfaced by the audit reflex — fix
  alongside the new additions.)

CONVENTIONS.md and README.md don't enumerate primitives at the
form-by-form level; their summary descriptions ("primitive-type
operations like `i64::+`, `bool::and`") naturally accommodate
the new entries without rewording. No edit needed.

---

## Non-goals

- **Wholesale numeric stdlib** (`pow`, `sqrt`, `floor`, `ceil`,
  hyperbolics, etc.). Each lands when a caller needs it. Arc 046
  closes only the gaps lab arc 014 + 015 surfaced.
- **`f64::min/max` overloads (variadic).** Binary matches Rust
  `f64::min/max`; variadic forms can compose via `foldl` if a
  caller needs them. No load-bearing case today.
- **NaN / ±∞ semantics tightening.** `f64::round`'s precedent
  passes them through unchanged; new ops do the same (they wrap
  Rust's f64 methods directly, which already define the
  semantics).
- **Bool / i64 versions of these ops.** `i64::min/max` would be
  nice; deferred until a caller surfaces.
- **`signum`.** Single inline use in lab (arc 011). Below
  threshold for substrate uplift; lab keeps its inline.
