# DESIGN — Stone C4: mixed-float contagion — the `(+ 1 2.0)` unlock (the last numeric stone)

**Thesis.** The numeric tower is one contagion pattern (C1/C2). Two mixed pairs remain unfilled — both
involving `f64`, both → `f64` (float wins): `i64 ⊕ f64` and `f64 ⊕ bigint`. Installing their arms lights up
`(+ 1 2.0) => 3.0` — the mixed int/float arithmetic the builder walked away from years ago, now two arms in
the proven pattern. C4 also folds in one oversight the grounding surfaced: the `i64 ↔ f64` **equality** arm
(`(= 1 1.0)` currently *errors*; clj says `false` — category-aware, and C1/C2 already made `=` so).

## Grounded contract (clj 1.12.4 + wat HEAD, this session)

```clojure
;; ARITHMETIC — float wins (the gap: all NoMatchingClause at HEAD):
(+ 1 2.0) (+ 2.0 1)  => 3.0  f64      ; i64 ⊕ f64 → f64
(+ 2.0 1N) (+ 1N 2.0) => 3.0 f64      ; f64 ⊕ bigint → f64
(* 3 2.0) => 6.0 f64   (- 5.0 2) => 3.0 f64   (/ 1 2.0) => 0.5 f64
;; EQUALITY — category-aware (the gap: (= 1 1.0) ERRORS at HEAD; clj → false):
(= 1 1.0) => false                    ; i64 ↔ f64 — different category → false (NOT an error, NOT true)
(= 1 1N)  => true (C1, present)   (= 1N 1.0) => false (C1)   (= 1/2 0.5) => false (C2)
;; COMPARE — already works (values_compare has i64↔f64):
(< 1 2.0) => true                     ; verify f64↔bigint too ((< 1N 2.0)); add the arm only if missing
;; MIXED N-ARY — the honest gap (the builder's ruling, already living):
(+ 1 2.0 3) => NoMatchingClause        ; heterogeneous N-ary tosses; caller homogenizes (map to-f64) then folds
```

## The pinned contract

- **Float-contagion arithmetic** `+ - * /`: `i64 ⊕ f64 → f64` and `f64 ⊕ bigint → f64` (both operand
  orders). Mechanism: promote the non-`f64` operand to `f64` (`i64::to-f64` / `bigint::to-f64` — **both
  already exist**), then the existing `f64::op` → `f64`. No collapse (float contagion pulls down to f64).
- **Equality** `=`: add the `i64 ↔ f64` arm → `Some(false)` (category-aware; a float and an integer are
  different categories — completes what C1's `bigint↔f64→false` and C2's `rational↔f64→false` began). `=`
  must never *error* on a numeric type-mismatch — it returns `false`.
- **Compare** `< > <= >=`: `i64↔f64` already works; verify `f64↔bigint` (`(< 1N 2.0)`) and add that arm
  only if the grounding shows it missing.
- **Mixed N-ary**: unchanged — a heterogeneous N-ary call (`(+ 1 2.0 3)`) tosses a clean `NoMatchingClause`;
  the caller homogenizes (`(apply + (map to-f64 …))`) then folds. Do NOT add mixed N-ary fold arms.

## Rooms (mirror C1/C2's committed contagion sites)

```clojure
{:defclause "wat/core.wat (+ - * /) — add i64⊕f64 / f64⊕i64 / bigint⊕f64 / f64⊕bigint arms → f64, promoting
             the non-f64 operand via i64::to-f64 / bigint::to-f64 then f64::op. Mirror C1's i64⊕bigint +
             C2's rational⊕f64 contagion arms exactly."
 :equal     "values_equal (runtime.rs ~8156) — add (i64,f64)/(f64,i64) => Some(false); mirror the
             (bigint,f64)/(rational,f64) => Some(false) arms C1/C2 added."
 :compare   "values_compare (runtime.rs ~8360) — verify f64↔bigint; add the arm only if the RED probe shows it missing."
 :conversions "i64::to-f64 + bigint::to-f64 both already exist — no new intrinsics."}
```

## Out of scope (the tower is complete after C4)

- No new numeric type. No collapse (float contagion has none). No mixed N-ary fold (honest gap).
- `==` (the builder's cut). The `'` auto-promote family (not a wat concept — precision is type-carried).

## STOP triggers

- STOP if `(+ 1 2.0)` ≠ `3.0` (f64) — float must win.
- STOP if `(= 1 1.0)` ≠ `false` — it must be a category-aware `false`, NOT an error and NOT `true`.
- STOP if a mixed N-ary (`(+ 1 2.0 3)`) does NOT toss `NoMatchingClause` — the honest gap must stand
  (do not add heterogeneous N-ary fold arms).
- STOP if C4 needs a collapse, a new type, or any i64-overflow / bigint / rational-arithmetic change
  (C1/C2/C3 are done, committed).

## RED spec

`tests/value/probe_rational_C4_mixed_float.rs`: `(+ 1 2.0)`/`(+ 2.0 1N)`/`(* 3 2.0)` → f64; `(= 1 1.0)` →
false; `(+ 1 2.0 3)` → tosses (honest N-ary gap). RED at HEAD: the arithmetic → NoMatchingClause,
`(= 1 1.0)` → TypeMismatch.
