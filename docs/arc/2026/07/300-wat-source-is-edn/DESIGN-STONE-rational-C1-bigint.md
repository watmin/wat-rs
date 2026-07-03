# DESIGN — Stone C1: numeric-scalar naming cleanup + `bigint` as a first-class arithmetic type

**Thesis.** clj numeric parity (ratified) requires rational arithmetic to collapse to **BigInt**
(`(+ 1/2 1/2)` => `1N`); the runtime has no BigInt. C1 (a) cleans up the numeric-scalar *naming* per
Doctrine 2, and (b) adds `bigint` as a **full first-class arithmetic integer type** (contagious,
never-demotes, arbitrary-precision — grounded vs the clj oracle), the target C2's rational arithmetic
collapses onto. Reader-half (read the `1N` literal) is edn-reader parity, the same move A/B made for `1/2`.

## Part A — naming cleanup (intueri verdict, Doctrine 2 / Stone 242.1)

**Rule:** scalar VALUE → lowercase (`i64 f64 u8 bool char`); identity-or-structure → Capital (`Uuid`,
`String`, containers). A number is a value → lowercase. So:

```clojure
{:rename {":wat::core::Rational" ":wat::core::rational"}   ; Stone B shipped a Level-2 mumble; fix it
 :new    ":wat::core::bigint"                              ; the incoming type, lowercase from birth
 :rust-variants :stay-Capital}   ; Value::wat__core__Rational / ::wat__core__BigInt — mirrors wat__core__Char
```

- **Rename `Rational → rational`** — the SURFACE string at 5 sites (Rust *variant* `wat__core__Rational`
  stays Capital, exactly the `Char → char` precedent):
  `value.rs:1151` (type_name) · `runtime.rs:6726` · `edn_shim.rs:1495` · `check.rs:3270` · `check.rs:7498`
  (+ docs `value.rs:311`, `check.rs:3269`). Update the Stone B probe's assertion `"wat::core::Rational"`
  → `"wat::core::rational"`.
- **Finish the `Char` rename** (Doctrine 2 half-propagated): `type_name()`/reflection still emit capital
  `"wat::core::Char"` at `value.rs:1149` + `runtime.rs:13438` → lowercase `"wat::core::char"`
  (`check.rs:13587` is already correct).
- **Register the scalars** — `is_builtin_primitive` (`runtime.rs:13427-13454`) + the pure-scalar list
  (`check.rs:13579-13596`): add `"wat::core::rational"` and `"wat::core::bigint"` beside `i64`/`f64`/`char`.

## Part B — `bigint`, a full arithmetic integer type

### Grounded contract (clj 1.12.4, oracle-verified this session)

```clojure
(pr-str 1N) => "1N"   (str 1N) => "1"                 ; pr/edn carries N; str drops it
(+ 1 1N) (+ 1N 1) => 2N   (* 2 3N) => 6N   (- 5N 2) => 3N   ; CONTAGION: i64 ⊕ bigint → bigint
(+ 1N 1N) => 2N   (- 3N 2N) => 1N                     ; bigint arithmetic STAYS bigint (never demotes)
(+ (bigint i64::MAX) 1) => …N                         ; arbitrary precision — NEVER overflows/throws
(/ 6N 3N) => 2N   (/ 1N 2N) => 1/2                    ; bigint / : divisible→bigint, else→rational
(< 1N 2) (< 1N 3/2) (< 1N 1.5) => true                ; total order across i64/rational/f64
(= 1N 1) => true   (= 1N 1.0) => false                ; = is category-aware: bigint↔i64 same INTEGER category
(double 1N) => 1.0
```

### The pinned contract

`Value::wat__core__BigInt(Box<num_bigint::BigInt>)` (surface `:wat::core::bigint`) — a first-class integer:
- **arithmetic `+ - *`**: arbitrary precision, NEVER wraps/overflows (contrast i64 → error, C3).
- **`/`**: divisible → `bigint`; else → `rational` (reuses Stone B's `BigRational`).
- **contagion**: in the shared `+ - * /` defclause, `i64 ⊕ bigint → bigint` (i64 promotes to bigint).
- **compare `< > <= >=`**: total order, cross-type with `i64`/`f64`/`rational`.
- **equality `=`**: category-aware — `bigint ↔ i64` true (integer category), `bigint ↔ f64` false.
- **render**: `"<n>N"` (pr/edn); `to-string` → `"<n>"`.
- **reader**: `wat-reader` lexes the `1N` literal → bigint value (edn parity; wat-edn already does).
- **`to-f64`**: `bigint::to-f64` (num-traits `ToPrimitive`).

### The op-form — mirror the LIVE i64/f64 shape (no `'2`; 237.8b killed it)

Ops live on the home type as the 2-ary `::` intrinsic, wired through the `:wat::core::+` **defclause**
(multi-arity + pre/post are defclause's job), which folds the 2-ary over N args:

```clojure
(:wat::core::defclause :wat::core::+           ; extend the existing clause with bigint arms
  …
  ([x <- :wat::core::bigint] -> :wat::core::bigint x)                         ; 1-ary
  ([x <- :wat::core::bigint  y <- :wat::core::bigint] -> :wat::core::bigint
    (:wat::core::bigint::+ x y))                                             ; 2-ary → Rust intrinsic (`::` form, like i64::+)
  ([x <- :wat::core::bigint  y <- :wat::core::bigint  & rest <- …] -> …
    (:wat::core::foldl … (:wat::core::bigint::+ x y) rest))                  ; N-ary folds the 2-ary
  ;; + the mixed i64⊕bigint contagion arms → :wat::core::bigint
  )
```

**No `+'` family** — precision is carried by the TYPE (`i64/+`→error, `bigint/+`→arbitrary), not a
separate operator. **No `'2`** — the per-Type binary is bare `:wat::core::bigint::+`.

## Rooms (from the C map — exact file:line)

```clojure
{:value-variant "value.rs:319 (beside wat__core__Rational); Value::wat__core__BigInt(Box<num_bigint::BigInt>)"
 :value-fanout  "PartialEq :604 (+ i64↔bigint category eq) · Hash :755 · type_name :1151 (\"wat::core::bigint\") · :1243"
 :type-keyword  "runtime.rs:6726 (bigint → :wat::core::bigint)"
 :arith-intrinsic "runtime.rs:4278-4293 (eval dispatch) + :8785-8790 (dispatch_substrate_impl); impl fns model eval_i64_arith :7314 / arith_i64_i64_inner :8811 — bigint has NO overflow branch"
 :defclause     "wat/core.wat:58-160 (+ - * /) — add :wat::core::bigint arms + i64⊕bigint contagion arms"
 :compare       "values_compare runtime.rs:8360-8437 (bigint + cross-type arms, cf i64↔f64 :8369)"
 :equal         "values_equal runtime.rs:8156-8331 (bigint-bigint + i64↔bigint category arm)"
 :render        "value/observe.rs render_value (beside Rational) → format!(\"{}N\", n)"
 :to-f64        "runtime.rs:4390 dispatch + eval fn model eval_i64_to_f64 :7487 (BigInt::to_f64 via num-traits)"
 :reader-1N     "crates/wat-reader/src/lexer.rs lex_numeric_or_symbol (1N suffix → Token::BigInt); wat-edn already lexes 1N"
 :scalar-reg    "is_builtin_primitive runtime.rs:13427 + pure-scalar list check.rs:13579"
 :deps          "num-bigint already workspace + root dep — no new dep"}
```

## Out of scope → later stones (sequencing, NOT divergence)

- **C2** rational arithmetic (`rational::+ - * /`, collapse → bigint, ratio contagion arms).
- **C3** i64 `wrap → error` (the don't-wrap-error ruling; every i64 arith site).
- `==` (the builder's one clj cut).

## STOP triggers

- STOP if `bigint` arithmetic wraps/overflows — it MUST be arbitrary precision (its whole point).
- STOP if `(= 1N 1)` ≠ `true` or `(= 1N 1.0)` ≠ `false` — category-aware `=` is the contract.
- STOP if `/` divisible ≠ `bigint` or non-divisible ≠ `rational`.
- STOP if the rename touches the Rust *variant* identifiers `wat__core__Rational`/`wat__core__BigInt`
  (those stay Capital) — only the SURFACE `:wat::core::…` strings lowercase.
- STOP if closing C1 requires i64 overflow behavior (C3) or rational *arithmetic* (C2).
