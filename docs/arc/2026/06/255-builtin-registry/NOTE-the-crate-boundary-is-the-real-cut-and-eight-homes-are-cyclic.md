# NOTE — the CRATE boundary is the real cut, and eight homes are CYCLIC with `runtime.rs`

> Measured 2026-08-31 after the builder named the trajectory. **No row, nothing drawn.** This exists
> so the near-term decomposition is cut against the boundary it must eventually satisfy, instead of
> being done twice.

## The builder's stated trajectory

> *"long term… `wat-rs/src/*.rs` is likely to only hold a `lib.rs`… everything else is in
> `wat-rs/src/<some-home>/<some-files>.rs`"*
>
> *"long longer term (maybe like a week out?) we'll break up nearly everything in
> `wat-rs/src/<some-home>/` into `wat-rs/crates/<some-crate>/`"*

That is ROAD step 2 (`break into crates`) with a shape and a date.

## ⛔ THE MEASUREMENT — eight of nine homes cannot be lifted today

```
                  runtime -> home      home -> runtime
  rete                 23                    80        CYCLE
  intrinsic            32                   242        CYCLE
  collection           45                     9        CYCLE
  edn                  45                    58        CYCLE
  macros               18                    28        CYCLE
  resolve              51                     4        CYCLE
  types               182                     1        CYCLE
  check                38                    10        CYCLE
  value                47                     0        ✅ ACYCLIC — liftable TODAY
```

A Rust crate graph is a DAG. **A cycle is not a refactor cost; it is a hard blocker.** So the
near-term move (`src/*.rs` → `src/<home>/`) is not a relocation — **it is the work of making each
home acyclic**, and a home cut without that check is a home that must be cut again.

## ★★★ AND THE REGISTER-IN-PLACE PATTERN IS BUILDING THE BLOCKER

**160 of `intrinsic`'s 242 back-edges are `crate::runtime::eval_*`** — the thin-delegate calls this
campaign has been creating deliberately, one per homed verb, under the standing brief instruction
*"bodies do not move."*

That instruction came from W3's ruling, which was sound on **its** axis (moving a body that leans on
module-private helpers costs a dozen `pub(crate)` widenings for no behavioural gain). It was never
weighed against the crate boundary, because the crate boundary had not been stated yet.

★ So the pattern has three effects, and only the first two were ever counted:
1. ✅ it fires the consistency gates — doc lies, arity truth, `apply` reachability, purity rulings;
2. ⚠ it does not shrink the megafile (`[[NOTE-homing-in-place-does-not-shrink-the-megafile]]`);
3. ⛔ **it adds an `intrinsic → runtime` edge per verb, which is the step-2 blocker for the largest
   home in the tree.**

`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]` — W3's premise did not survive the
builder naming the crate split, but the ruling was still being applied verbatim this session.

## The asymmetries name the cheap wins

The back-edge counts are wildly uneven, and small ones are near-free to sever:

```
  value        0   ✅ liftable now — the proof-of-pattern candidate for step 2
  types        1   ONE reference stands between a 7,250+2,274-line home and a crate
  resolve      4
  collection   9
  check       10
  macros      28
  edn         58
  rete        80
  intrinsic  242   ← 160 of them created by this campaign
```

⚠ **`types → runtime` being ONE reference is the single highest-leverage fact here** and it has not
been looked at. If that one edge is severable, a 9,524-line home becomes a crate.

## What this does NOT say

It does not say stop homing. The registry campaign's yield is correctness, and killing the dispatch
match is the thesis of arc 255 and a hard prerequisite for ROAD steps 3–4.

It says the *default* changes: **a body that can move should move**, because each one that stays is
an edge someone pays for at step 2. The instrument for "can it move" is **rustc**, not a grep over a
hand-list of helper names — move it and read the screams
(`[[feedback_impose_the_check_and_read_the_screams]]`).

## ⬜ The question, NOT drawn

> Should the near-term `src/*.rs` → `src/<home>/` decomposition be **ordered by back-edge count**
> (`value` → `types` → `resolve` → `collection` → `check` → …), so each home lands acyclic and
> step 2 is a `Cargo.toml` edit rather than a second decomposition?

That ordering is derivable from the table above, and it is the opposite of "start with the biggest
file." It is the builder's ruling.

---

## ⛔ AMENDED 2026-09-01 — MOST OF THOSE CYCLES ARE RE-EXPORT ARTIFACTS

The table above counts `crate::runtime::` references and calls each one a back-edge. **That
over-states the coupling badly**, and the mechanism is one line-range:

```
src/runtime.rs:759-784   pub use crate::value::{ Environment, SymbolTable, Function,
                                                 EvalBreak, Value, TrackedValue, … }   22 names
```

`runtime.rs` **re-exports the whole `value` module**. So a home that writes
`use crate::runtime::SymbolTable` is not depending on the runtime at all — it is reaching a
`crate::value::` type through a facade. `src/check.rs:56` does exactly this:
`use crate::runtime::{Function, FunctionBody, SymbolTable};` — all three live in `src/value/`.

**Re-measured, splitting each home's references into re-exported-`value` versus genuine-`runtime`:**

```
                 refs   re-exported value types      genuine runtime
  resolve           6     5   (83%)                    1
  macros           31    26   (83%)                    5
  rete            172   128   (74%)                   44
  edn              66    46   (69%)                   20
  collection       33    22   (66%)                   11
  intrinsic       278    27    (9%)                  251   ← the outlier
```

★★ **Re-pointing an import dissolves a cycle without moving a line of implementation.** `resolve`
is ONE genuine reference away from acyclic; `macros` is five. That is a mechanical sweep, not a
decomposition.

★★★ **And `intrinsic`'s 9% is the exception that proves the shape.** Its 251 genuine references are
the DELEGATE-BACKS — the edge calling implementations that never got a home and are still squatting
in `runtime.rs`. That is not an import artifact and cannot be swept; it is fixed by giving those
impls homes (`[[DESIGN-STONE-the-numeric-home]]` is the first).

## So the crate campaign is TWO distinct moves, not one hard one

1. **Re-point re-exported imports** — `crate::runtime::X` → `crate::value::X` wherever `X` is one of
   the 22 re-exported names. Mechanical, no logic moves, dissolves most of the cycle count.
2. **Home the squatting impls** — the 69 domain implementations behind 11 homeless edges. Real work,
   one home at a time, numeric first.

⚠ **And this reframes the ORDER question the NOTE closed on.** It is no longer "which home moves
first" — move 1 is nearly free and independent of move 2, and it is what makes the *other* homes
liftable. Whether it goes first is still the builder's ruling; what changed is that it is cheap.
