# BRIEF — Stone C2: rational arithmetic

**The work (one paragraph).** Stone B made `rational` representable; C1 (`8edfbc14`) made `bigint` a full
arithmetic type. Add **rational arithmetic** — `+ - * /`, comparison (`< > <= >=`), category-aware `=`,
`to-f64`, and `numerator`/`denominator` — so `1/2` computes, clj-faithfully. This is a **direct one-type-
over mirror of C1's committed `bigint` work**: same home-type `::`-intrinsic + `:wat::core::+` defclause
shape, same fan-out. Two things are unique to C2: (1) the **collapse** — ratio arithmetic reducing to a
whole number becomes a `bigint` (C1's type), the inverse of C1's `bigint /` → rational collapse; and (2)
the **ratio contagion** matrix. Turn `tests/value/probe_rational_C2_arithmetic.rs` green.

## Read in order

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C2-arithmetic.md` — design, oracle
   contract, room table. **Read first.**
2. `tests/value/probe_rational_C2_arithmetic.rs` — the RED spec (4 tests).
3. **C1's committed diff** (`git show 8edfbc14`) — the `bigint` arithmetic + defclause + compare/equal +
   contagion + to-f64 you MIRROR one type over. `eval_bigint_arith`/`arith_bigint_bigint_inner`,
   `bigint_div`, the `core.wat` bigint arms, the `values_compare`/`values_equal` bigint arms, and the
   `i64::to-bigint`/`i64⊕bigint` contagion are your exact templates.

## The oracle contract (verified this session)

```
(+ 1/2 1/2)=>1N bigint   (+ 1/2 1/4)=>3/4 rational   (* 2/3 3/2)=>1N   (- 5/2 3/2)=>1N
(+ 1/2 1)=>3/2 rational  (+ 1/2 1N)=>3/2   (+ 1/2 1.0)=>1.5 f64 (float contagion)
(< 1/2 2/3)=>true   (= 1/2 1/2)=>true   (= 1/2 0.5)=>false (category-aware)   (double 1/2)=>0.5
```

## The work (mirror C1, rational instead of bigint)

- **Intrinsics** `:wat::core::rational::+ - * /` — eval dispatch (~runtime.rs:4278) + inner (~8785); impl
  fns `eval_rational_arith`/`arith_rational_rational_inner` modeled on C1's bigint fns, but the result
  **COLLAPSES**: build the `BigRational` result, then `is_integer()` ? `Value::wat__core__BigInt(numer)` :
  `Value::wat__core__Rational`. Arbitrary precision — no overflow.
- **Conversions** for contagion (mirror C1's `i64::to-bigint`): `i64→rational`, `bigint→rational` (promote
  to rational, then the collapse-aware op), and reuse `rational::to-f64` for the f64 arm. Register any new
  intrinsic in `check.rs` TypeSchemes + `rete/purity.rs` allow-list (as C1 did for its bigint intrinsics).
- **`core.wat` defclauses** (`+ - * /`, ~58-160): add rational 1/2/N-ary arms + the contagion arms —
  `rational⊕i64→rational`, `rational⊕bigint→rational`, `rational⊕f64→f64` (both operand orders). Mirror
  the C1 bigint/contagion arms exactly.
- **`values_compare`** (~8360): rational + cross-type (`rational↔i64/bigint/f64`) — total order.
- **`values_equal`** (~8156): `rational↔rational` by value; `rational↔{i64,bigint}` → `Some(false)` (a
  rational has den ≥ 2, never integer-valued); `rational↔f64` → `Some(false)` (category-aware).
- **`to-f64`**: `rational::to-f64` dispatch + fn (`BigRational::to_f64`). **`numerator`/`denominator`**:
  slash-form accessors (cf `Uuid/version`) → `i64` (or `bigint` if the component exceeds i64).

## How to work

Green at HEAD (post-C1). Follow the compile cascade toward zero — most sites are one-type-over copies of
the C1 bigint arm right beside where you're editing. Then: `cargo test -p wat --test value
probe_rational_C2_arithmetic` (4/4) → `probe_rational_C1_bigint` + `probe_rational_B` (still green) →
`cargo test -p wat-edn` (green) → broad `cargo nextest run`. **Read the Summary; capture once to a file,
grep the file.** The suite must show **exactly ONE standing red**: `no_inlined_wat_in_tests` (the meter,
351). A process-management test (`wat-cli sigterm…`) may intermittently trip under parallel load — if so,
verify it passes with `--test-threads=1` (the known arc-170 race, not yours). `deporder` is fixed (30s).
**Anything else red is a real regression — halt and report.** Do NOT commit.

## STOP triggers

- STOP if `(+ 1/2 1/2)` ≠ `1N` bigint — ratio collapse must produce C1's bigint, not stay `1/1` rational.
- STOP if `(+ 1/2 1.0)` ≠ f64, or `(+ 1/2 1)` ≠ `3/2` rational (contagion).
- STOP if `(= 1/2 0.5)` ≠ `false` or `(= 1/2 1/2)` ≠ `true` (category-aware `=`).
- STOP if rational arithmetic can wrap/overflow — BigRational is arbitrary precision.
- STOP if closing C2 needs an i64-overflow change (C3) or a `bigint` change (C1 is done, committed).

## Done = green

- `probe_rational_C2_arithmetic` 4/4; `probe_rational_C1_bigint`, `probe_rational_B` still green.
- `cargo build -p wat` clean; `cargo test -p wat-edn` green.
- Broad `cargo nextest run`: exactly one standing red (the 351 meter); no new failures.

Report: files changed; fan-out sites vs the room table; how you mirrored C1; how the collapse/contagion
were implemented; any STOP hits; the final Summary line.

**Prior reference:** C1's diff (`git show 8edfbc14`) + `BRIEF-STONE-rational-C1-bigint.md` — same shape.
