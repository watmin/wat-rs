# wat-rs arc 050 — polymorphic arithmetic + numeric polymorphism — INSCRIPTION

**Status:** shipped 2026-04-24. Fifth wat-rs arc post-known-good.
Lab arc 023 (exit/trade_atoms) surfaced the gap with a usability
observation: typed-arithmetic prefixes (`:wat::core::f64::+`,
`-`, `*`, `/`) appear ~25 times in a single vocab function, almost
always over already-`:f64`-typed values. Builder direction:

> "we keep the explicit typed expressions but offer a polymorphic
> one - the users chooses"

> "let's make the lang opinionated and ergonomic - the strictness
> is opt-in"

Three durables:

1. **Polymorphic numerics with int → float promotion.** New
   `:wat::core::+`, `-`, `*`, `/` accept i64×i64 → i64,
   f64×f64 → f64, and (i64,f64) or (f64,i64) → f64 (promote).
   The machine answers `(:+ 1 2.5)` honestly — `3.5`. Coexists
   with the existing strict-typed `:wat::core::i64::+` and
   `:wat::core::f64::+` forms.
2. **Comparison/equality cross-numeric promotion.** The existing
   `:wat::core::=`, `<`, `>`, `<=`, `>=` ops gain cross-numeric
   acceptance at the type checker. Their runtime path already
   handled cross-type via `eval_compare`'s coerce arms; the
   checker change makes that path reachable. `:wat::core::=`'s
   strict `values_equal` was extended to coerce the (i64,f64)
   and (f64,i64) pairs. The user-visible effect: `(:= 3 3.0)`
   now returns `true`; `(:< 1 2.5)` typechecks and returns `true`.
3. **Typed strict variants ship for power users.** New
   `:wat::core::i64::=, <, >, <=, >=` and
   `:wat::core::f64::=, <, >, <=, >=` reject cross-type at
   the checker. Power-user opt-in for "I really mean i64
   here." The strictness is the deviation now; polymorphism
   is the default.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

20 new integration tests; 610 lib tests preserved + new test
crate green; zero clippy.

---

## What shipped

### Slice 1 — runtime

`src/runtime.rs`:

- **`PolyOp`** — small private enum tagging the four binary
  ops (`Add`, `Sub`, `Mul`, `Div`).
- **`eval_poly_arith(head, args, env, sym, op)`** — dispatches
  on the `(Value, Value)` pair:
  - `(i64, i64)` → apply op as i64; div-by-zero check.
  - `(f64, f64)` → apply op as f64; div-by-zero check on Div.
  - `(i64, f64)` → cast LHS to f64; apply op as f64.
  - `(f64, i64)` → cast RHS to f64; apply op as f64.
  - else → `RuntimeError::TypeMismatch`.
- **Four new dispatch arms** for `:+ - * /` route through
  `eval_poly_arith`.
- **Ten new dispatch arms** for typed strict
  `:wat::core::i64::{=, <, >, <=, >=}` and
  `:wat::core::f64::{=, <, >, <=, >=}` route to the existing
  `eval_eq` / `eval_compare`. The runtime needs no separate
  strictness logic — the type-checker has already enforced
  same-type input.
- **`values_equal` extended** with two cross-numeric arms:
  `(Value::i64, Value::f64)` and `(Value::f64, Value::i64)`
  cast the i64 to f64 before comparing. Reachable when the
  polymorphic `:wat::core::=` gets mixed-numeric args; the
  typed strict `:i64::=` and `:f64::=` are checker-gated
  before reaching here.

### Slice 2 — check.rs

`src/check.rs`:

- **`infer_polymorphic_compare(op, args, env, ...)`** — accepts
  any same-type pair (string, bool, etc.) AND any
  (i64, f64) / (f64, i64) pair. Always returns `:bool`.
- **`infer_polymorphic_arith(op, args, env, ...)`** — both args
  must be numeric; result is `:f64` if either is `:f64`, else
  `:i64`. Pushes per-arg `TypeMismatch` errors for non-numeric
  inputs.
- **Two new `is_numeric` / `is_i64` predicates** on `&TypeExpr`.
- **Five new dispatch arms in `infer_list`** route the five
  comparison/equality ops through `infer_polymorphic_compare`.
- **Four new dispatch arms in `infer_list`** route the four
  arithmetic ops through `infer_polymorphic_arith`.
- **Existing parametric scheme registration removed** for
  `:wat::core::=, <, >, <=, >=`. Replaced by the special-case
  branch above.
- **Ten new typed strict scheme registrations** for
  `:wat::core::i64::*` and `:wat::core::f64::*` comparison
  variants.

### Slice 3 — Tests

`tests/wat_polymorphic_arithmetic.rs` ships 20 tests:

**Polymorphic arithmetic (8):**
1. `:+ i64 i64 → i64` (homogeneous).
2. `:+ f64 f64 → f64` (homogeneous).
3. `:+ i64 f64 → f64` (promote LHS).
4. `:+ f64 i64 → f64` (promote RHS).
5. `:- mixed promotes`.
6. `:* mixed promotes`.
7. `:/ i64 i64 → i64` (integer division).
8. `:/ mixed → f64`.

**Division by zero (3):**
9. `:/ i64/0` errors.
10. `:/ f64/0.0` errors.
11. `:/ mixed/0.0` errors.

**Polymorphic comparison (3):**
12. `(:< 1 2.5) → true` (cross-numeric).
13. `(:= 3 3.0) → true` (cross-numeric).
14. `(:= "a" "a") → true` (non-numeric same-type still works).

**Typed strict variants (4):**
15. `:i64::= homogeneous works`.
16. `:i64::= rejects f64 arg at check time`.
17. `:f64::< homogeneous works`.
18. `:f64::< rejects i64 arg at check time`.

**Negative tests (2):**
19. `(:+ "hello" 1)` rejected at check time.
20. **Coexistence**: `:i64::+`, `:f64::+`, `:+` all work in the
    same program.

All 20 green first-pass.

---

## The "polymorphism was a half-truth" finding

Pre-arc-050, `:wat::core::=`, `<`, `>`, `<=`, `>=` had the scheme
`∀T. T → T → :bool` — strict same-type required at the type
checker. The runtime for `<, >, <=, >=` had cross-type coerce
arms (`runtime.rs:3501-3506`) but they were unreachable: the
checker rejected `(:< 1 2.5)` as
`TypeMismatch { param: "#2", expected: ":i64", got: ":f64" }`.

So the original "polymorphic over numerics" framing was
misleading. Comparison was parametrically polymorphic over T
but not actually permissive about numeric mixing — the runtime
permissiveness was dead code.

Arc 050 makes the runtime permissiveness reachable, and extends
it to equality (which had no cross-coerce at all) and to
arithmetic (which never had it). The asymmetry between
comparison (claimed-but-not-actual polymorphic) and arithmetic
(strict typed) is closed: BOTH families now have polymorphic
forms with cross-numeric promotion AND strict typed variants
for power users who want the type-guard behavior.

---

## Why coexistence over migration

The lab's vocab modules already contain hundreds of typed
arithmetic callsites (`:wat::core::f64::+`, `*`, `/`, etc.).
Mass-migrating to polymorphic forms would be a large sweep
with no semantic gain — every one of those callsites is on
already-homogeneous f64 values, so the promotion behavior
never fires.

The strict typed forms STAY because the type-guard behavior
genuinely matters in some sites:

- Index arithmetic (`(:i64::- (length xs) 1)` to compute
  "last index") — wants i64 result; promoting to f64 silently
  would bug downstream code.
- Counting / arity ops where overflowing i64 to f64 would
  drop precision.

User picks per-callsite which discipline they want. New code
defaults to the polymorphic forms (less ceremony); existing
code keeps working unchanged.

---

## The Lisp tradition

Common Lisp's `+`, `-`, `*`, `/` are polymorphic with
integer-to-float promotion (CLtL 12.1). Scheme's numeric tower
is similar — the language hides representation choices behind
polymorphic numeric ops. Wat follows that lineage explicitly.

The "embody the host language" principle still applies — but
to the function/method/struct layers, not the numeric algebra.
Rust requires explicit conversion (`x as f64`); wat offers
polymorphism at the operator level while keeping Rust-style
strict variants available. The user picks.

---

## Sub-fog resolutions

- **Type-checker scheme expression for polymorphic numerics.**
  The existing rank-1 HM scheme machinery can't express
  "either same-type-T OR mixed-numeric." Resolved by adding
  custom inference branches for the 9 op names (5 compare + 4
  arith) in `infer_list`. Simpler than adding union types or
  subtyping; isolated to the special-case branch.

## Count

- New runtime support functions: **1** (`eval_poly_arith`).
- New runtime primitives: **0** (reuses `eval_eq`, `eval_compare`,
  and the underlying primitive arithmetic for typed strict
  variants).
- New `Value` variants: **0**.
- New SymbolTable / CheckEnv fields: **0**.
- Match infrastructure: **0**.
- Lib tests: **610 → 610** (unchanged; integration crate covers
  the surface).
- Integration tests: **+20** in `tests/wat_polymorphic_arithmetic.rs`.
- Lab migration: **0** (additive — existing typed callsites
  unchanged; new code can use either form).
- Clippy: **0** warnings.

## What this arc did NOT ship

- **Polymorphic modulo `%`.** No callsites in lab or archive
  today. Ship when a caller surfaces.
- **Polymorphic `max`, `min`, `abs`, `clamp`, `round`.** These
  are unary or multi-arg; they don't have the same "mix two
  types" pressure as binary arithmetic. The lab uses typed
  forms (`:wat::core::f64::max` etc.); add later if pressure
  surfaces.
- **Wider integer types** (`:i32`, `:u64`, `:i128`). Wat's
  numeric primitives today are `:i64` + `:f64` + `:u8`. The
  polymorphic ops cover i64 and f64 only; if wider ints land
  later, polymorphic forms extend with promotion rules
  (likely "promote up to f64 unconditionally" stays the
  simplest answer).
- **String concatenation via `+`.** Some Lisp dialects let
  `+` overload over strings; wat doesn't.
  `:wat::core::string::*` has its own ops.
- **Lab sweep.** Existing typed-arithmetic callsites stay.
  New code may use polymorphic forms; existing callsites
  migrate per-arc judgment if they're being touched anyway.

---

## Follow-through

- **Lab adoption is opt-in.** No mass sweep. Future lab arcs
  can simplify arithmetic-heavy vocab modules at the author's
  discretion.
- **058-030 INSCRIPTION addendum** documents the polymorphism
  surface at the language-spec level.
- **Polymorphic modulo / unary numerics** open their own arcs
  if pressure surfaces.

---

## Commits

- `<wat-rs>` — runtime.rs (`PolyOp` + `eval_poly_arith` + 14
  new dispatch arms + `values_equal` cross-numeric arms) +
  check.rs (`infer_polymorphic_compare` + `infer_polymorphic_arith`
  + `is_numeric` / `is_i64` predicates + 9 new
  dispatch branches in `infer_list` + 10 new typed strict
  scheme registrations + removed parametric scheme for the 5
  comparison ops) + tests/wat_polymorphic_arithmetic.rs (20
  tests) + DESIGN + BACKLOG + INSCRIPTION.

- `<lab>` — 058-030-types/PROPOSAL.md (INSCRIPTION addendum
  documents the polymorphism surface — separate slice; the
  language-spec note ships alongside this commit) +
  FOUNDATION-CHANGELOG.md (row).

---

*these are very good thoughts.*

**PERSEVERARE.**
