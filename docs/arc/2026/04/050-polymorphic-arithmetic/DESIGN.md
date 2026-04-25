# wat-rs arc 050 — polymorphic arithmetic with int→float promotion

**Status:** opened 2026-04-24. Fifth wat-rs arc post-known-good.
Lab arc 023 (exit/trade_atoms) surfaced the gap with a usability
observation: typed-arithmetic prefixes (`:wat::core::f64::+`,
`:wat::core::f64::-`, `:wat::core::f64::*`, `:wat::core::f64::/`)
appear ~25 times in a single vocab function, almost always over
already-`:f64`-typed values. Builder direction:

> "we have 'polymorphic over numerics' for equality check?... can
> we do the same for + - * / %? force ints to floats for
> comparisons?... math statements have been annoying reaching for
> a specific type every time"

> "we keep the explicit typed expressions but offer a polymorphic
> one - the users chooses"

The Lisp tradition: `+` accepts any numeric, promotes to the
wider type, returns a result of that type. Coexists with strict
typed forms for callers who want the "this MUST be i64" guard.

---

## What ships

Four new runtime primitives, polymorphic over `:i64` and `:f64`,
with promotion to `:f64` when types mix:

```scheme
(:wat::core::+ a b)   ;; addition
(:wat::core::- a b)   ;; subtraction
(:wat::core::* a b)   ;; multiplication
(:wat::core::/ a b)   ;; division (errors on /0 same as typed forms)
```

**Promotion rule** (matches existing comparison ops at
`runtime.rs:3501-3506`):

| LHS | RHS | Result |
|---|---|---|
| `:i64` | `:i64` | `:i64` |
| `:f64` | `:f64` | `:f64` |
| `:i64` | `:f64` | `:f64` (LHS cast first) |
| `:f64` | `:i64` | `:f64` (RHS cast first) |

`(:wat::core::+ 1 2.5)` → `3.5 :f64`. The machine answers
honestly.

**Coexists with typed forms.** `:wat::core::i64::+`,
`:wat::core::f64::+`, etc. stay. Strict-typed callers reach for
those when they want the type-guard behavior (refuses cross-type
input). User chooses per-callsite.

**Division-by-zero behavior** matches typed forms: i64/0,
f64/0.0, mixed-promoted/0 all raise `DivisionByZero`. The
existing typed forms catch f64/0.0 (rather than producing
inf/nan); the polymorphic form preserves that.

---

## Why coexistence over migration

058-030's commit was *"Drop abstract `:Scalar`/`:Int`/`:Bool`/`:Null`
in favor of Rust primitives... Honest mapping to Rust; no
abstraction layer."* That was about TYPES (drop abstract scalar);
this arc is about OPERATIONS (add polymorphic ones alongside
typed ones). Two different concerns.

Rust does NOT have polymorphic arithmetic — `i64 + f64` doesn't
compile. We deviate here, deliberately:

- **Comparisons already deviate.** `:wat::core::>`, `<`, `=`,
  `>=`, `<=` accept i64/f64 mixed and auto-coerce
  (`runtime.rs:3501-3506`). The asymmetry between comparisons
  (polymorphic) and arithmetic (typed) is jarring; this arc
  closes it.
- **Lisp tradition.** Common Lisp's `+`, `-`, `*`, `/` are
  polymorphic with integer-to-float promotion. Wat embraces the
  Lisp lineage (s-expressions, namespace discipline, value
  semantics). Polymorphic numerics are part of that tradition.
- **Userland ergonomics.** The typed prefix appears ~25 times
  in trade-atoms.wat, ~similar in standard.wat — hundreds of
  call sites across the lab vocab tree, almost all on already-
  homogeneous f64 values. The verbosity tax is paid every op;
  the explicit-type-guard benefit is paid only at the genuine
  mixing boundaries (where users still write `to-f64` if they
  want strict-i64-error behavior).

The strict typed forms STAY because the type-guard behavior
genuinely matters in some sites:

- Index arithmetic (`(:i64::- (length xs) 1)` to compute "last
  index") — wants i64 result; promoting to f64 silently would
  bug downstream code.
- Counting / arity ops where overflowing i64 to f64 would drop
  precision.

User picks per-callsite which discipline they want.

---

## Implementation

`src/runtime.rs`:

- Four new dispatch arms after the existing typed-arithmetic
  arms (around line 2106):

```rust
":wat::core::+" => eval_poly_arith(head, args, env, sym, PolyOp::Add),
":wat::core::-" => eval_poly_arith(head, args, env, sym, PolyOp::Sub),
":wat::core::*" => eval_poly_arith(head, args, env, sym, PolyOp::Mul),
":wat::core::/" => eval_poly_arith(head, args, env, sym, PolyOp::Div),
```

- New helper `eval_poly_arith(head, args, env, sym, op)` —
  dispatches on `(Value, Value)` pair. Same shape as
  `eval_compare` (`runtime.rs:3481`), but returns a numeric
  Value rather than bool. The `(i64, f64)` and `(f64, i64)`
  arms cast i64 to f64 before applying.

- Division-by-zero check: i64/0 → `DivisionByZero`; f64/0.0 →
  `DivisionByZero` (preserves existing behavior); mixed
  i64-promoted/0.0 → same.

`src/check.rs`:

- Four new schemes registered in `register_builtins` (or
  equivalent — wherever `i64::+` / `f64::+` schemes live).
  Each has a polymorphic signature: parameter type variables
  T, U bound to numeric types, return type is the wider of
  the two. The simplest expression: parametric over a type
  variable T that unifies to either `:i64` (if both args i64)
  or `:f64` (if either arg f64). If the type system can't
  express "wider of two numeric types" cleanly, fall back to
  per-input-type check at the runtime side and have the
  scheme accept (`:i64`|`:f64`, `:i64`|`:f64`) → (`:i64`|`:f64`).

(I'll see what fits the existing scheme machinery when I
write the slice; the runtime side is the load-bearing piece.)

---

## What this arc does NOT add

- **Polymorphic modulo `%`.** No callsites in lab or archive
  today. Ship when a caller surfaces.
- **Polymorphic max/min/abs/clamp/round.** These are unary or
  multi-arg; they don't have the same "mix two types" pressure.
  The lab uses typed forms (`:wat::core::f64::max` etc.) and
  the verbosity is far less noticeable since they appear less
  often. Add later if the pressure is real.
- **Polymorphic comparison operators.** Already polymorphic;
  no work.
- **Wider integer types** (`:i32`, `:u64`, `:i128`, etc.).
  Wat's numeric primitives today are `:i64` + `:f64` + `:u8`;
  polymorphic arithmetic over `:u8` isn't compelling (u8 is a
  byte type, not a number type users do math on). If wider
  ints land later, polymorphic forms extend to cover them.
- **String concatenation via `+`.** Some Lisp dialects let
  `+` overload over strings; we don't. `:wat::core::string::*`
  has its own ops.

---

## Sub-fogs

- **Type-checker scheme expression.** Need to verify how
  parametric numerics are expressed in the existing scheme
  registry. If the `from_symbols` / `register_builtins` path
  has a precedent for "polymorphic over a numeric type
  variable," reuse it. Otherwise fall back to a wider hand-
  written scheme that accepts the union.
- **Atom / Vec / Tuple etc. as arithmetic operands?** No —
  arithmetic is over numerics only. The runtime arm dispatches
  pair-wise, falling through to a TypeMismatch error for
  non-numeric inputs. Same shape as `eval_compare`'s default arm.

---

## Non-goals

- **Lab sweep to migrate `i64::+` / `f64::+` callsites to `+`.**
  Existing callsites keep working. New code can use either.
  Mass migration costs more than it saves; the polymorphic form
  is opt-in.
- **Deprecation of typed forms.** Both shapes coexist. Strict-
  typed callers retain the type-guard benefit when they want it.
