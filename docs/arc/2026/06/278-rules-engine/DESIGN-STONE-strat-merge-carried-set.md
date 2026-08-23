# DESIGN-STONE — the stratified membership set is carried, not rebuilt

> **Origin (2026-08-23).** Theater hunt T1
> (`NEXT-STRIKES-theater-hunt.md`). Catch-up take-left LANDED
> (`e21b7fba`). The stratified loop accumulates a closure across
> strata; `merge_facts` rebuilds its membership set from that whole
> closure on every call. The set is knowledge the previous iteration
> already held and threw away.

## The enemy

```
for s in 0..=max_s:                       // rules.rs:107
    ...
    acc_facts = merge_facts(&acc_facts, &new_derived)   // rules.rs:202

merge_facts(facts_pv, derived):           // mod.rs:1177
    pv = facts_pv.clone()
    present: HashSet<Value> = pv.iter().cloned().collect()   // ← the whole closure
    for f in derived:
        if present.insert(f.clone()): pv.push_back_mut(f.clone())
```

`acc_facts` is the accumulated closure. Every stratum re-hashes and
Arc-clones **every fact derived so far** to re-learn a set the prior
iteration built and dropped. This is the seed `(0..n).collect()` enemy
(`DESIGN-STONE-seed-d-alpha-range`) one tier up: reconstructing an answer
already computed. The `HashSet` itself is NOT the enemy — the linear
`.any()` it replaced was the documented `[7,3000]` quadratic hang. The
enemy is **where** it is built.

`acc_derived_set` two lines below (`rules.rs:206`) is already carried
across the loop. `acc_facts` never got the same treatment.

## The measurement that convicted it

`strat_merge_present_parts` — isolated, the strat-neg `[6 2000]` ladder
(2000 `Item` seeds, ~1000 derived per stratum, closure 8000), mean of 3:

| lump | ms |
|---|---:|
| A REBUILD per stratum (today) | 3.47 |
| B CARRIED across strata | 1.24 |
| **A−B the theater** | **2.23** |

Hashes: **REBUILD 27000 · CARRIED 8000 · wasted 19000.** The cell is
13.6 ms, so the theater is ~16% of it. Written as a DISCONFIRMING probe
first: under the 0.5 ms gate the strike stops and the rebuild is recorded
as physics. It cleared the gate 4×.

## The algorithm

```
present = match acc_facts:                // built ONCE, mirrors merge_facts' first call
    PersistentVector(v) => v.iter().cloned().collect()
    _                   => HashSet::new()

for s in 0..=max_s:
    ...
    acc_facts = merge_facts(&acc_facts, &mut present, &new_derived)

merge_facts(facts_pv, present, derived):
    pv = facts_pv.clone()
    for f in derived:
        if present.insert(f.clone()): pv.push_back_mut(f.clone())
```

Same value-dedup semantics, same `push_back` order, same closure Value.
The `_ => PVec::new()` arm is mirrored by `_ => HashSet::new()`, so a
non-PersistentVector `facts` behaves exactly as today.

**Why the carried set stays exact:** the set is seeded from the same
contents `merge_facts` would have collected on call 1, and thereafter
receives an insert for precisely every fact that is pushed. Dedup within a
batch, across batches, and across strata are all unchanged.

## ★ THE ONE CONTRACT DECISION

**The membership set is carried across strata, not rebuilt per stratum.**
Dedup stays **value** equality (R18 — merge is value-dedup, NOT concat);
it does not become identity or id-keyed. The oracle reads
`(:wat::rete::Session/facts fired)` and must see a byte-identical closure.

## The gate

1. `strat_merge_present_parts` prints A/B/A−B. Already ≥ 0.5 ms.
2. In-fire: strat-neg `[6 2000]` grid `:wat-ns` drops. The grid's own
   noise floor on this cell is ±0.4 ms (`T19-17-35Z` vs `T21-28-42Z`,
   same HEAD), so only a drop clearly above that counts.
3. 7strat 3/3 including three-stratum.
4. `spec_equals_native_on_every_where_family` green.
5. Floor `scripts/floor.sh` GREEN. Clippy `--release --workspace
   --all-targets -- -D warnings` silent.

## Predicted win

Independent guess (written before the cut): the isolated probe is the
**upper bound** — in-fire the allocator and cache state differ, and
`merge_facts` is not the whole stratum. Expect strat-neg `[6 2000]`
**−1.0 to −2.2 ms** (13.6 → ~11.5–12.6). `[6 1000]` and `[6 500]`
proportionally smaller. Single-stratum axes (fanout, accum, negation,
deep-cascade) **unchanged** — they call `merge_facts` once, where rebuild
== build.

## Blast radius

`fire/mod.rs` `merge_facts` signature + body. `fire/rules.rs` the
`present` initializer + the one call site. `kernel/tests.rs` the probe.
No `.wat`. No Session field. No `AlphaDelta` / `BetaMemory` type change.
`merge_facts` has exactly ONE non-test caller.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. Array1. 297.
- Dedup by fact id / identity instead of value — changes R18 semantics.
- Hoisting `acc_facts` out of `Value` into a native `Vec` in the frozen
  Session.
- Folding T5 (`acc_derived` double clone) into this stone — it is a
  separate strike and stays on the list.

## Sequencing

1. Probe. Rank. (DONE — 2.23 ms.)
2. Carry the set. Weigh in-fire. Stop.
3. Revert if 7strat red, spec_equals red, or the strat-neg cell does not
   drop above the grid's noise floor.

## Weigh (2026-08-23) — LANDED

Floor **GREEN** `.floor/2026-08-23T21-53-49Z` — **4928 passed**, 19 skipped,
273.861s, no ARM (4927 + `strat_merge_present_parts`). Clippy CI-identical
(`--release --workspace --all-targets -- -D warnings`) **silent, exit 0**.
Rete cohort **358/358** including `spec_equals_native_on_every_where_family`
and `grid_axes_run_and_derive_nonvacuously`. 7strat three-stratum
differential **3/3**.

Same-session grid, `GRID_SKIP_ORACLE=1 GRID_RUNS=5`, before then after:

| cell | before | after | delta |
|---|---:|---:|---:|
| strat-neg `[6 500]` | 3.2154 | **2.8865** | **−0.329** (−10.2%) |
| strat-neg `[6 1000]` | 6.4038 | **5.6640** | **−0.740** (−11.6%) |
| strat-neg `[6 2000]` | 13.1738 | **11.7679** | **−1.406** (−10.7%) |

`:accuracy :match` and `:winner :us` on all three. Ratio vs Clara at
`[6 2000]` **50.79 → 57.80×**. The `[6 2000]` drop is **3.5× that cell's
measured noise floor** (±0.4 ms, `T19-17-35Z` vs `T21-28-42Z` at one HEAD),
so it is the engine and not the instrument.

**The prediction held.** Written before the cut: *−1.0 to −2.2 ms at
`[6 2000]`*; actual **−1.406**. The isolated probe's 2.23 ms was correctly
called an upper bound — in-fire the loop pays allocator and cache costs the
probe does not.

**Honest deltas.**
- The win is **~10–11% at every rung**, not just the top one. The probe
  predicted the effect would grow with strata×facts, and it does in absolute
  ms, but the *proportion* is flat across the ladder — `merge_facts` is a
  roughly constant share of each stratum.
- `wall-ms` barely moved (449 → 440 at `[6 2000]`): `fire-share-pct` is 2.67,
  so the timed region is ~3% of the program. The **engine** got 10.7% faster;
  the program did not, and the two must not be conflated
  (`run-axis.sh` keeps `:ratio` and `:wall-ratio` separate for exactly this).
- Single-stratum axes were predicted unchanged and were not re-measured in
  this weigh — `merge_facts` is called once there, where rebuild == build,
  and the full floor + cohort cover correctness. A full grid is the record.

**What this does NOT claim:** not that the stratified loop is now clean —
T5 (`acc_derived` double clone, `rules.rs:206`) is the sibling theater in the
same loop and is still standing. Not that `merge_facts` is free — the carried
path still pays 8000 hashes, which is the honest O(N). Not that the
`HashSet` was ever the enemy; the `.any()` scan it replaced was the
documented `[7,3000]` quadratic hang.

**The gate that was retired mid-strike.** This stone was drawn with a
"≥ 0.5 ms" gate. Builder, 2026-08-23: *"that 0.5ms gate criteria... it may
have outlived its purpose now.... we are basically at the physics boundary
with a few bumps to smooth over."* The standard is now **any named drop an
`Instant` proves**. Recorded in `NEXT-STRIKES-theater-hunt.md` § Rules of
engagement, rule 0. This strike cleared either bar; the change matters for
T3/T4/T7/T8, which are all sub-ms.
