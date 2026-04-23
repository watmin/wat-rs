# Arc 019 — `:wat::core::f64::round` primitive

**Status:** opened 2026-04-22.
**Motivation:** the trading lab's Phase 3 encoding helpers need
f64 rounding (`round_to(v, 2)` from the archive, used throughout
vocab for cache-key stability). wat ships `:wat::core::f64::+` /
`-` / `*` / `/` arithmetic and `f64::to-i64` / `f64::to-string`
conversions, but no `round`. Implementing round-half-away-from-
zero via truncation + sign-bias tricks is awkward and wrong-
semantic at edge cases; the substrate deserves the primitive.

Builder framing:

> i feel like round should be in the core lang - let's get it in

Single-primitive arc. Tight scope.

---

## UX target

```scheme
(:wat::core::f64::round 1.00001 0)   ;; → 1.0
(:wat::core::f64::round 12.1234 3)   ;; → 12.123
(:wat::core::f64::round 1.5 0)       ;; → 2.0  (half-away-from-zero)
(:wat::core::f64::round -1.5 0)      ;; → -2.0 (half-away-from-zero)
(:wat::core::f64::round 15.0 -1)     ;; → MalformedForm (digits < 0 rejected)
```

Semantic: `(v * 10^digits).round() / 10^digits`. Round-half-away-
from-zero for finite values; NaN / ±∞ pass through unchanged.
Returns `:f64` (not `:Option<f64>`) because every finite f64 has
a finite rounded form — no fallibility at the round operation
itself.

Signature: `:f64 × :i64 -> :f64`. `digits` must be non-negative
at runtime (rejected as `MalformedForm` otherwise — "round to
nearest 10" has no load-bearing use case and feels like asking
a divide-by-zero question). If a real caller surfaces demand for
negative digits later, a future arc can extend.

---

## Non-goals

- **`floor`, `ceil`, `abs`, `trunc`.** Ship round now; add the
  siblings when a caller surfaces demand. "Three similar lines
  is better than a premature abstraction."
- **`round_even` (banker's rounding).** Not asked for.
- **Integer-valued return.** Stays `:f64 -> :f64`. Callers who
  want an i64 follow round with `f64::to-i64` (arc 014).
- **Overflow guard for extreme digits.** `digits=50` makes
  `10^50` overflow to infinity; the chain produces NaN. Not
  worth guarding — real callers use `digits` in the range
  0..6 for display / cache-stability purposes.
- **Negative `digits`.** Rejected at runtime. Not a hard
  technical limitation; a deliberate scope cut. Reopens with a
  real caller.

---

## What this arc ships

One slice. `:wat::core::f64::round` registered alongside the
existing `:wat::core::f64::*` scheme block in `check.rs`;
dispatched in `runtime.rs` via a new `eval_f64_round`. Lab's
`wat/encoding/round.wat` (Phase 3.1) simplifies to wrap the new
primitive instead of the sign-bias hack.

- `src/check.rs` — one line in the scheme-registration block
  (register `f64::round` with type `:f64 -> :f64`).
- `src/runtime.rs` — one dispatch arm + one `eval_f64_round`
  function (~8 lines) following the `eval_f64_to_i64` shape.
- Unit test — a handful of round-trip cases including negatives
  and NaN / infinity pass-through.
- `holon-lab-trading/wat/encoding/round.wat` — simplify
  `round-to-2` to use the new primitive directly.
- `docs/USER-GUIDE.md`, `docs/arc/.../INSCRIPTION.md`,
  `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`.

---

## Resolved design decisions

- **2026-04-22** — **`:wat::core::f64::round` namespace.** Fits
  alongside `:wat::core::f64::+` (arithmetic) and
  `:wat::core::f64::to-i64` (conversion). The `:wat::std::math::*`
  namespace already holds transcendentals (ln, log, sin, cos);
  round is more primitive than those.
- **2026-04-22** — **Returns `:f64`, not `:Option<f64>`.** Every
  finite input has a finite rounded output; NaN / ±∞ pass
  through. No fallibility at the round operation itself.
- **2026-04-22** — **Round-half-away-from-zero semantic.**
  Matches Rust's `f64::round()`. Matches the archive's
  `(v * factor).round() / factor` convention.
- **2026-04-22** — **Scope: `round` only.** Floor / ceil / abs
  / trunc ship when a concrete caller surfaces demand.

---

## What this arc does NOT ship

- Other unary f64 operations (floor, ceil, abs, trunc).
- Generalized `round_to(v, digits)`.
- Banker's rounding.
- Changes to `:wat::std::math::*`.
