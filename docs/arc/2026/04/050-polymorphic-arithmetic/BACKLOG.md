# wat-rs arc 050 — polymorphic arithmetic — BACKLOG

**Shape:** four slices. Runtime + check.rs, integration tests,
INSCRIPTION + docs, lab adoption (deferred).

---

## Slice 1 — runtime + check.rs

**Status: ready.**

`src/runtime.rs`:
- Four new dispatch arms in the eval `match head` block (after
  the existing `:wat::core::f64::*` arm, around line 2106):
  - `:wat::core::+` → `eval_poly_arith(head, args, env, sym, PolyOp::Add)`
  - `:wat::core::-` → `eval_poly_arith(head, args, env, sym, PolyOp::Sub)`
  - `:wat::core::*` → `eval_poly_arith(head, args, env, sym, PolyOp::Mul)`
  - `:wat::core::/` → `eval_poly_arith(head, args, env, sym, PolyOp::Div)`
- New `enum PolyOp { Add, Sub, Mul, Div }` (private, in
  runtime.rs alongside the helper).
- New `fn eval_poly_arith(head, args, env, sym, op) -> Result<Value, RuntimeError>`:
  - Arity check: 2 args.
  - Eval both args.
  - Match on `(&a, &b)`:
    - `(i64, i64)` — apply op as i64; div-by-zero check.
    - `(f64, f64)` — apply op as f64; div-by-zero check on Div.
    - `(i64, f64)` — promote LHS, apply as f64; div-by-zero on Div.
    - `(f64, i64)` — promote RHS, apply as f64; div-by-zero on Div.
    - Else — `RuntimeError::TypeMismatch` with helpful "expected numeric pair" message.

`src/check.rs`:
- Four new scheme entries in the builtins registry. The exact
  expression depends on the existing scheme machinery; investigate
  during implementation whether parametric-numeric is supported
  (preferred) or whether a wider Union scheme is needed (fallback).
- Either way, the schemes encode: result type = f64 if either
  arg is f64, else i64.

**Sub-fogs:**
- **1a — scheme expression for polymorphic numerics.** Resolve
  during implementation by reading how comparison-op schemes
  are encoded. They have the same polymorphic-over-numerics
  shape but always return bool; the arithmetic version
  generalizes to "result = wider of two inputs."

## Slice 2 — integration tests

**Status: obvious in shape** (once slice 1 lands).

New `tests/wat_polymorphic_arithmetic.rs`. ~10 tests:

1. `:+ i64 i64 → i64`, e.g. `(:+ 2 3) → 5`.
2. `:+ f64 f64 → f64`, e.g. `(:+ 2.0 3.5) → 5.5`.
3. `:+ i64 f64 → f64`, e.g. `(:+ 2 3.5) → 5.5`.
4. `:+ f64 i64 → f64`, e.g. `(:+ 2.5 3) → 5.5`.
5. `:- mixed`, e.g. `(:- 5 1.5) → 3.5`.
6. `:* mixed`, e.g. `(:* 3 1.5) → 4.5`.
7. `:/ mixed → f64`, e.g. `(:/ 7 2) → 3` (i64); `(:/ 7 2.0) → 3.5` (f64).
8. `:/ i64/0 → DivisionByZero error`.
9. `:/ f64/0.0 → DivisionByZero error`.
10. `:/ i64/0.0 → DivisionByZero error` (mixed-promoted).
11. **Type-check level:** `(:+ "string" 1)` rejected by checker
    or runtime with TypeMismatch.

## Slice 3 — INSCRIPTION + USER-GUIDE + 058 addendum

**Status: obvious in shape** (once slices 1 – 2 land).

- `wat-rs/docs/arc/2026/04/050-polymorphic-arithmetic/INSCRIPTION.md` —
  what shipped, the coexistence-over-migration decision,
  promotion rule, division-by-zero preservation.
- `wat-rs/docs/USER-GUIDE.md` Language core list mention
  (already lists arithmetic operators — add note about
  polymorphic forms).
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row documenting wat-rs arc 050 + the
  "machine-answers-honestly" framing.

## Slice 4 — wat-rs commit + push; lab adoption deferred

**Status: obvious in shape** (once slices 1 – 3 land).

- wat-rs commit + push.
- **Lab does NOT mass-migrate.** New code can use polymorphic
  ops; existing callsites keep typed forms unchanged. If a
  lab arc touches a vocab module and wants to simplify, that's
  per-arc judgment. No big sweep.

**Sub-fogs:**
- (none.)
