# DESIGN-STONE — first delta is the facts vector; `seen` is filled once

> **Origin (2026-08-18).** Weigh after 2b. `[200 200]` quiet FIRE
> ~84 ms. Named leftovers: **SETUP ~14 ms**, drop ~10 ms,
> `accum:index` ~8 ms. Persist is still ~0 on a cold fire.
> This stone is SETUP.

## The measurement

`fire_fixpoint_delta` SETUP does:

```
delta_facts = wm.facts.iter().cloned().collect()   // 40,200 Value clones
seen        = delta_facts.iter().cloned().collect() // 40,200 clones + Hash
arm         = rete_arm_get_or_build(...)            // intern hit
```

`Value::Aggregate` is `Arc` — clone is a refcount. `Hash` of an
aggregate is **structural** (class + fields, `value.rs`). One
aggregate hash was already measured at ~121 ns (production
`seen` comment). 40,200 × 121 ns ≈ **4.9 ms** of hash. The
second collect is another 40,200 refcount clones into a `Vec`
we only need so the first `for fact in &delta_facts` has
somewhere to point.

The first worklist **is** `wm.facts`. We do not need a parallel
`Vec` of those Values.

## The algorithm

```
input = wm.facts           // PV handle, Arc bump
seen  = HashSet::with_capacity(input.len())
for f in input.iter() { seen.insert(f.clone()); }   // ONE clone+hash

round 1:  for fact in input.iter() { alpha_activate(fact) }
round 2+: for fact in &owned_delta { alpha_activate(fact) }
```

`alpha_activate` is the existing step-1 body, extracted so the
two worklists share it. Derived `next_delta` is still an owned
`Vec` (new Values). `seen.insert` on those already reports
newness (the production reserve stays).

## ★ THE ONE CONTRACT DECISION

**Every input fact is in `seen` before any derived fact is
considered.** Same membership as today. We do not skip hashing
inputs (a rule may emit a fact equal to an input). We do not
hash pointer identity. We do not add a hasher crate in this
stone — one SipHash of the inputs is the remaining SETUP tax
and is honest.

Order of the first alpha pass follows `wm.facts` iteration
(the PV), not `HashSet` iteration. Today it followed the
cloned `Vec`, which was also PV order. Unchanged.

## The gate

1. SETUP has a `setup:seen` mark. No `delta_facts.iter().cloned()`
   into `seen`. First-round alpha iterates the facts PV.
2. `accum_fire_phase_census` `[200 200]`: fold < 25, snapshot
   < 1. SETUP is printed, **not** wall-gated (2b taught us).
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

~5–8 ms off SETUP (the extra 40k clone + Vec). Hash remains
~5 ms. Quiet FIRE ~84 → ~76–80. If SETUP barely moves, the
14 ms *is* the hash — say so; a faster hasher is a later stone.

## Blast radius

`src/rete/kernel.rs` only. No `.wat`. No new crate.

## Out of scope = REJECTED

- `FxHash` / `rustc-hash` (new crate; only after this proves
  the leftover is the hasher).
- Pointer-hash / dropping inputs from `seen`.
- Persist gather. Cross-call TM.

## Sequencing

1. Extract `alpha_activate_fact`.
2. First worklist = facts PV. `seen` once.
3. Weigh SETUP in the table. Stop.
