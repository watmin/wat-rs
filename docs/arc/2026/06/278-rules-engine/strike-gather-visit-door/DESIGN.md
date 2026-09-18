# DESIGN — `GATHER_VISITS` counts some of the examining, and a perf gate rests on all of it

> Drawn 2026-09-05 at HEAD `d878408f7`. Source: vigilia 2026-09-05 `recon/census-name-audit.md`
> item C. **Verified on disk by the orchestrator at THIS HEAD — and the audit OVERSTATED it.**

## ⛔ Correcting the source first

The audit named five uncounted scan paths. **Three are real; two are not**, and briefing all five
would send the rider to instrument two O(1) reads that have nothing to examine:

| site | shape | genuinely uncounted scan? |
|---|---|---|
| `fire/acc.rs:407` | `let gathered: Vec<&Element> = bucket.iter().map(…).collect()` | **YES** — materialises every element |
| `fire/pass/accumulate.rs:252` | `gathered.extend(bucket.iter().map(…))` | **YES** |
| `fire/mod.rs:~1980` | `return bucket.iter().map(\|&i\| { … })` (no-`SeedCmp` arm) | **YES** — per-element work |
| `fire/acc.rs:339` | `AccFold::Count => fold_i64s(fold, empty(), bucket.len())` | **no** — `len()` is O(1); nothing is examined |
| `fire/mod.rs:1949` | `if !compiled.has_seed_cmp() { return !bucket.is_empty() }` | **no** — O(1) |

A recorded finding is a claim until it is re-read. This one was 3/5.

## The defect

`census.rs`'s declaration: *"one element **EXAMINED** by an Accumulate / Negation / Exists gather"*,
and it argues its own importance: *"Counting the EXAMINATIONS — rather than the wall-clock — is what
makes the keyed-gather gate honest."*

Seven sites call `census_gather_visit()`. **Three paths examine every element of a bucket and call
it zero times.**

## ★ And the gate it makes honest cannot see them

`tests/rank_and_instrument.rs::keyed_gather_visits_do_not_scale_with_group_count` drives two runs
with a constant 800 elements, moving only the token count, and asserts:

```rust
assert!(small > 0, "…recorded ZERO — the gathers were never entered…");
let ratio = big as f64 / small as f64;
assert!(ratio <= 2.0, "gather visits scale with the TOKEN count … the gathers are still scanning
        the whole memory per token instead of probing a key index");
```

Its non-vacuity guard is real and it does bite on the *counted* paths. But **a regression to
whole-memory scanning on any of the three uncounted paths registers zero visits**: `small` and `big`
do not move, the ratio holds, and the gate reports the mechanism is intact.

The gate's own message names the failure it is blind to — *"still scanning the whole memory per
token"* — for three of the paths that could do it.

## The one contract decision, pinned

**Make the name true: an element examined by a gather is counted, and a gather path that examines
without counting has no form.**

- **Check rung:** count at the three real sites.
- **Shape rung:** one counting helper the gather paths iterate through — `bucket.iter()` in a gather
  arm becomes a call that counts as it yields — plus a lint over the gather modules refusing a raw
  bucket walk outside it, with a per-site rune for a walk proven not to examine (the two O(1) reads
  above are the shape of a legitimate exemption, if they ever gain a `.iter()`).

This is the shape F1 produced: one verb, and a lint whose exemption list is visible.

## ⛔ Expect the ratio to MOVE, and DO NOT tune the threshold

`ratio <= 2.0` is asserted over a quantity that is about to grow three new contributors.

**If the ratio exceeds 2.0 once the missing paths are counted, that is a FINDING, not a threshold to
raise.** It would mean the gathers *are* scanning with the token count on a path the instrument has
never watched — the exact regression this gate exists to catch, hidden by its own blindness since
the paths were written.

Report both readings — before and after — whichever way it goes.

## Scope

**IN:** the three sites, the counting helper, the lint, and the re-read ratio. Floor GREEN.

**OUT, affirmatively cut:** the two O(1) reads (nothing to count — say so, do not instrument them);
the remaining census sections (B, D–M); `temperare`'s cost rows, which become readable once the
counters mean what they say; A4, D2p, F2.
