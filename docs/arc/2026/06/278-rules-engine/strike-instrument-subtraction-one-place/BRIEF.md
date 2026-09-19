# BRIEF — lift `raw − pairs × cal` out of two closures into one `fn`

## The work

Add one free function to `src/rete/kernel/tests/mod.rs` computing the instrument-subtracted value,
and call it from all **13** sites. Delete the two closures that currently do it privately.

**Arithmetic-preserving only.** Every number the cost tables print must be byte-identical after.

## Read in order

1. `.../strike-instrument-subtraction-one-place/DESIGN.md` — ⚠ **the row's count (20) is wrong; it
   is 13**, and `net_of` is a closure that cannot be called, not a canonical helper. Read why before
   you touch anything.
2. `src/rete/kernel/tests/mod.rs:473-482` — the doc comment that argues for exactly this extraction
   and is the reason the strike exists. **Your helper's doc should point back at it.**
3. `src/rete/kernel/tests/mod.rs:540-560` — `stat` and `net_of`, the imprisoned closure.
4. `src/rete/kernel/tests/accum_cost.rs:698` — **the same closure, invented independently.** This is
   the evidence; read it before you decide the helper's signature, because its shape
   (`|raw: f64, pairs: u64|`) is the one two authors converged on.
5. `src/rete/kernel/tests/mod.rs:303-308` — the class already biting once in this file: *"one
   instrument, two estimators, and the wrong one on the larger measurement."*
6. The 13 sites, listed in DESIGN.md with file and line.

## Implementation sketch

```rust
/// Instrument-subtracted nanoseconds: the raw reading less what the mark pairs cost.
/// … cite mod.rs:475-478 — this is the arithmetic that comment says must live in one place.
fn net_ns(raw: f64, pairs: u64, cal_ns_per_pair: f64) -> f64 {
    raw - pairs as f64 * cal_ns_per_pair
}
```

Then: `mod.rs:547`'s `net_of` closure keeps its key-lookup shape but its *body* becomes a call;
`accum_cost.rs:698`'s closure is deleted and its call sites call the helper; the ten inline sites
become calls.

⚠ **`fanout_cost.rs:828` and `strat_cost.rs:416` fold a minimum over the subtraction.** Keep the
fold at the call site — `.min(net_ns(ns as f64, k, cal))` — and do **not** move `min` into the
helper. The pinned contract decision says why.

## STOP triggers

1. **If any cost table's printed numbers change** — STOP and report the diff. This strike is
   arithmetic-preserving; a changed number means a site was not doing what the others do, and **that
   is a finding worth more than the refactor.** Capture it before touching anything else.
2. **If a site's operand types do not fit** (`u64` vs `f64` vs a borrowed count) without a cast that
   changes a value — STOP and report which. Silent widening/truncation is the failure mode here.
3. **If you find a 14th site** — report it; my 13 was derived once, by one grep, and the two counts
   before mine were both wrong.
4. Do not touch `calibrate_mark_ns`, any measured constant, or the min/mean estimator question.

## Blast radius

`mod.rs`, `accum_cost.rs`, `cascade_cost.rs`, `fanout_cost.rs`, `strat_cost.rs` — the 13 sites and
the two deleted closures. **No new tests, no behaviour change.**

## ⛔ Write `SCORE.md` AS YOU GO

Append each site as you convert it and each expectation row as you check it. An earlier executor
held its measurements only in context, backgrounded a floor, and stopped.
