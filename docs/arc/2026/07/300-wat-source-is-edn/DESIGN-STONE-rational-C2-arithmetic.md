# DESIGN — Stone C2: rational arithmetic (the piece that makes `1/2` compute)

**Thesis.** Stone B made `rational` representable; C1 made `bigint` a full arithmetic type (the collapse
target). C2 adds **rational arithmetic** — `+ - * /`, comparison, `to-f64`, numerator/denominator — so
`1/2` finally computes, clj-faithfully. It is a direct one-type-over mirror of C1's now-committed `bigint`
work (`8edfbc14`): same op-form (home-type `::` intrinsics wired through the `:wat::core::+` defclause),
same fan-out shape. The one thing unique to C2 is the **collapse** (ratio arithmetic that reduces to a
whole number becomes a `bigint`, C1's type) and the **ratio contagion** matrix.

## Grounded contract (clj 1.12.4, oracle-verified this session)

```clojure
(+ 1/2 1/2) => 1N    bigint     ; RATIO arithmetic collapse → bigint (arbitrary precision, never overflows)
(+ 1/2 1/4) => 3/4   rational   ; stays rational (den >= 2)
(- 5/2 3/2) => 1N    bigint     ; collapse
(* 1/2 2)   => 1N    bigint     ; ratio ⊕ i64, collapses
(+ 1/2 1)   => 3/2   rational   ; ratio ⊕ i64  → rational
(+ 1/2 1N)  => 3/2   rational   ; ratio ⊕ bigint → rational
(+ 1/2 1.0) => 1.5   f64        ; ratio ⊕ f64  → f64 (FLOAT CONTAGION — convert to f64, no collapse)
(/ 1/2 1/2) => 1N    bigint     ; ratio / ratio, collapses
(= 1/2 1/2) => true             ; rational = (same-type, value)
(= 1/2 0.5) => false            ; rational vs f64 — category-aware = → false
(< 1/2 2/3) => true             ; total order
(double 1/2) => 0.5
```

## The pinned contract

Rational arithmetic reuses C1's exact shape, plus two contract points:
- **collapse**: `rational op rational` → `BigRational` result → `is_integer()` ? **`bigint`** (`Value::
  wat__core__BigInt`, C1's type) : `rational`. (Mirrors C1's `bigint /` → `bigint`|`rational` collapse,
  inverted: rational arithmetic collapses UP to bigint.)
- **contagion** (the mixed-operand matrix, in the `core.wat` defclause arms):
  `rational ⊕ i64 → rational` · `rational ⊕ bigint → rational` · `rational ⊕ f64 → f64` (both operand
  orders). i64/bigint promote to rational (then collapse-aware `+`); f64 pulls the rational down to f64.
- **compare** `< > <= >=`: total order, rational cross-type with `i64`/`bigint`/`f64`.
- **equality** `=`: `rational ↔ rational` by value; a rational is never integer-valued (den ≥ 2) so
  `rational ↔ {i64,bigint}` is always `false`; `rational ↔ f64` → `false` (category-aware).
- **`to-f64`**: `rational::to-f64` (`BigRational::to_f64` via num-traits). **`numerator`/`denominator`**
  accessors (clj has them; slash-form like `Uuid/version`).

## Rooms (mirror C1's committed sites, `rational` instead of `bigint`)

```clojure
{:arith-intrinsic "runtime.rs — add :wat::core::rational::+ - * / (eval dispatch ~4278 + inner ~8785);
                   impl fns eval_rational_arith / arith_rational_rational_inner MODELED ON C1's
                   eval_bigint_arith / arith_bigint_bigint_inner, but the result closure COLLAPSES
                   (BigRational → is_integer ? Value::wat__core__BigInt : Value::wat__core__Rational)"
 :conversions     "i64→rational, bigint→rational (for the contagion promote), rational→f64 (for f64 contagion)
                   — mirror C1's i64::to-bigint; register in check.rs TypeSchemes + rete/purity allow-list"
 :defclause       "wat/core.wat:58-160 (+ - * /) — add rational 1/2/N-ary arms + the 6 contagion arms
                   (rational⊕{i64,bigint}→rational, rational⊕f64→f64, both orders). Mirror C1's bigint arms."
 :compare         "values_compare runtime.rs:~8360 — rational + cross-type arms (cf C1's bigint arms)"
 :equal           "values_equal runtime.rs:~8156 — rational-rational; rational↔{i64,bigint,f64} → Some(false)"
 :to-f64          "runtime.rs dispatch ~4390 + eval fn (BigRational::to_f64)"
 :accessors       "runtime.rs slash-form dispatch (cf Uuid/version ~4331) — rational/numerator, /denominator → i64|bigint"
 :scalar-reg      "already done in C1 (rational registered as builtin scalar)"}
```

## Out of scope → later / cut

- **C3** i64 `wrap → error` (the don't-wrap-error ruling; every i64 arith site).
- `==` (the builder's clj cut). The `'` auto-promote family (not a wat concept — precision is type-carried).

## STOP triggers

- STOP if `(+ 1/2 1/2)` ≠ `1N` (bigint) — ratio collapse must produce C1's bigint, not stay `1/1`.
- STOP if `(+ 1/2 1.0)` ≠ `1.5` (f64) — float contagion; or if `(+ 1/2 1)` ≠ `3/2` (rational).
- STOP if `(= 1/2 0.5)` ≠ `false` or `(= 1/2 1/2)` ≠ `true` (category-aware `=`).
- STOP if rational arithmetic can wrap/overflow — BigRational is arbitrary precision.
- STOP if closing C2 needs i64-overflow changes (C3) or a `bigint` change (C1 is done).

## RED spec

`tests/value/probe_rational_C2_arithmetic.rs` — grounded rows above. RED at HEAD: `(:wat::core::+ 1/2 1/2)`
→ `NoMatchingClause` (no rational arm); `(< 1/2 2/3)` → `TypeMismatch`.
