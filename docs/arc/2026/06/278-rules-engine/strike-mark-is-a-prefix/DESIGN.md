# DESIGN — D1: the invariant we test is not the invariant we rely on

> Drawn 2026-09-05 at HEAD `7f4bb3699`. Source: vigilia 2026-09-05 D1 (`vocare`), **and my board row
> for it was under-specified.** Reading the file corrected me; this DESIGN records the correction.

## What my board row got wrong

RETE-BOARD called `right_index_counter_invariant.rs` a tautology and stopped there. The file is far
more careful than that: applicability guards, a ★ "the two writers met on ONE index" check read off
`RIGHT_IDX_SITE_MAINTAINER`, and a control test that **drives** the guard and requires it to refuse a
single-HashJoin shape. Its `maintained_joins()` was already corrected post-cure, with the reason
written down: reading mark presence *"would have left the ★ guard passing on a workload where the
maintainer never ran at all: a guard that cannot fail."*

**So the file is not vacuous. The problem is narrower and worse.**

## The real defect: two different facts wearing one number

`JoinRightIndex::already(join_id)` returns `indexed_n[join]` — documented as *"how many elements have
been pushed into `buckets[join]`"*. A **count of pushes**.

`fire/mod.rs:829-833` uses it as a **prefix offset into a different sequence**:

```rust
let already = idx.right_idx.already(join_id);
if already < right_elements.len() {
    for el in &right_elements[already..] { … }
```

Those coincide only if every push came from `right_elements`, **in order, from 0, with nothing
skipped and nothing pushed twice**.

**`mark == Σ|buckets|` — the thing the test asserts — is TRUE whenever the prefix property is FALSE**,
because every push advances the mark by one regardless of *which* element it pushed. The cure made
that assertion hold by construction (the file says so: *"a FOURTH writer that appends without
advancing cannot be written"*), so today it has one possible outcome — while the property the code
actually depends on is checked by nothing.

`vocare` named exactly this and it was never closed: *"session.rs:238 assumes the mark is also the
length of the alpha prefix already indexed, which is what `already` slices on… in both directions
mark and population move together, so the invariant stays green."*

## What protects it today, and why that is not enough

`sequi` L2-a: the first-keying catch-up (`hash_join.rs`) pushes **the entire** `all_right`, not
`right[already..]`. If it ever ran with `already > 0` it would re-push the whole memory: buckets
doubled, mark advanced past a prefix that was never indexed, and `mark == Σ|buckets|` still green.

It does not run in that state because the catch-up gate and the maintainer are coupled — and **A1
changed which mechanism provides that coupling**. It used to be `join_keys_cache` membership; it is
now `left_idx.is_keyed`, set by `key_and_index`, which `keyed_join_persistent` calls
(`fire/mod.rs:812`). The protection survived the refactor by luck of call order, and **nothing
asserts it**.

## The one contract decision, pinned

**Make the mark's meaning structural, then check the property that is relied upon.**

- **Shape rung:** the catch-up indexes `right[already..]`, like `keyed_join_persistent` does, so
  *every* writer respects the mark and "the mark is the length of the indexed prefix" is true by
  construction rather than by call order. This is `sequi` L2-a, cut from A1 and now the cure.
- **Check rung:** the strike test asserts the **prefix property** — the indexed elements are exactly
  `right_elements[0..mark]` — not merely that a count matches a sum.

Keeping `mark == Σ|buckets|` is fine as a cheap second reading. It must not be the *only* one.

## Scope

**IN:** the catch-up's tail-only walk; the prefix assertion; a probe that tries to VIOLATE the prefix
property. Floor GREEN at the end.

**OUT, affirmatively cut:** the left index (A1's `JoinLeftIndex` has no `already` offset — `writer()`
appends Δleft and slices nothing, so this class does not apply); F2; the CLASS A remnants.

## ⛔ First job is to try to break it

**This is LATENT, not driven.** I have not constructed a fire where mark ≠ indexed-prefix-length.
The strike's first act is to attempt exactly that. If it cannot be constructed, say so plainly and
close it structurally anyway — a property held by call order is one refactor from being false, and
A1 already moved the mechanism once.
