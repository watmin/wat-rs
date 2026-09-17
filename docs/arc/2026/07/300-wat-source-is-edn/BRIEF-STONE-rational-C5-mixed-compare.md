# BRIEF — Stone C5: mixed-numeric comparison passes the checker (consistency fix)

**The work (one paragraph).** C4 made mixed-numeric *arithmetic* type-check. But mixed *comparison/equality*
is inconsistent: EVAL accepts it (`(< 1 2.0)` → true, `(= 1 1.0)` → false — the `values_compare`/`values_equal`
arms C1–C4 added), while the CHECKER rejects it (arc 237.8a deleted the cross-numeric path in
`infer_equality`, `src/check.rs`). So a real program rejects `(< 1 2.0)` at check even though eval computes
it. Make the checker accept mixed-**numeric** `= not= < > <= >=` → `:wat::core::bool`, matching eval + clj —
the comparison analog of C4's arithmetic reversal. Turn `tests/value/probe_rational_C5_mixed_compare.rs`
green (RED at HEAD: the co-located fixture's mixed comparisons fail to type-check → `startup_beside` errors).

## Read in order

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C5-mixed-compare.md` — design + the grounded
   state + room table. **Read first.**
2. `tests/value/probe_rational_C5_mixed_compare.rs` + `.wat` — the RED spec (2 tests + the co-located fixture).
3. `src/check.rs` `infer_equality` (~line 12412), the `types_compatible` decision (~12456), and the 237.8a
   comments at `:12395`/`:12440` ("cross-numeric path DELETED") — this is the site you loosen.

## The work

- **`src/check.rs` `infer_equality`** (~12456): the `types_compatible` decision currently accepts unify /
  subtype / both-record-share-`:wat::core::Record`. **ADD a `both_numeric` arm**: if `a_resolved` and
  `b_resolved` are BOTH in `{:wat::core::i64, :wat::core::f64, :wat::core::bigint, :wat::core::rational}`
  (+ `:wat::core::u8` if it participates), the comparison is well-formed → `:bool`. This undoes 237.8a's
  cross-numeric deletion for the numeric case only. Do NOT touch the same-type / subtype / record paths.
- **Verify the eval matrix is complete** — `values_compare` (runtime.rs ~8360) + `values_equal` (~8156)
  must handle every mixed-numeric pair the checker now admits (`i64↔f64` ✓, `i64↔bigint` ✓, `bigint↔f64` ✓,
  `rational↔f64` ✓, `rational↔i64` ✓, `rational↔bigint`?). If the C5 probe's eval test surfaces a missing
  pair (check-accepts / eval-errors — a NEW inconsistency), ADD that arm. Check and eval MUST agree.
- **Flip the two 237.8a COMPARISON-reject tests** to expect Ok (the same move C4 made for arithmetic):
  - `tests/types/probe_arc237_8a_no_implicit_coercion.rs::comparison_i64_f64_mixed_rejected_at_check`
    (fixture `..._cmp_i64_f64.wat.bad`)
  - `tests/function/probe_arc237_8b_defclause_arithmetic.rs::regression_cross_type_lt_rejected`
    (fixture `probe_arc237_8b_regression_cross_lt.wat`)
  Flip `assert!(result.is_err())` → `assert!(result.is_ok())`, rename to `..._coerces`/`..._works`, update
  the messages + doc-comments + the fixtures' header comments (they now type-check — note arc 300 C5).

## Out of scope

- `=` on NON-numeric heterogeneous types (`(= 1 "a")` — clj → false; wat still rejects). Separate
  `=`-semantics question — do NOT fold it in.
- No eval-semantics change: `=` stays category-aware (`(= 1 1.0)` → false, C4's contract). C5 only makes the
  CHECK accept it as well-formed.
- No new type; arithmetic (C4), overflow (C3), bigint/rational (C1/C2) untouched.

## How to work

Green at HEAD (post-C4). Add the checker arm, then: `cargo test -p wat --test value
probe_rational_C5_mixed_compare` (2/2), then the flipped 237.8a comparison tests green, then the other
rational probes (`_C4_mixed_float`, `_C3_i64_overflow`, `_C2_arithmetic`, `_C1_bigint`,
`_B_runtime_representation` — still green), then `cargo test -p wat-edn` (green), then a broad `cargo nextest
run` — **read the Summary; capture once to a file, grep the file**. The suite must show **exactly ONE
standing red**: `no_inlined_wat_in_tests` (the meter, 351). If `wat-cli sigterm…` trips, verify solo with
`--test-threads=1` (arc-170 race). `deporder` is 30s. Any other red is a real regression — halt + report.
Do NOT commit.

## STOP triggers

- STOP if a mixed-numeric comparison that CHECKS does not EVAL to the right value — check + eval MUST agree.
- STOP if `(= 1 1.0)` eval changes from `false` (category-aware `=` is C4's, unchanged).
- STOP if the checker now accepts a NON-numeric mixed comparison (`(= 1 "a")`) — numerics only.
- STOP if same-type / subtype / record-compatibility comparison behavior changes (only ADD the numeric arm).

## Done = green

- `probe_rational_C5_mixed_compare` 2/2; the flipped 237.8a comparison tests green; other rational probes green.
- `cargo build -p wat` clean; `cargo test -p wat-edn` green.
- Broad `cargo nextest run`: exactly one standing red (the 351 meter); no new failures.

Report: files changed; the `both_numeric` arm; any eval-matrix pair you had to add; the flipped tests; the
Summary line; any STOP hits.

**Prior reference:** C4 (`bbfc347a`) — the arithmetic analog + the 237.8a arith-test flip pattern.
