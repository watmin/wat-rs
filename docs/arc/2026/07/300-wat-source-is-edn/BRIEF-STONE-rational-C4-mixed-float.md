# BRIEF — Stone C4: mixed-float contagion (the `(+ 1 2.0)` unlock; the last numeric stone)

**The work (one paragraph).** The numeric tower is one contagion pattern (C1/C2). Two mixed pairs remain,
both → `f64` (float wins): `i64 ⊕ f64` and `f64 ⊕ bigint`. Install their arms in the `+ - * /` defclauses
so `(+ 1 2.0) => 3.0` — mixed int/float arithmetic. Also fold in the `i64 ↔ f64` **equality** arm:
`(= 1 1.0)` currently ERRORS (TypeMismatch), but clj says `false` (category-aware — different category),
and C1/C2 already made `=` category-aware (`(= 1N 1.0)`→false, `(= 1/2 0.5)`→false); the plain `i64↔f64`
arm was just never added. Turn `tests/value/probe_rational_C4_mixed_float.rs` green. **A direct mirror of
C1/C2's contagion arms** — one type-pair over.

## Read in order

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C4-mixed-float.md` — design + oracle
   contract + room table. **Read first.**
2. `tests/value/probe_rational_C4_mixed_float.rs` — the RED spec (3 tests; the honest-N-ary one already passes).
3. **C1/C2's committed contagion arms** (`git show 8edfbc14` and `git show 305c7e3d`) — the `i64⊕bigint`
   (C1) and `rational⊕f64` (C2) arms in `wat/core.wat`, and the `(bigint,f64)`/`(rational,f64) => Some(false)`
   equality arms in `values_equal`. C4 mirrors these exactly, one pair over.

## The oracle contract (verified this session)

```
(+ 1 2.0)=>3.0 f64   (+ 2.0 1)=>3.0   (+ 2.0 1N)=>3.0   (+ 1N 2.0)=>3.0   (* 3 2.0)=>6.0   (- 5.0 2)=>3.0
(= 1 1.0)=>false (category-aware, NOT an error)     (< 1 2.0)=>true (already works)
(+ 1 2.0 3)=>NoMatchingClause (the honest mixed-N-ary gap — must STAY; do NOT add heterogeneous N-ary folds)
```

## The work (mirror C1/C2, f64-contagion instead)

- **`wat/core.wat` defclauses** (`+ - * /`): add the float-contagion arms — `i64 ⊕ f64 → f64`,
  `f64 ⊕ i64 → f64`, `bigint ⊕ f64 → f64`, `f64 ⊕ bigint → f64`. Mechanism: promote the non-`f64` operand
  to `f64` (`i64::to-f64` and `bigint::to-f64` **already exist**), then the existing `f64::op` → `f64`. No
  collapse. Mirror the shape of C1's `i64⊕bigint` and C2's `rational⊕f64` arms.
- **`values_equal`** (runtime.rs ~8156): add `(i64, f64)` and `(f64, i64)` → `Some(false)` (category-aware;
  a float and an integer are different categories). Mirror the `(bigint,f64)`/`(rational,f64) => Some(false)`
  arms C1/C2 added. `=` must return `false`, never *error*, on a numeric type-mismatch.
- **`values_compare`** (runtime.rs ~8360): `i64↔f64` already works; check `f64↔bigint` (`(< 1N 2.0)`) and
  add that arm **only if** it's missing.
- **Do NOT** add heterogeneous N-ary fold arms — the mixed N-ary gap (`(+ 1 2.0 3)` → toss) is intentional
  and permanently guarded by the probe.

## How to work

Green at HEAD (post-C3). Add the arms, follow the compile cascade (most sites are one-pair-over copies of
the adjacent C1/C2 arm). Then: `cargo test -p wat --test value probe_rational_C4_mixed_float` (3/3), then
the other rational probes (`_C3_i64_overflow`, `_C2_arithmetic`, `_C1_bigint`, `_B_runtime_representation`
— still green), then `cargo test -p wat-edn` (green), then a broad `cargo nextest run` — **read the
Summary; capture once to a file, grep the file**. The suite must show **exactly ONE standing red**:
`no_inlined_wat_in_tests` (the meter, 351). If `wat-cli sigterm…` trips, verify solo with
`--test-threads=1` (the arc-170 race). `deporder` is fixed (30s). Any other red is a real regression — halt
+ report. Do NOT commit.

## STOP triggers

- STOP if `(+ 1 2.0)` ≠ `3.0` (f64) — float must win.
- STOP if `(= 1 1.0)` ≠ `false` — category-aware `false`, NOT an error, NOT `true`.
- STOP if `(+ 1 2.0 3)` does NOT toss `NoMatchingClause` — the honest N-ary gap must stand.
- STOP if C4 needs a collapse, a new type, or any i64-overflow/bigint/rational-arithmetic change
  (C1/C2/C3 are done, committed).

## Done = green

- `probe_rational_C4_mixed_float` 3/3; the other rational probes still green.
- `cargo build -p wat` clean; `cargo test -p wat-edn` green.
- Broad `cargo nextest run`: exactly one standing red (the 351 meter); no new failures.

Report: files changed; the fan-out sites vs the room table; whether `f64↔bigint` compare needed adding; any
STOP hits; the Summary line.

**Prior reference:** C1 (`8edfbc14`) + C2 (`305c7e3d`) diffs — same contagion-arm shape, `f64` instead.
