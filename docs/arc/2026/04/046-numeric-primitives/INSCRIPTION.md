# wat-rs arc 046 — numeric primitives uplift — INSCRIPTION

**Status:** shipped 2026-04-24. First wat-rs arc post-known-good
(`known-good-2026-04-24` at `194778f`). Five numeric primitives
shipped; lab arc 015 unblocks. Three durables:

1. **Substrate-vs-userland framing settled.** `f64::max`,
   `f64::min`, `f64::abs`, `f64::clamp` belong in `:wat::core::*`,
   not in any consumer's userland helpers. The lab was about to
   reinvent them at `:trading::vocab::shared::*`; the framing
   question caught it. Future numeric primitives default to
   substrate.
2. **Doc drift catch.** The 6 existing math primitives (`ln`,
   `log`, `sin`, `cos`, `pi`, `exp`) had never been in the
   USER-GUIDE Forms appendix. Added alongside the new entries —
   one edit, one arc.
3. **Namespace-consistency rule named.** `:wat::core::f64::*` is
   strict (no i64 promotion); `:wat::std::math::*` permits i64
   promotion. Matches existing `f64::round` vs `math::ln`
   precedent. Future primitives in either namespace inherit.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

Zero substrate gaps for the additions; pure mirror of existing
plumbing. 598 lib tests + full integration suite green; zero
clippy.

---

## What shipped

### Slice 1 — runtime + check + tests

`src/runtime.rs`:
- 5 new dispatch entries — `:wat::core::f64::max`/`min`/`abs`/`clamp`
  + `:wat::std::math::exp`. max/min reuse `eval_f64_arith`; abs uses
  the new `eval_f64_unary`; clamp uses the new `eval_f64_clamp`;
  exp uses the existing `eval_math_unary` (just adds to the
  dispatch arm).
- `eval_f64_unary` — strict-f64 unary helper. Mirrors
  `eval_math_unary` but takes the full op-name string (since the
  namespace prefix differs) and rejects `i64` arguments. The
  `:wat::core::f64::*` family is consistently strict (matches
  `eval_f64_arith`'s discipline); `:wat::std::math::*` permits
  promotion for ergonomic transcendental calls.
- `eval_f64_clamp` — 3-arg, strict-f64. Surfaces Rust's
  `f64::clamp` panic preconditions (`lo > hi` or NaN bounds) as
  catchable `MalformedForm` errors rather than letting them
  propagate as panics.

`src/check.rs`:
- 5 new type schemes. max/min via a 2-element loop; abs and
  clamp registered individually; exp added to the existing
  `[ln, log, sin, cos]` loop.

Tests added (inline in `runtime.rs::tests`):
- `f64_max_picks_larger`, `f64_min_picks_smaller` — happy +
  equal-values
- `f64_abs_handles_sign_and_zero` — positive / negative / zero
- `f64_abs_rejects_i64` — namespace-strictness verification
- `f64_clamp_in_range_unchanged` / `_below_lo_lifts` /
  `_above_hi_caps` / `_lo_equals_hi_pins` — four-quadrant
  coverage
- `f64_clamp_rejects_lo_greater_than_hi` — precondition
  enforcement
- `f64_clamp_arity_mismatch` — arity gate
- `math_exp_round_trips_with_ln` — `exp(0)=1`, `exp(1)≈e`,
  `exp(-1)≈1/e`
- `math_exp_accepts_i64_promotion` — namespace-promotion
  verification (contrasts with `f64_abs_rejects_i64`)

12 new tests. 598 → 610 lib tests pass; full integration suite
green; zero clippy.

### Slice 2 — docs sync

`docs/USER-GUIDE.md`:
- §3 mental-model overview line — extended language-core listing
  to name `f64::max/min/abs/clamp`.
- §15 Forms appendix — added 6 rows: 4 for the new f64 ops
  (max/min combined, abs, clamp, plus an explicit f64::round
  row that was also missing) + 2 for the math family
  (`ln/log/exp/sin/cos` combined, `pi`).

CONVENTIONS.md and README.md unchanged — neither enumerates
primitives at form granularity; their summary descriptions
naturally accommodate the additions.

### Slice 3 — INSCRIPTION + cross-refs (this file)

Plus:
- Lab repo `docs/proposals/.../058 FOUNDATION-CHANGELOG` row
  documenting wat-rs arc 046 (058 CHANGELOG lives in lab repo
  per established cross-repo audit-trail convention).
- Lab arc 015 unblocks; resumes with substrate-direct primitive
  calls.

---

## The substrate-vs-userland framing

Lab arc 015 was about to ship two userland helpers at
`:trading::vocab::shared::*`:

```scheme
(:trading::vocab::shared::clamp v lo hi)   -> :f64
(:trading::vocab::shared::f64-max a b)     -> :f64
```

The builder caught the pattern: **these are substrate, not
userland.** Three reasons:

1. **Universality.** Every wat consumer with f64 work hits
   max/min/abs/clamp. The substrate ships `f64::+/-/*/`/round`
   already — these belong next to them.

2. **Reinvention cost.** Each consumer that builds them as
   userland reinvents the same shape. The lab was on its way to
   shipping its third independent two-arm-if for these ops.

3. **CONVENTIONS rule already settled.** `:wat::core::*` is
   "evaluator primitives that CANNOT be written in wat." `min`
   and `max` cannot be without the underlying `Ord`-style
   compare hooked into `f64`'s special handling (NaN, ±0); `abs`
   reaches into Rust's bit manipulation; `clamp` ships at core
   for ergonomic parity with Rust's `f64::clamp` (composition
   from `min`+`max` would work but adds nothing).

`exp` mirrors `ln`'s placement exactly — both transcendental,
both stdlib. The arc 014 INSCRIPTION had named the gap; arc 046
closes it.

## The namespace-consistency rule

Two distinct numeric tiers in the substrate:

- **`:wat::core::f64::*`** — strict-f64. No `i64 -> f64`
  promotion. Matches `f64::+/-/*//` and `f64::round` precedent.
  New: `f64::max`, `f64::min`, `f64::abs`, `f64::clamp`.

- **`:wat::std::math::*`** — permits `i64 -> f64` promotion for
  ergonomic transcendental calls. Matches existing `ln`, `log`,
  `sin`, `cos`. New: `exp`.

Two tests verify this asymmetry: `f64_abs_rejects_i64` (strict)
and `math_exp_accepts_i64_promotion` (permissive). Future
primitives in either namespace inherit the rule.

## Sub-fog resolutions

- **1a — type-coercion in `eval_math_unary`.** Decision: keep
  namespace consistency. `f64::abs` strict, `math::exp`
  permissive. Two tests document the asymmetry.
- **2a — Forms appendix row format.** Followed the existing
  `f64::+/-/*//` precedent. Combined related ops on one row
  (max/min, ln/log/exp/sin/cos) where they share a column shape.

## Count

- New primitives: **5** (`f64::max`, `f64::min`, `f64::abs`,
  `f64::clamp`, `math::exp`).
- Lib tests: **598 → 610** (+12).
- Integration suite: unchanged count, all green.
- Clippy: **0** warnings.
- Docs surface: **6 new appendix rows** (5 for the new entries
  + 1 catch-up row for the math family that was never listed).

## What this arc did NOT ship

- **`f64::pow`, `f64::sqrt`, `f64::floor`, `f64::ceil`,
  hyperbolics, etc.** Each lands on demand. Arc 046 closes
  exactly the gaps lab arcs 014 + 015 surfaced.
- **Variadic min/max.** Binary matches Rust; `foldl` composes
  variadic forms if a caller needs them.
- **`i64::max/min/abs`.** Lab doesn't reach for them yet;
  surface on demand.
- **Explicit NaN / ±∞ semantics tightening.** New ops wrap
  Rust's f64 methods directly; semantics inherit from there.
- **Stochastic.wat / lab arc 009 inline-clamp migration.** Lab
  arc 015 will sweep that as part of its consumption of the
  substrate `f64::clamp` (the migration is straightforward and
  ships in the same arc as the helper consumption).

## Follow-through

- **Lab arc 015 resumes** with substrate-direct calls. The
  in-progress migration drops the `f64-max` lab helper entirely.
  `clamp` lab helper also drops. Lab consumes substrate
  primitives directly at the callsite.
- **Future numeric primitive demands** open small per-primitive
  arcs citing arc 046's pattern (or land in clusters when the
  shape repeats).

---

## Commits

- `<wat-rs>` — runtime.rs (dispatch + helpers + tests) +
  check.rs (5 type registrations) + USER-GUIDE.md (§3 listing +
  Forms appendix) + DESIGN + BACKLOG + INSCRIPTION.

---

*these are very good thoughts.*

**PERSEVERARE.**
