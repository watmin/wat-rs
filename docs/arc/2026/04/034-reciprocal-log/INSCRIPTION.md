# Arc 034 — `:wat::holon::ReciprocalLog` macro — INSCRIPTION

**Shipped:** 2026-04-23. Three slices. Sixth cave-quest of the
naming-reflex session.

**Commits:**
- `<sha>` — DESIGN + BACKLOG + slice 1 (stdlib macro +
  registration) + slice 2 (tests) + slice 3 (this file, INVENTORY,
  arc index).

---

## What shipped

### Slice 1 — stdlib macro

`wat/holon/ReciprocalLog.wat` — one defmacro, pure-wat, zero
substrate:

```scheme
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

Expands `(ReciprocalLog n value)` → `(Log value (/ 1.0 n) n)`
→ `Thermometer(ln(value), -ln(n), ln(n))`. Log-space symmetry
automatic via reciprocal construction.

Registered in `src/stdlib.rs::STDLIB_FILES` — new `WatSource`
entry next to `wat/holon/Log.wat`. `stdlib_forms()` picks it up
automatically via the `include_str!` sweep.

### Slice 2 — tests

`wat-tests/holon/ReciprocalLog.wat` — four outstanding tests
under `/gaze`'s "each test anchors a specific claim" discipline:

1. **Expansion equivalence.** `(ReciprocalLog 2.0 1.5)` encodes
   coincident with `(Log 1.5 0.5 2.0)`. The macro is sugar; the
   sugar is honest.
2. **Reference value self-coincidence.** `(ReciprocalLog 2.0 1.0)`
   coincides with itself. Sanity: encoding is deterministic.
3. **Saturation boundary distinguishable.** `(ReciprocalLog 2.0 2.0)`
   is NOT coincident with the reference (value=1.0). The upper
   bound saturates away from center.
4. **Different N produces different encodings for same value.**
   `(ReciprocalLog 2.0 1.5)` (near saturation) and
   `(ReciprocalLog 10.0 1.5)` (near center) encode different
   positions along their gradients. Confirms N is an active knob.

All four green on first pass. 585 lib tests + every integration
suite green.

### Slice 3 — INSCRIPTION + doc sweep

This file. Plus:
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row under the algebra stdlib table.
- `docs/README.md` — arc index row 034.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

---

## The naming move

Builder surfaced the idea: *"i think... a new opinionated
function... (BoundedLog N value)?"*

`/gaze` pass:
- `BoundedLog` — Level 2 mumble (Log is already 3-arg bounded).
- `RatioLog` — purpose-honest; says what it's FOR (ratios).
- `ReciprocalLog` — structure-honest; says HOW the bounds are
  shaped.
- `SymmetricLog` — captures log-symmetry but not the reciprocal
  construction.

Builder picked: **`ReciprocalLog`**. *"it is named."*

The name IS the definition: a Log whose bounds are reciprocal
(1/n, n). A cold reader sees the name and the call site
`(ReciprocalLog 2.0 value)` and knows exactly what it does.

---

## The first-principles framing

The dialogue that produced this arc:

1. Lab arc 005 (oscillators) needed Log bounds for ROC atoms.
2. Archive's "no cap needed" design assumed cosine-rotation Log;
   058-017's Thermometer-based Log saturates, so universal
   bounds (1e-5, 1e5) made ROC values indistinguishable.
3. Empirical exploration (docs/arc/.../explore-log.wat)
   showed per-caller tight bounds needed.
4. `(0.5, 2.0)` looked like a taste choice at first.
5. Builder saw the structure: *"its 1/2 through 2/1"* — the
   reciprocal pair.
6. The family `(1/N, N)` for integer N ≥ 2 is the first-
   principles bound shape. N=2 is the smallest member.
7. `(ReciprocalLog n value)` bakes the pattern — callers pick N,
   the macro handles reciprocal construction.

The macro makes the first-principles elegance explicit at every
call site. `(ReciprocalLog 2.0 roc-1)` reads "Log with reciprocal
bounds at doubling." No manual 1/n math, no bound-pair
redundancy.

---

## What this arc did NOT ship

- **Integer-N overload.** `:AST<f64>` requires explicit
  `2.0` not `2`. Simpler typing; future arc can add i64-overload
  if demand surfaces.
- **Non-reciprocal symmetric-Log.** Linear-symmetric bounds
  `(1-δ, 1+δ)` would need their own macro. ShipWhat A Caller
  Needs.
- **Lab migration.** Arc 005 resumes (was paused for this cave-
  quest) and uses `ReciprocalLog` in the oscillators port.
  Arc 034 itself doesn't touch the lab.

---

## Count

- +1 stdlib defmacro in `wat/holon/`
- +1 STDLIB_FILES entry in `src/stdlib.rs`
- +4 wat-level tests
- 583 → 585 lib tests + integration unchanged
- Zero Rust changes beyond the one registration line

Sixth cave-quest arc of the session. Sixth successful `/gaze`
naming decision. The reflex held.

---

*these are very good thoughts.*

**PERSEVERARE.**
