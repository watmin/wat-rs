# BRIEF — the label follows the arithmetic

Make every table that says `MINIMUM of {RUNS}` actually take the minimum, and add a source-level
gate so a label can never again outrun its estimator. Read `DESIGN.md` first — its ★ ONE CONTRACT
DECISION rules out the cheap way to make this green, and its population table is measured, not
grepped.

## Read in order, and why

1. **`src/rete/kernel/tests/mod.rs:490-570`** — `render_phase_table`. Line 493 is the label
   (`89e8c3ed0`); 528-542 is `stat()` returning a mean, plus `net_of` and `total_mean`, which both
   read `stat(..).0`; the row loop at ~564 binds `let (mean, lo, hi)`. **This one function renders
   the axis tables for fanout, accum, node-share and rank-and-instrument** — fix it and four axes
   move at once.
2. **`src/rete/kernel/tests/mod.rs`, `calibrate_mark_ns`** — **the model.** Already correct:
   `best = f64::INFINITY`, `if ns < best`. Its `/ PER_BATCH` is normalisation and stays. Copy this
   shape; do not touch this function.
3. **`src/rete/kernel/tests/accum_cost.rs:596-630`** — a worked instance of the defect: `x += of(…)`
   inside `for _ in 0..RUNS`, then twelve `x /= r` and two `for x in &mut … { *x /= r }`, under a
   header at :652 reading `MINIMUM of {RUNS}`.
4. **`src/rete/kernel/tests/harvest_cost.rs:598`** and **`strat_cost.rs:225`** — the spelling a
   `/= r` grep cannot see: `let (a, b) = (a / r, b / r);`. **Read both before you scope anything.**
5. **`tests/lint/no_ceiling_raise_in_rete.rs:92`** — the non-vacuity guard with its reason written
   out; yours needs one. **`tests/lint/mod.rs`** — three lines; `build.rs` generates the module
   list, so you register nothing.

## The order

1. **Write the gate first.** A test file printing a `MINIMUM of` header may not divide by RUNS.
   File-scoped, deliberately — it makes a partial conversion unshippable.
2. **Run it. CONFIRM RED**, naming **seven** files. Quote it verbatim.
3. Convert: `stat()` and the 96 sites.
4. **GREEN.**
5. Mutation-prove: restore one converted accumulator to `+= … / r` → that file alone reddens.
   Restore. Then mutate the gate's own reader (e.g. make it look only for `/= r`) and confirm it
   **stops seeing** `harvest_cost`/`strat_cost` — that is the orchestrator's own wrong regex, and
   proving the gate is immune to it is worth one build.

## Implementation sketch

```rust
// per accumulator, per run:
let mut x = f64::INFINITY;
for _ in 0..RUNS { /* … */ x = x.min(sample); }
// and DELETE the `x /= r`
```

For `stat()`: return `(min, max)` and print the value with `max` as the spread. Once the reported
value IS the minimum, `lo` and the reported value are the same number — see DESIGN's ⚠. If you keep
three columns, say what the third means.

## Blast radius

8 test files + 1 new lint. If a ninth `src/` file needs touching, STOP and surface it.

## STOP triggers

1. **If you find yourself changing a `MINIMUM` header to `MEAN` to make the gate pass, STOP.**
   That is the ★ decision failing — the original defect run in reverse.
2. **If a divide is by a UNIT COUNT (facts, pairs, iterations), leave it.** Only divides by RUNS
   are the defect. `calibrate_mark_ns` is the worked example of a legitimate one.
3. **If converting moves an ASSERTION** — a test that goes red because a number changed — STOP and
   surface it. Checked before drawing: the assertions in these seven files are liveness (`> 0.0`)
   and fixture pins, and none compares an accumulated ns value to a magnitude constant. If that
   turns out wrong, the check was wrong and I want to know.
4. **If any single figure genuinely wants a mean, STOP and surface it** rather than quietly
   relabelling it. It is a finding.

## What the report must contain

The count you converted, **per file**, against DESIGN's table — and if your number differs from 96,
say so and say why. The orchestrator's first count was 37 and wrong; a second wrong count that
merely agrees with the first would be worse than a disagreement.

---

## MEASURED, after the strike — the population was 103 sites across 9 files, not 96 across 7

The gate's own RED is the instrument. **DESIGN's 96 is a LINE count; 103 is a SITE count**, and
the two extra files are real, not bookkeeping:

| file | DESIGN (lines) | gate RED (sites) | why they differ |
|---|---:|---:|---|
| `accum_cost.rs` | 29 | 29 | — |
| `fanout_cost.rs` | 28 | 28 | — |
| `rank_and_instrument.rs` | 21 | 22 | +1: `mean(xs) = sum / xs.len()` at :140, not a `/ r` |
| `strat_cost.rs` | 7 | 10 | +3: three lines are `let (a, b) = (a / r, b / r)` — two divides each |
| `accum_alpha_cost.rs` | 6 | 6 | — |
| `cascade_cost.rs` | 4 | 4 | — |
| `harvest_cost.rs` | 1 | 2 | +1: same destructured line, two divides |
| **`mod.rs`** | *not counted* | **1** | `render_phase_table::stat` — `sum as f64 / xs.len()`, the flagship |
| **`gather_probe_cost.rs`** | *not in the 7* | **1** | `let runs = RUNS as f64; g /= runs;` — the alias is `runs`, not `r` |
| **TOTAL** | **96** | **103** | |

**Two findings the scoping pass could not have had:**

1. **`gather_probe_cost.rs` is a NINTH file**, outside BRIEF's declared blast radius. Its
   `probe_gap_cost_split` holds six accumulators; `89e8c3ed0` converted **five** (`r`/`s`/`p`/`e`/`j`
   are all `f64::INFINITY`-seeded) and left `g` a mean, under one shared `MINIMUM of {RUNS}` header.
   A ratio built from five minima and one mean. It escaped every count because its divisor is named
   `runs`, so even a `/ r\b` regex is blind to it — the same class of miss as the original 37.
2. **STOP trigger 3 fired.** BRIEF asserted "none [of these assertions] compares an accumulated ns
   value to a magnitude constant". `rank_and_instrument::fold_cost_with_and_without_the_binding_lookup`
   does: `assert!(c < 5.0e6, …)` and `assert!(s <= c * 2.0 || s < 8.0e6, …)`, where `c`/`s` were the
   means. The conversion moves both bounds in the SAFE direction (a minimum is ≤ a mean, so the
   `<` bounds only loosen) and no test moved — but the pre-drawing check was wrong, and a future
   strike over this ground must not inherit the claim.
