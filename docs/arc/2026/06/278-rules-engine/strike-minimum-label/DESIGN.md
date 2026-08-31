# DESIGN-STONE — the label follows the arithmetic

> **Origin.** Vigilia Class C1, found INDEPENDENTLY by `complectens` and `vocare`. It is the
> orchestrator's own defect: commit `89e8c3ed0` ("the instrument measured the wrong estimator —
> 106 accumulators, mean -> minimum") **moved the labels and left the arithmetic.**

## Why

`git blame` settles authorship. `src/rete/kernel/tests/mod.rs:493` — the header reading
`MINIMUM of {RUNS} runs` — is `89e8c3ed0`. Lines 528-542, where `stat()` returns
`sum as f64 / xs.len() as f64` and `net_of` / `total_mean` / the row loop all read `.0`, are the
**earlier split commit, untouched.**

So `render_phase_table` prints **means** under a header that says **minimum**, and it renders the
axis tables for fanout, accum, node-share and rank-and-instrument. The same label-without-estimator
flip survives in the per-test accumulators: `x += sample` inside `for _ in 0..RUNS`, then `x /= r`.

**The commit message asserted a general fix while the diff performed a specific one.** The isolated
micro-arms (`x.min(elapsed_ns(...))`) were genuinely converted. The in-fire census accumulators
never were.

## The population — measured, after a first count that was wrong

Files binding `let r = RUNS as f64` **and** printing a `MINIMUM of` header:

| file | divides by RUNS | MINIMUM headers |
|---|---:|---:|
| `accum_cost.rs` | 29 | 8 |
| `fanout_cost.rs` | 28 | 5 |
| `rank_and_instrument.rs` | 21 | 5 |
| `strat_cost.rs` | 7 | 5 |
| `accum_alpha_cost.rs` | 6 | 5 |
| `cascade_cost.rs` | 4 | 2 |
| `harvest_cost.rs` | 1 | 5 |
| **TOTAL** | **96** | **35** |

⛔ **A FIRST COUNT SAID 37 AND IS KEPT HERE.** The regex was `^\s*[a-z_]+ */= r;`, shaped from the
first site read (`fire /= r;`). It is blind to `let (a, b) = (a / r, b / r);` — how `harvest_cost`
and `strat_cost` spell it — and reported `rank_and_instrument.rs` as **zero** where the file has
**21**. `vocare` had named those exact sites; the orchestrator's grep contradicted the ward and the
ward was right.

**There are at least three spellings.** `x /= r;` · `let (a, b) = (a / r, b / r);` · `*x /= r;`
inside a loop. **Do not scope this strike with a `/= r` grep.**

## The model is one function above the defect

`calibrate_mark_ns` (`mod.rs`) is **already correct**: `best = f64::INFINITY`, `if ns < best`. Its
internal `/ PER_BATCH` is a legitimate per-iteration normalisation, not a mean across measurements.

**That is the discriminator, and it is exact:** dividing by a UNIT COUNT (facts, pairs, iterations)
is normalisation and stays. Dividing by **RUNS** is averaging across repeated measurements and is
the defect. `calibrate_mark_ns` is untouched by this strike and is the shape to copy.

## The algorithm

Each accumulator becomes min-tracking: seed `f64::INFINITY`, `x = x.min(sample)` per run, delete
the divide. `stat()` stops returning a mean.

⚠ **ONE GENUINE DESIGN QUESTION THE BRIEF DOES NOT DUCK.** `stat()` returns `(mean, lo, hi)` and
the row prints `mean` with `lo`/`hi` as the spread. Once the reported value IS the minimum, the
reported value and `lo` are the same number and printing both is noise. Shape: return `(min, max)`
and print the value with `max` as the spread. If the rider finds a reason to keep three, it must
say what the third column means.

## ★ THE ONE CONTRACT DECISION

**The LABEL follows the ARITHMETIC, never the reverse.**

The cheap way to make this strike's gate green is to change 35 headers from `MINIMUM` to `MEAN`.
That would pass every test and be **the original defect performed in the opposite direction** — a
label moved to match code instead of code moved to match a label. The estimator is wrong on its
merits: `89e8c3ed0`'s own measurement showed the first arm of each round paying a one-time cost of
**287.4 ms against 11.5 and 11.4 for identical work**, which is exactly what a mean cannot survive.

So: **convert the arithmetic.** If any single figure genuinely wants a mean, that figure's header
says MEAN and the reason is written beside it — but it is a finding to surface, not a default.

## The gate — and it needs no stopwatch

A source-level lint: **a test file that prints a `MINIMUM of` header may not divide by RUNS.** File-
scoped deliberately, which makes a PARTIAL conversion impossible to ship — a file must be fully
converted to pass, and a half-swept file is precisely the defect this strike exists to remove.

RED today on all seven files; GREEN when converted. **That is the mutation proof, obtained for
free, with no timing anywhere** — which matters because this arc has already had to strike two
timing assertions that reddened the floor under parallel load.

## Blast radius

`src/rete/kernel/tests/{mod,accum_cost,fanout_cost,rank_and_instrument,strat_cost,accum_alpha_cost,cascade_cost,harvest_cost}.rs`
— 8 files — plus one new `tests/lint/*.rs` (no registration: `build.rs` generates the module list).

## Out of scope — AFFIRMATIVELY CUT

- **`calibrate_mark_ns`** and every divide by a unit count. Named above; converting them would
  break normalisation that is correct.
- **Re-deriving the arc's recorded cost figures.** Every number in the record was read off this
  instrument and many will move. That is a curare job for after the estimator is trustworthy, and
  doing it inside this strike would mean re-recording numbers while the thing producing them is
  half-converted.
- **The two timing assertions already struck** (`2a7051c67`). Not related; do not reinstate them.
