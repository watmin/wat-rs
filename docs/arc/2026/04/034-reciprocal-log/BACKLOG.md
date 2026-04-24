# Arc 034 — `:wat::holon::ReciprocalLog` macro — BACKLOG

**Shape:** three slices. Pure-wat stdlib; zero Rust.

---

## Slice 1 — ship the macro

**Status: ready.**

New file `wat/holon/ReciprocalLog.wat`:

```scheme
;; wat/holon/ReciprocalLog.wat — stdlib macro, arc 034.
;;
;; (ReciprocalLog n value) → Log with reciprocal bounds (1/n, n).
;; Ratio-valued indicator encoding for values near 1.0.
;;
;; Expands to (Log value (/ 1.0 n) n). ln-space symmetry is
;; automatic: ln(1/n) = -ln(n). Bounds family: N=2 (±doubling),
;; N=3 (±tripling), N=10 (±10x), etc.

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

Stdlib discovery: `stdlib_forms()` in `src/stdlib.rs` baked-file
array gains an entry for `wat/holon/ReciprocalLog.wat`.

**Sub-fogs:**
- **1a — stdlib registration.** Verify `stdlib_forms()` picks up
  the new file. `wat/holon/*.wat` discovery pattern already
  exists; new file should register automatically via the
  `include_str!` build list. Check at implementation.
- **1b — macro expansion at existing Log test sites.** Arc 032's
  Log tests should continue to pass — they reference `:wat::holon::Log`
  directly, not ReciprocalLog. No regression expected.

## Slice 2 — tests

**Status: obvious in shape** (once slice 1 lands).

New file `wat-tests/holon/ReciprocalLog.wat`. Test claims:

1. **Macro expansion works.** `(ReciprocalLog 2.0 1.0)` encodes the
   same holon as `(Log 1.0 0.5 2.0)` — coincident.
2. **Reciprocal identity.** `(ReciprocalLog 2.0 1.0)` is the
   reference — value 1.0 at the center of (0.5, 2.0). Should
   coincide with itself (trivially true).
3. **Log-symmetric distinguishability.** `(ReciprocalLog 2.0 0.5)` and
   `(ReciprocalLog 2.0 2.0)` produce holons that are NOT coincident
   with the 1.0 reference (they're at the saturation ends).
4. **Different N values produce distinguishable encodings for
   the same value.** `(ReciprocalLog 2.0 1.5)` and
   `(ReciprocalLog 10.0 1.5)` encode differently (one near
   saturation, one near center). Confirms N is an active knob.

Uses arc 031's `make-deftest` + inherited-config shape per the
ergonomic-testing practice.

## Slice 3 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slices 1 + 2 land).

- `docs/arc/2026/04/034-reciprocal-log/INSCRIPTION.md`
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row under the algebra stdlib table for
  `:wat::holon::ReciprocalLog`.
- `docs/README.md` — arc index gains row 034.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

**Sub-fogs:**
- **3a — should 058-017's INSCRIPTION get a pointer to
  ReciprocalLog?** Not required — 058-017 is the Log primitive;
  ReciprocalLog is a pure-macro idiom over it. INVENTORY + arc
  doc is sufficient. Revisit if 058-017 needs an amendment.

---

## Working notes

- Opened 2026-04-23 same session as lab arc 005, which blocked
  on Log bounds for ROC atoms. Observation program showed the
  archive's universal bounds don't translate; per-caller bounds
  under the reciprocal-pair family `(1/N, N)` emerged as the
  first-principles shape.
- Pure-wat macro; sixth cave-quest of the naming-reflex session.
