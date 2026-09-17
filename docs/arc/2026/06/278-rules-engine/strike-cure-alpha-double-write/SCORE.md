# SCORE — D7's cure, weighed against the orchestrator's own re-run

> **The cure lands. `native=2 oracle=3` → `native=3 oracle=3`.** The finding is that **mutation 2's
> detector did not exist** — for the second consecutive brief I named an instrument structurally
> blind to what I pointed it at.

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ repro agrees with the oracle | ✅ **`native=3 oracle=3`** |
| 2 | ★ one writer per `aid` per pass | ✅ class-uniform batching |
| 3 | width control unchanged | ✅ `wide=3 narrow=3` |
| 4 | a differential gate on the floor | ✅ 7 arms; **RED under mutation 1, driven by me** |
| 5 | the decision is read | ✅ — but only via a detector the rider had to **build**; see B |
| 6 | element ordering preserved | ✅ by construction — deferred activate runs in fact order |
| 7 | hot-path cost stated | ✅ **+1.9% median**, 27 samples/build, quartiles overlapping |
| 8 | `leaf_occ` not used as the gate | ✅ |
| 9 | floor | ✅ **`5344 tests run: 5344 passed, 21 skipped`** (5336 + 8) |
| 10 | lints / clippy | ✅ 210/210, rc=0 |

## ⭐ A — THE CURE, AND A BETTER ARGUMENT THAN MINE

**Class-uniform batching**: `class_ids` carries an "every fact of this class packed" flag; a uniform
class batches, a mixed class has *all* its facts activated in fact order. One writer per `aid`, and
ordering preserved **by construction** rather than by care. Radius: **one fire-path file**
(`pass/alpha.rs`); `delta.rs` untouched.

Its argument against shape 2 is stronger than the DESIGN's. I said declared-schema packability costs
new state; the rider added that it is **strictly more conservative** — `Box[T]`'s declared `v <- :T`
is not `i64`, so a fire whose boxes all hold i64 would lose a batch class-uniformity keeps. **New
state for a worse decision.**

**Mutation 1, driven by me** — reverting only `pass/alpha.rs`:

```
FAIL mixed_packability_erased_first   native="1,2,"  oracle="0,1,2,"
FAIL mixed_packability_alternating    native="0,2,"  oracle="0,1,2,3,"
FAIL mixed_packability_record_filler  native="0,2,"  oracle="0,1,2,"
FAIL mixed_class_beside_uniform_class_with_join   native="0,2," oracle="0,1,2,"
```

Both uniform controls stayed green — correct, they were never affected. **The gate compares sorted
key SETS, never counts**, which this defect demands: the aliasing half produces a right-sized wrong
answer.

## ⛔⛔ B — MUTATION 2's DETECTOR DID NOT EXIST, AND I NAMED IT ANYWAY

My BRIEF: *"make a uniformly-packable class fail to batch → **the width control or a perf arm** must
show it."* Driven: with batching disabled entirely, the width control still printed
`wide=3 narrow=3` and **all seven differential arms stayed green**. Batching-versus-activating is
**correctness-invariant by design** — that is the point of the batch — and there was no perf arm on
the floor.

**As briefed, mutation 2 had no possible RED.** A rider following it literally reports "mutation 2
shows nothing" or skips it. The rider instead **built** the detector — three `#[cfg(test)]` census
counters plus a second gate in `pass_semantics.rs` — before the mutation could mean anything.

⛔ **Second consecutive brief in which I pointed at a structurally blind instrument** (last: the
`leaf_occ` differential, C16). The pattern is mine, not the riders': **I name an observable without
asking what would have to change for it to move.**

And **STOP-4 was unfalsifiable for the same reason** — I told it to stop if the width control
changed, and narrowing batching does not change it. A trigger that cannot fire for the condition it
names reads as coverage while providing none.

## ⛔ C — A RED IT CAUSED, DISCLOSED, AND FIXED CORRECTLY

Its new counter made `accum_matcher_op_census`'s `rows.len() == 10` read 11. That pin is
`[[a-count-in-a-scorecard-bounds-the-executor]]` exactly — a bare cardinality caps instrumentation
downward and cannot see a swap. Replaced with a sorted **name-set** over all 11 counters, which is
the claim the surrounding sentence already made.

## ⚠ D — TWO CORRECTIONS TO THE RECORD IT SURFACED

1. **`session.rs:256`'s doc is the thing that is wrong**, not the code it describes. *"`None` = not
   all **declared** fields i64"* should say **runtime** — or the sentence stays false. The DESIGN
   called the gap "the defect"; the gap is real but the *doc* is the wrong half to keep.
2. **`arm.rs:330-336` proves the bug; `alpha_tree.rs:180 root_for(class)` licenses the cure.** A fact
   of class C reaches only C's aids — without that, deferring C's activations past the batch loop
   could reorder elements at an aid shared with another class. My read-list cited the half that
   explains the defect and not the half that makes the fix sound.

## Per-arm status

| arm | status |
|---|---|
| the cure; `native=3 oracle=3` | **proven** — driven by me |
| Gate A REDs on revert | **proven** — driven by me, 4 arms quoted |
| Gate B REDs when batching narrows | **proven** (rider) |
| width control | **proven**, four states |
| element ordering | **proven by construction**, not independently driven — disclosed as such |
| hot-path cost | **measured** — +1.9% median, overlapping quartiles; batching itself worth 30.8% |
| a third writer | **none** — `wm.alpha` written in exactly two places in the seed pass |
