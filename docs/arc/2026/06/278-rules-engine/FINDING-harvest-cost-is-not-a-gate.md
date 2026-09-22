# FINDING — `harvest_wrap_split` is not a gate. Its bias is SYSTEMATIC, not noise.

**Measured 2026-09-22** after it went RED inside 255.12's floor run 1
(`.floor/2026-09-22T08-13-55Z/`, captured whole, **not re-run to green**).
⛔ **"Timing" is not a disposition in this repo, so here is the mechanism.**

## The assertion

`src/rete/kernel/tests/harvest_cost.rs:337`

```rust
assert!(h >= (s + w) * 0.5 && h <= (s + w) * 2.0, …)
```

Observed: `h=9.73 ms`, `s=5.70`, `w=14.73` → lower bound `10.215`, **missed by 4.75 %**.

## ⛔ Why `w > h` is not the impossibility it looks like

The test's own message says one of the closures must be *"measuring something other than what its
name says"* — because `h` does scan **and** wrap, so `h ≥ w` should hold. ⭐ **It does not, and the
reason is the harness, not the code under test:**

```rust
for _ in 0..RUNS {                       // RUNS = 3
    s = s.min(ns_per_iter(1, …scan…));
    let collected = …;                   // built OUTSIDE the timed region
    w = w.min(ns_per_iter(1, || …map 40_000 PMaps over `collected`…));
    h = h.min(ns_per_iter(1, || …scan AND map 40_000 PMaps…));
}
```

1. ⛔ **`ns_per_iter(1, …)` is ONE iteration** — `t0.elapsed() / 1`. Each sample is a single
   wall-clock reading of one pass over 40 000 facts.
2. ⛔ **`w` ALWAYS runs before `h`.** `w` allocates 40 000 `PMap`s **first**, paying first-touch /
   page-fault / allocator-growth cost. By the time `h` allocates its 40 000, those pages are warm.
3. ⛔⛔ **`min` over 3 samples CANNOT cancel this.** The order is identical every run, so the bias on
   `w` is **systematic, not random** — taking a minimum reduces noise and leaves bias untouched.
4. The bound **tightens as `w` inflates**: an over-measured `w` raises `(s+w)*0.5` toward and past `h`.

⭐ **So the test fires exactly when the allocator effect is largest** — i.e. under a loaded machine.
It went red inside a **5 986-test parallel floor**. Prior record: 19 kept logs, all PASS at
0.120–0.264 s; the red run is the slowest of 20 at 0.279 s.

## Disposition

⛔ **This is NOT a flake and must not be recorded as one.** It is a **measurement-design defect** with
a named mechanism, and it **will fire again** — `[[feedback_a_wall_clock_ratio_is_not_a_gate]]`,
which this repo has already learned once.

**It is also NOT 255.12's diff** — verified: the timed closures call `PVec::iter`, an inline
`matches_class` closure, `PMap::from_pairs` and `Value::clone`; **none of 255.12's 8 changed files is
on that path**, and the file was last edited 2026-09-21 by a different stone (`b7c48b630`).

## What a cure must do — and what it must NOT

⛔ **DO NOT widen the bounds.** That is the "re-order the array to match the code" move: it makes the
gate say nothing rather than say something true. The apportionment claim is worth keeping — the test
exists to catch *"a phase silently dropping out of the combined measurement."*

Candidates, to be **measured, not chosen from this list**:
- **Hoist the allocation cost out of the comparison** — give `w` and `h` the same first-touch state
  (e.g. a warm-up pass before sampling, or alternate the order across runs so the bias cancels).
- **Sample more than one iteration** — `ns_per_iter(1, …)` defeats the function's own purpose.
- ⭐ **Assert the INVARIANT instead of the ratio.** The real claim is *no phase dropped out*; a
  count-based or work-based assertion (facts scanned, maps built) is deterministic and cannot be
  perturbed by a loaded runner at all. **This is the one I would try first.**

⚠ Its sibling cost tests use the same `RUNS = 3` / `ns_per_iter(1, …)` shape
(`strat_cost.rs:14`, `kernel/tests/mod.rs:505`) — **census them before curing just this one.**
