# EXPECTATIONS — the cosine family's outcome wall

Written BEFORE the strike so the result cannot move the goalposts. Scored by the orchestrator's own
`--release` re-run, never the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the rider's own count | its report | 27 cosine · 3 dot · 16 coincident? · 17 presence? · 1 explain; **STOP-1 if materially off** |
| 2 | build clean | `cargo build --release --all-targets` | exit 0, **zero** warnings |
| 3 | clippy clean | `cargo clippy --release --all-targets` | exit 0, zero warnings |
| 4 | holon-rs, default | `cargo test` in `../holon-rs` | green |
| 5 | holon-rs, simd | `cargo test --features simd` in `../holon-rs` | green — **the const is consumed by a `cfg`-gated arm; the default run does not compile it** |
| 6 | `cosine` is total | `purity.rs` total block | `:wat::holon::cosine` present; its "left FALSE" comment retired |
| 7 | `dot` is total | same | present |
| 8 | `coincident?` is total | same | present — **and its return type is still `bool`** |
| 9 | `presence?` unmoved | same | present, unchanged, **no diff in `eval_algebra_presence_q`** |
| 10 | the mask is dead | `grep -n "return 0.0" ../holon-rs/src/kernel/similarity.rs` | the guard still exists as a backstop, but **wat never reaches it** — proven by row 11 |
| 11 | the degenerate case is FACED | a probe adapting `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat` | `(vector-blend v v 1.0 -1.0)` vs a real vector → `Degenerate[:Target]`, **not** `0.0` |
| 12 | the mismatch is FACED | a probe on two different-dimension vectors | `DimensionMismatch[expected got]`, no raise, no panic |
| 13 | the non-degenerate control still answers | same probe | genuine unrelatedness still reads ≈ `-0.0086`, self-similarity `1.0` |
| 14 | corpus loads | `cargo test --release --test lint` | `every_wat_scripts_file_loads` green |
| 15 | the whole floor | orchestrator's `cargo nextest run --release` | **4322 passed / 0 failed** (the floor at `cefee3ad`) |

## Rows 11–13 are the load-bearing ones

Everything else is hygiene. Row 11 is the reason this stone exists: R63 proved by run that a
zero-magnitude vector is two lines of ordinary wat away (`vector-blend v v 1.0 -1.0` cancels every
cell), and that the sentinel reads **exactly** `0.0` while genuine unrelatedness reads `-0.0086`. Both
pass `(f64::> … 0.9)` as *no match*, so today a rule author cannot tell "these are unrelated" from "you
compared against a degenerate vector."

Row 13 is the **non-vacuity control**. Without it, row 11 could pass because the probe broke rather
than because the wall works.

## Independent prediction

**Runtime 45–75 min.** The two enum registrations and the guard rework are small and have six shipped
exemplars; the 30 call-site matches are the bulk and are mechanical once the first two exist.

## Trap-doors, named in advance

- **The `cfg` split.** The holon-rs const is consumed under `#[cfg(feature = "simd")]` in one arm and
  not the other. A green default `cargo test` proves nothing about the code that changed. Rows 4 AND 5.
- **A duplicated `1e-10`.** If the rider copies the literal into wat-rs instead of consuming the const,
  the two can drift and the mask returns silently. Read the diff for a bare `1e-10`.
- **`presence?` collateral.** It shares a file and a family with the four but not the code path. Any
  diff inside `eval_algebra_presence_q` is a scope escape (STOP-3).
- **The `_` arm.** 30 new match sites is 30 chances to take the exhaustiveness error's own suggestion.
  Doctrine-illegal; grep the diff for it.
- **A `format!("#{}/{}")` in a test** trips `no_inlined_edn` — the false-positive class that lint's
  design predicts.

## What would make this a Mode B

The rider converting `coincident?`/`presence?` to outcome enums "for consistency." They are
**predicates**; absorbing an undefined comparison into `false` is their stated job, and the full
measurement remains available beside them in `cosine`. That distinction is ruled and the brief says so
twice.
