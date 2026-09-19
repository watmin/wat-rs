# DESIGN — the instrument subtraction was never extracted, only imprisoned

**Status:** drawn 2026-09-08 from vigilia row **`4S2`** (solvere). ⚠ **The row's own count is wrong
and is corrected here.**

## Why

`src/rete/kernel/tests/mod.rs:475-478`, on `render_phase_table`:

> *"Extracted 2026-08-01 when node-share needed the same table accum already had. **Copying it would
> have put the instrument-subtraction arithmetic in two places**, and the whole reason that
> arithmetic exists is that a table which misreports its own instrument is worse than no table —
> **two copies is how one of them silently stops subtracting**."*

The arithmetic is `raw − pairs × cal_ns_per_pair`.

## ⛔ What the row got wrong, and it changes the cure

`4S2` cites `mod.rs:546` as *"the canonical formula"* and says it is hand-rolled at **20** further
sites, as though the sites were bypassing a shared helper.

Re-derived this session:

* **`net_of` is a CLOSURE inside `render_phase_table`'s body** (`mod.rs:546-548`), not a function.
  **It cannot be called from anywhere.** So the 20 sites are not bypassing an abstraction — **there
  is nothing to bypass.** The comment claims an extraction that happened only at the *table* level;
  the *arithmetic* it names was never lifted at all.
* **The count is 13, not 20.** Thirteen subtraction expressions across six files. My 20 (and the
  ward's 17) inflated by counting `cal` *mentions* — `let cal = calibrate_mark_ns()`, comments,
  unrelated uses — as subtractions.

| file | sites |
|---|---|
| `fanout_cost.rs` | `:425, :426, :427, :558, :730, :731, :732, :828` (8) |
| `cascade_cost.rs` | `:393, :396` (2) |
| `mod.rs` | `:547` — the imprisoned closure |
| `accum_cost.rs` | `:698` |
| `strat_cost.rs` | `:416` |

## ⭐⭐ The fact that decides the strike

**`accum_cost.rs:698` is already the identical closure, invented independently:**

```rust
let net = |raw: f64, pairs: u64| raw - pairs as f64 * cal;
```

against `mod.rs:547`'s

```rust
let net_of = |k: &str, xs: &[u64]| stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair;
```

**Two files independently arrived at the same abstraction. It has been invented twice and shared
zero times.** That is not a style preference; it is the comment's own predicted failure, already
half-realised.

## ⛔ And the file records the failure ONCE ALREADY

`mod.rs:303-308`, unprompted, on a different instrument:

> *"The calibration constant used the minimum. The splits it feeds used the mean. **One instrument,
> two estimators, and the wrong one on the larger measurement** — which is exactly what
> `render_phase_table`'s own doc warns about: 'two copies is how one of them silently stops
> subtracting.'"*

**This class has already produced one real defect in this very file.** The strike is not
hypothetical hygiene.

## What this delivers

One `fn` in `mod.rs` — `raw − pairs × cal` — called by all thirteen sites. The estimator question
(minimum vs mean) then has exactly one place it can be answered, which is what `:303-308` says was
missing when it bit.

## The one contract decision, pinned

⛔ **A FREE `fn`, NOT A METHOD AND NOT A CLOSURE.** Both existing spellings are closures capturing
`cal` from their scope, which is precisely why neither can be shared. The helper takes `cal`
explicitly. **Two sites fold a minimum over the subtraction** (`fanout_cost.rs:828`,
`strat_cost.rs:416`: `.min(ns as f64 - k as f64 * cal)`) — they keep their fold and call the helper
*inside* it. Do not absorb the `min` into the helper; the estimator is the caller's decision and
`:303-308` is what happens when that decision is made in two places.

## Out of scope = rejected

- **Unifying the min/mean estimators.** Real, related, and a different strike — this one gives it a
  single site to happen at.
- **`4T1`'s positional triple.** Same file family, different abstraction. Separate strike.
- **Touching `calibrate_mark_ns` or any measured constant.**
