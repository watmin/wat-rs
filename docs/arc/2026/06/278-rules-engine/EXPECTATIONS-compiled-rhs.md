# EXPECTATIONS — the compiled RHS

Written **before** the strike so the result cannot move the goalposts. Every command is the
orchestrator's own re-run; nothing is graded on the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the mechanism, by COUNT not timing | `cargo nextest run --release -E 'test(fanout_rhs_key_alloc_census)' --no-capture` | `match:key-alloc` **exactly 0** (was 120,000) |
| 2 | the zero is not a dead fire | same run | `prod:derivations` **40,000** |
| 3 | the class alloc is UNCHANGED | same run, `prod:class-alloc` | **40,000** — expected, not a miss |
| 4 | the differential — same VALUE, not same shape | included in row 5 (the rete differentials are in the floor) | green |
| 5 | semantics unchanged | `cargo nextest run --release` | **4241/4241**, exit 0 |
| 6 | the wall | `cargo clippy --release --workspace --all-targets -- -D warnings` | exit 0, **0** warnings |
| 7 | the fire, A/B in ONE batch | `fanout_per_call_alpha_census` x3 each side, stash between arms | recorded with ranges |
| 8 | the re-pointed gate still bites | revert one `Bind` to rebuild its key; row 1 must go RED | fires, naming the count |

Row 3 is on the card deliberately: the class `String` is still allocated per fact because
`AggregateValue::record` takes an owned `String`. A rider who "helpfully" interned it would be
smuggling in `109-kill-std/NOTE-keyword-storage-must-intern.md` and destroying the attribution. 40,000
is the pass condition.

## Independent prediction

**Runtime: 20–40 minutes.** A new module, a compile fn, an executor, two wiring sites, one test
re-pointed — with `compiled_cond.rs` sitting there as the same stone one layer up. There is no
discovery burden and the shape is dictated. Time-box at **2× the upper bound = 80 minutes**.

Calibration note carried forward: the last rider took **79 seconds against a predicted 8–15
minutes** (7× over). That was a pre-enumerated mechanical sweep; this one has real design in it, so
the band is wider — but if the pattern holds, expect the low end.

**Perf: RECORDED, NOT GRADED.** I am not predicting a millisecond figure and row 7 has no threshold.
The four `prod:*` marks fire 160,000 times on this cell, so a large fraction of the 17.45 ms they
report is instrument; extrapolating from them is exactly the error that made compiled-conditions'
own estimate wrong by 10×. What IS predicted is the mechanism: **240,000 heap allocations
disappear** (120,000 `String` + 120,000 `Arc`). Whether that is 2 ms or 8 ms of a ~76 ms fire is
what the A/B is for.

The one directional claim I will make: `prod:validate` and `prod:shape` should approach zero on the
compiled path, because the work they bracket moves to setup. If they do **not**, the compiled path
is not actually being taken — check the fallback arm before believing any other number.

## Trap doors named in advance

- **`kernel.rs:5768`.** The `match:key-alloc > 0` assertion inverts. Deleting it is the failure;
  re-pointing it (with a non-vacuity guard) is the fix. Row 8 exists to prove it still bites.
- **The doc comment above that test** says "every remaining count is the production pass" — false
  once the count is zero. Prose outliving its property is the defect even when the assertion is
  right.
- **A silent fallback.** If `compile_rhs` returns `None` for the fanout rule, everything still
  passes and nothing improves — row 1 catches it (the count stays 120,000), which is why the
  mechanism is gated on a counter and not on a stopwatch.
- **Scope creep into interning.** Row 3 pins it.
- **Load on the box.** Row 7 is only comparable within one batch; both arms run adjacent with a
  stash between them, never against a figure quoted from earlier in the session.

## What "done" means

Rows 1–6 and 8 pass on the orchestrator's own re-run, with their own exit codes read, before
anything is committed. Row 7 is recorded and reported with ranges, honestly, including whether they
overlap.
