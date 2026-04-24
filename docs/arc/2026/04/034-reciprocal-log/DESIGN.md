# Arc 034 — `:wat::holon::ReciprocalLog` macro

**Status:** opened 2026-04-23. Third `/gaze`-named arc of the
session (after 032 BundleResult + 033 Holons). Cut from lab arc 005's
oscillators port — ROC atoms need Log bounds, observation showed
the archive's universal bounds don't translate, and the smallest-
reciprocal-pair `(1/N, N/1)` emerged as the first-principles
bound-family for ratio-valued indicators.

**Motivation.** `:wat::holon::Log` ships as a 3-arg macro per
058-017: `(Log value min max)` → `Thermometer(ln(value), ln(min),
ln(max))`. Callers supply min/max. For ratio-valued indicators
(ROC, volume-ratio, atr-ratio, variance-ratio, etc.), bounds
pair naturally as `(1/N, N)` — reciprocals around the unit
ratio 1.0:

- `N = 2` → bounds `(0.5, 2.0)` → ±doubling
- `N = 3` → bounds `(1/3, 3.0)` → ±tripling
- `N = 10` → bounds `(0.1, 10.0)` → ±10x

Every ratio-bounded Log call uses this same pattern. Naming it
collapses `(Log value (/ 1.0 n) n)` to `(ReciprocalLog n value)`.

**`/gaze` pass:** `ReciprocalLog` — Level-2-safe. A cold reader
sees `(ReciprocalLog 2.0 value)` and knows the bounds are 2.0
and 1/2.0. "Reciprocal" IS the defining property. Alternatives
considered:

- `BoundedLog` — Level 2 mumble (Log is already bounded).
- `RatioLog` — says what it's for but not how it's shaped.
- `SymmetricLog` — captures log-symmetry but doesn't say "how."

`ReciprocalLog` wins on Level-2-safety.

---

## Semantics

```
(ReciprocalLog n value) = (Log value (/ 1.0 n) n)
                        = (Thermometer (ln value) (ln (/ 1.0 n)) (ln n))
                        = (Thermometer (ln value) (-(ln n)) (ln n))
```

The last line shows the elegance: ln-space symmetry is automatic
because `ln(1/n) = -ln(n)`. No taste-anchored round-number picks;
the log-symmetry falls out of the reciprocal construction.

**Input ranges:**
- `n > 1.0` produces useful bounds. `n ≤ 0` is a positive-log
  violation (ln undefined at 0/negative); runtime will error
  when Thermometer encodes `ln(non-positive)`. Caller guarantees
  `n > 0` as per 058-017 Q2.
- `n = 1.0` is degenerate: `(Log value 1.0 1.0)` → Thermometer
  with zero range → all-saturated. Caller responsibility to
  avoid.

Non-parametric return type (Log returns HolonAST; ReciprocalLog
inherits).

---

## Registration

Pure-wat stdlib file at `wat/holon/ReciprocalLog.wat`. Mirrors
the pattern shipped by Subtract.wat, Log.wat, Circular.wat, etc.
Registered into the stdlib via the baked-file sweep (`stdlib_forms()`
in `src/stdlib.rs` picks up the new file).

```scheme
;; wat/holon/ReciprocalLog.wat
(:wat::core::defmacro
  (:wat::holon::ReciprocalLog
    (n :AST<f64>)
    (value :AST<f64>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Log
     ,value
     (:wat::core::f64::/ 1.0 ,n)
     ,n))
```

Zero Rust. Zero substrate changes. Expands to already-shipped
primitives (`Log`, `f64::/`).

---

## Why substrate, not lab-local

`:wat::holon::*` hosts the algebra. ReciprocalLog is pure algebra
— it doesn't reference any trading-domain concept. Future wat
crates (DDoS rate encoding, MTG resource ratios, any ratio-valued
domain) will want the same macro. One home at the substrate.

---

## What does NOT change

- `:wat::holon::Log`'s signature. Callers can still write the
  explicit 3-arg form. ReciprocalLog is sugar over it.
- Archive-translation bounds. Per arc 005's exploration, the
  archive's (1e-5, 1e5) semantic does NOT translate to Thermometer
  Log (saturation ≠ wrap-around). ReciprocalLog with N=2 or N=3
  gives per-caller bounds matching the indicator's natural range.
- Non-ratio Log callers. Counts (`since-rsi-extreme`), durations
  (`exit-age`), etc., aren't reciprocal-bounded. They keep the
  explicit 3-arg `Log` form with domain-appropriate min/max.

---

## Non-goals

- **Integer N overload.** `:AST<f64>` accepts `2.0` but not `2`
  (integer). Simpler typing; callers write explicit floats.
  Future arc can ship an i64-overload if demand surfaces.
- **Parametric bound families.** ReciprocalLog covers the
  reciprocal (1/N, N) case only. Other symmetric-around-1 shapes
  (like `(1 - δ, 1 + δ)` linear-symmetric instead of ratio-symmetric)
  would need their own macros if they surface.
- **ShipWhatACallerNeeds:** this IS what a caller needs — lab
  arc 005 is the caller.

---

## Follow-through

Lab arc 005 uses `(:wat::holon::ReciprocalLog 2.0 roc-N)` for all
four ROC atoms. Future market vocab (momentum, flow, regime,
etc.) uses the same pattern for ratio-valued indicators they
emit. Each vocab module picks its own N — 2 for mild ratios, 3
or 10 for wider-range ratios — as the indicator's natural range
dictates.

The family (1/N, N) is the pattern; N is the per-indicator knob.
