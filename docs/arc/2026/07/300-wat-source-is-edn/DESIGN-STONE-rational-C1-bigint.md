# DESIGN — Stone C1: BigInt as a first-class arithmetic integer type

**Thesis.** clj numeric parity (the builder's ratified path) requires rational arithmetic to collapse to
**BigInt** (`(+ 1/2 1/2)` → `1N`). The runtime has no BigInt. C1 adds it — and not as an inert
collapse-target (that ships a value that *looks* like a number and can't compute — the less-correct option,
struck), but as a **full first-class arithmetic integer type**, which is what clj's BigInt *is*. C1 is the
foundation C2 (rational arithmetic) collapses onto.

**Not a detour from "edn parity" — its reader half IS edn parity.** `clojure.edn` reads the `1N` literal as
a BigInt value, exactly as it reads `1/2` as a Ratio. So "read `1N`" is edn-reader parity (the arc's core
concern), the same move Stones A/B made for `1/2`. The *arithmetic* is `clojure.core` — the deeper half,
taken because the builder ratified full clj parity.

## Grounded contract (clj 1.12.4, run this session — every row is oracle-verified)

```clojure
;; the target behaviour C1 must reproduce (BigInt = clojure.lang.BigInt):
(pr-str 1N)          => "1N"     ; pr/edn form carries the N
(str 1N)             => "1"      ; str form drops it
(+ 1 1N)  (+ 1N 1)   => 2N       ; CONTAGION: i64 ⊕ BigInt → BigInt
(* 2 3N)             => 6N
(- 5N 2)             => 3N
(+ 1N 1N)            => 2N       ; BigInt arithmetic STAYS BigInt (never demotes to i64)
(+ (bigint i64::MAX) 1) => …N    ; arbitrary precision — NEVER overflows/throws (the never-overflow track)
(/ 6N 3N)            => 2N       ; BigInt / divisible → BigInt
(/ 1N 2N)            => 1/2      ; BigInt / non-divisible → Ratio (needs Stone B's Rational)
(< 1N 2) (< 1N 3/2) (< 1N 1.5) => true   ; total order across i64 / Ratio / f64
(= 1N 1)             => true     ; = is category-aware: BigInt ↔ i64 (same INTEGER category)
(= 1N 1.0)           => false    ;   BigInt ↔ f64 → false (different category)
(= 2N (/ 4 2))       => true     ;   (/ 4 2) is i64 2 → equal
(double 1N)          => 1.0
```

## THE PINNED CONTRACT

`Value::wat__core__BigInt(Box<num_bigint::BigInt>)` — a first-class integer type:
- **arithmetic** `+ - *` : arbitrary precision, **never** wraps/overflows (contrast i64, which errors — C3).
- **division** `/` : divisible → `BigInt`; else → `Rational` (reuses Stone B's `BigRational`).
- **contagion** in the shared `core.wat` `+ - * /` clauses: `i64 ⊕ BigInt → BigInt` (the i64 promotes).
- **compare** `< > <= >=` : total order, cross-type with `i64` / `f64` / `Rational`.
- **equality** `=` : category-aware — `BigInt ↔ i64` true (integer category), `BigInt ↔ f64` false.
- **render** : `"<n>N"` (pr/edn form); `to-string` → `"<n>"`.
- **reader** : `wat-reader` + `wat-edn` lex the `1N` literal → BigInt value (edn parity).
- **to-f64** : `BigInt::to-f64`.

## Rooms (from the C map — exact file:line)

```clojure
{:value-variant   "src/value/value.rs:319 (beside wat__core__Rational); use num_bigint::BigInt"
 :value-fanout    "PartialEq :604 (+ i64↔BigInt category eq) · Hash :755 · type_name :1151 · :1243"
 :type-keyword    "runtime.rs:6726 (Rational arm — add BigInt → :wat::core::BigInt)"
 :arith-eval      "runtime.rs:4278-4293 (i64 arith dispatch) — add :wat::core::BigInt::+ etc."
 :arith-inner     "runtime.rs:8785-8790 (dispatch_substrate_impl) — mirror arm"
 :arith-impl      "model on eval_i64_arith :7314 + arith_i64_i64_inner :8811 (BigInt: no overflow branch)"
 :core-wat        "wat/core.wat:58-160 (+  - * / defclauses) — add :wat::core::BigInt-typed + mixed i64/BigInt arms"
 :compare         "values_compare runtime.rs:8360-8437 (add BigInt + cross-type arms, cf. i64↔f64 :8369)"
 :equal           "values_equal runtime.rs:8156-8331 (add BigInt-BigInt + i64↔BigInt category arm)"
 :render          "value/observe.rs render_value (beside Rational) → format!(\"{}N\", n)"
 :to-f64          "runtime.rs:4390 dispatch + eval fn model on eval_i64_to_f64 :7487 (BigInt::to_f64 via num-traits)"
 :reader-1N       "crates/wat-reader/src/lexer.rs lex_numeric_or_symbol (1N suffix) + wat-edn already has Value::BigInt"
 :reserved-syms   "runtime.rs:23279-23298 (if any new bare name)"
 :deps            "num-bigint already workspace + root dep (Cargo.toml) — no new dep"}
```

## Out of scope → later stones (NOT divergences — sequencing)

- **C2** rational arithmetic (`Rational::+ - * /`, collapse → BigInt, Ratio contagion).
- **C3** i64 `wrap → error` (the don't-wrap-error ruling; every i64 arith site).
- `==` and the `'` auto-promote family — the builder's two principled cuts.

## STOP triggers

- STOP if BigInt arithmetic wraps or overflows — it MUST be arbitrary precision (that is its whole point).
- STOP if `(= 1N 1)` is not `true` or `(= 1N 1.0)` is not `false` — category-aware `=` is the contract.
- STOP if `/` divisible does not yield BigInt or non-divisible does not yield Ratio.
- STOP if closing C1 requires touching i64 overflow behavior (that is C3) or rational arithmetic (C2).
